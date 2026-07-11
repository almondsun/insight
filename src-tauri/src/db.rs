use crate::model::*;
use rusqlite::{params, Connection, OptionalExtension};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
pub fn open(path: &Path) -> Result<Connection, String> {
    let c = Connection::open(path).map_err(err)?;
    c.pragma_update(None, "foreign_keys", "ON").map_err(err)?;
    c.execute_batch("CREATE TABLE IF NOT EXISTS accounts(id INTEGER PRIMARY KEY,label TEXT NOT NULL,username TEXT,created_at TEXT NOT NULL);CREATE TABLE IF NOT EXISTS snapshots(id INTEGER PRIMARY KEY,account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,imported_at TEXT NOT NULL,source_name TEXT NOT NULL,state_hash TEXT NOT NULL,followers INTEGER NOT NULL,following INTEGER NOT NULL,UNIQUE(account_id,state_hash));CREATE TABLE IF NOT EXISTS relationships(snapshot_id INTEGER NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,kind TEXT NOT NULL,norm TEXT NOT NULL,username TEXT NOT NULL,profile_url TEXT,source_timestamp INTEGER,PRIMARY KEY(snapshot_id,kind,norm));CREATE INDEX IF NOT EXISTS idx_rel_snapshot_kind ON relationships(snapshot_id,kind);PRAGMA user_version=1;").map_err(err)?;
    Ok(c)
}
fn err(e: rusqlite::Error) -> String {
    e.to_string()
}
pub fn accounts(c: &Connection) -> Result<Vec<Account>, String> {
    let mut q=c.prepare("SELECT a.id,a.label,a.username,COUNT(s.id) FROM accounts a LEFT JOIN snapshots s ON s.account_id=a.id GROUP BY a.id ORDER BY a.created_at").map_err(err)?;
    let rows = q
        .query_map([], |r| {
            Ok(Account {
                id: r.get(0)?,
                label: r.get(1)?,
                username: r.get(2)?,
                snapshot_count: r.get(3)?,
            })
        })
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err);
    rows
}
pub fn snapshots(c: &Connection, account: i64) -> Result<Vec<Snapshot>, String> {
    let mut q=c.prepare("SELECT id,account_id,imported_at,source_name,followers,following FROM snapshots WHERE account_id=? ORDER BY imported_at DESC,id DESC").map_err(err)?;
    let rows = q
        .query_map([account], |r| {
            Ok(Snapshot {
                id: r.get(0)?,
                account_id: r.get(1)?,
                imported_at: r.get(2)?,
                source_name: r.get(3)?,
                followers: r.get::<_, i64>(4)? as usize,
                following: r.get::<_, i64>(5)? as usize,
            })
        })
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err);
    rows
}
pub fn commit(
    c: &mut Connection,
    p: &ParsedImport,
    account: Option<i64>,
    label: &str,
) -> Result<Snapshot, String> {
    let tx = c.transaction().map_err(err)?;
    let aid = match account {
        Some(id) => {
            let ok: Option<i64> = tx
                .query_row("SELECT id FROM accounts WHERE id=?", [id], |r| r.get(0))
                .optional()
                .map_err(err)?;
            ok.ok_or("Account no longer exists")?
        }
        None => {
            tx.execute(
                "INSERT INTO accounts(label,username,created_at) VALUES(?,?,?)",
                params![
                    if label.trim().is_empty() {
                        "Instagram account"
                    } else {
                        label.trim()
                    },
                    p.detected_username,
                    chrono::Utc::now().to_rfc3339()
                ],
            )
            .map_err(err)?;
            tx.last_insert_rowid()
        }
    };
    if tx
        .query_row(
            "SELECT 1 FROM snapshots WHERE account_id=? AND state_hash=?",
            params![aid, p.hash],
            |r| r.get::<_, i64>(0),
        )
        .optional()
        .map_err(err)?
        .is_some()
    {
        return Err("This relationship snapshot has already been imported for this account".into());
    }
    let now = chrono::Utc::now().to_rfc3339();
    tx.execute("INSERT INTO snapshots(account_id,imported_at,source_name,state_hash,followers,following) VALUES(?,?,?,?,?,?)",params![aid,now,p.source_name,p.hash,p.followers.len() as i64,p.following.len() as i64]).map_err(err)?;
    let sid = tx.last_insert_rowid();
    for (kind, map) in [("followers", &p.followers), ("following", &p.following)] {
        let mut stmt=tx.prepare("INSERT INTO relationships(snapshot_id,kind,norm,username,profile_url,source_timestamp) VALUES(?,?,?,?,?,?)").map_err(err)?;
        for (norm, x) in map {
            stmt.execute(params![
                sid,
                kind,
                norm,
                x.username,
                x.profile_url,
                x.timestamp
            ])
            .map_err(err)?;
        }
    }
    tx.commit().map_err(err)?;
    Ok(Snapshot {
        id: sid,
        account_id: aid,
        imported_at: now,
        source_name: p.source_name.clone(),
        followers: p.followers.len(),
        following: p.following.len(),
    })
}
pub fn sets(c: &Connection, sid: i64) -> Result<(People, People), String> {
    let mut f = BTreeMap::new();
    let mut g = BTreeMap::new();
    let mut q=c.prepare("SELECT kind,norm,username,profile_url,source_timestamp FROM relationships WHERE snapshot_id=?").map_err(err)?;
    let rows = q
        .query_map([sid], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                Person {
                    username: r.get(2)?,
                    profile_url: r.get(3)?,
                    timestamp: r.get(4)?,
                },
            ))
        })
        .map_err(err)?;
    for row in rows {
        let (k, n, p) = row.map_err(err)?;
        if k == "followers" {
            f.insert(n, p);
        } else {
            g.insert(n, p);
        }
    }
    Ok((f, g))
}
fn selected(
    kind: &str,
    f: &BTreeMap<String, Person>,
    g: &BTreeMap<String, Person>,
) -> BTreeSet<String> {
    let fs = f.keys().cloned().collect::<BTreeSet<_>>();
    let gs = g.keys().cloned().collect::<BTreeSet<_>>();
    match kind {
        "followers" => fs,
        "following" => gs,
        "mutuals" => fs.intersection(&gs).cloned().collect(),
        "not_following_back" => gs.difference(&fs).cloned().collect(),
        "followers_not_followed_back" => fs.difference(&gs).cloned().collect(),
        _ => BTreeSet::new(),
    }
}
pub fn relationships(
    c: &Connection,
    sid: i64,
    kind: &str,
    search: &str,
) -> Result<Vec<Relationship>, String> {
    let (f, g) = sets(c, sid)?;
    let query = search.to_lowercase();
    Ok(selected(kind, &f, &g)
        .into_iter()
        .filter(|x| x.contains(&query))
        .filter_map(|key| {
            f.get(&key).or_else(|| g.get(&key)).map(|p| Relationship {
                username: p.username.clone(),
                profile_url: p.profile_url.clone(),
                kind: kind.into(),
            })
        })
        .collect())
}
pub fn summary(c: &Connection, account: i64, sid: Option<i64>) -> Result<Summary, String> {
    let ids = snapshots(c, account)?;
    let current = sid
        .or_else(|| ids.first().map(|s| s.id))
        .ok_or("No snapshots found")?;
    let (f, g) = sets(c, current)?;
    let previous = ids
        .iter()
        .position(|s| s.id == current)
        .and_then(|i| ids.get(i + 1));
    let (pf, _) = previous
        .map(|x| sets(c, x.id))
        .transpose()?
        .unwrap_or_default();
    let fs = f.keys().cloned().collect::<BTreeSet<_>>();
    let ps = pf.keys().cloned().collect::<BTreeSet<_>>();
    Ok(Summary {
        followers: f.len(),
        following: g.len(),
        mutuals: selected("mutuals", &f, &g).len(),
        not_following_back: selected("not_following_back", &f, &g).len(),
        followers_not_followed_back: selected("followers_not_followed_back", &f, &g).len(),
        new_followers: fs.difference(&ps).count(),
        lost_followers: ps.difference(&fs).count(),
    })
}
pub fn compare(c: &Connection, from: i64, to: i64) -> Result<Vec<Change>, String> {
    let account1: i64 = c
        .query_row("SELECT account_id FROM snapshots WHERE id=?", [from], |r| {
            r.get(0)
        })
        .map_err(err)?;
    let account2: i64 = c
        .query_row("SELECT account_id FROM snapshots WHERE id=?", [to], |r| {
            r.get(0)
        })
        .map_err(err)?;
    if account1 != account2 {
        return Err("Snapshots must belong to the same account".into());
    }
    let (ff, fg) = sets(c, from)?;
    let (tf, tg) = sets(c, to)?;
    let mut out = Vec::new();
    for category in [
        "followers",
        "following",
        "mutuals",
        "not_following_back",
        "followers_not_followed_back",
    ] {
        let a = selected(category, &ff, &fg);
        let b = selected(category, &tf, &tg);
        for (key, direction) in a
            .difference(&b)
            .map(|x| (x, "removed"))
            .chain(b.difference(&a).map(|x| (x, "added")))
        {
            let p = tf
                .get(key)
                .or_else(|| tg.get(key))
                .or_else(|| ff.get(key))
                .or_else(|| fg.get(key))
                .unwrap();
            out.push(Change {
                username: p.username.clone(),
                profile_url: p.profile_url.clone(),
                category: category.into(),
                direction: direction.into(),
            });
        }
    }
    Ok(out)
}
pub fn delete_snapshot(c: &Connection, id: i64) -> Result<(), String> {
    c.execute("DELETE FROM snapshots WHERE id=?", [id])
        .map_err(err)?;
    Ok(())
}
pub fn delete_account(c: &Connection, id: i64) -> Result<(), String> {
    c.execute("DELETE FROM accounts WHERE id=?", [id])
        .map_err(err)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use std::collections::BTreeMap;
    fn parsed(f: &[&str], g: &[&str], hash: &str) -> ParsedImport {
        let map = |xs: &[&str]| {
            xs.iter()
                .map(|x| {
                    (
                        x.to_string(),
                        Person {
                            username: x.to_string(),
                            profile_url: None,
                            timestamp: None,
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>()
        };
        ParsedImport {
            source_name: "test".into(),
            detected_username: None,
            followers: map(f),
            following: map(g),
            warnings: vec![],
            hash: hash.into(),
        }
    }
    #[test]
    fn persists_and_compares() {
        let mut c = open(Path::new(":memory:")).unwrap();
        let a = commit(&mut c, &parsed(&["a", "b"], &["b", "c"], "1"), None, "me").unwrap();
        let b = commit(
            &mut c,
            &parsed(&["b", "d"], &["b"], "2"),
            Some(a.account_id),
            "me",
        )
        .unwrap();
        let s = summary(&c, a.account_id, Some(b.id)).unwrap();
        assert_eq!((s.new_followers, s.lost_followers), (1, 1));
        let ch = compare(&c, a.id, b.id).unwrap();
        assert!(ch
            .iter()
            .any(|x| x.username == "d" && x.direction == "added"));
    }

    #[test]
    #[ignore = "requires INSIGHT_REAL_EXPORT to point to a private local export"]
    fn imports_real_export_through_sqlite() {
        let source = std::env::var("INSIGHT_REAL_EXPORT").expect("INSIGHT_REAL_EXPORT is required");
        let parsed = parser::parse_path(Path::new(&source)).expect("real export should parse");
        let expected_followers = parsed.followers.len();
        let expected_following = parsed.following.len();
        let dir = tempfile::tempdir().expect("temporary directory should be available");
        let database = dir.path().join("insight.db");

        let snapshot = {
            let mut connection = open(&database).expect("database should open");
            let snapshot = commit(&mut connection, &parsed, None, "E2E account")
                .expect("snapshot should commit");
            let duplicate = commit(
                &mut connection,
                &parsed,
                Some(snapshot.account_id),
                "E2E account",
            );
            assert!(duplicate.is_err(), "duplicate state should be rejected");
            snapshot
        };

        let connection = open(&database).expect("database should reopen");
        let result = summary(&connection, snapshot.account_id, Some(snapshot.id))
            .expect("summary should load");
        assert_eq!(result.followers, expected_followers);
        assert_eq!(result.following, expected_following);
        assert_eq!(
            relationships(&connection, snapshot.id, "followers", "")
                .expect("followers should load")
                .len(),
            expected_followers
        );
        assert_eq!(
            relationships(&connection, snapshot.id, "following", "")
                .expect("following should load")
                .len(),
            expected_following
        );
        eprintln!(
            "persisted {} followers, {} following, and {} mutuals",
            result.followers, result.following, result.mutuals
        );
    }
}
