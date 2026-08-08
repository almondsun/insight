use crate::model::*;
use rusqlite::{params, Connection, OptionalExtension};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
pub fn open(path: &Path) -> Result<Connection, String> {
    let c = Connection::open(path).map_err(err)?;
    c.pragma_update(None, "foreign_keys", "ON").map_err(err)?;
    let version: i64 = c
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(err)?;
    if version > 2 {
        return Err(format!(
            "Database schema version {version} is newer than this app supports"
        ));
    }
    c.execute_batch(
        "BEGIN IMMEDIATE;
         CREATE TABLE IF NOT EXISTS accounts(id INTEGER PRIMARY KEY,label TEXT NOT NULL,username TEXT,created_at TEXT NOT NULL);
         CREATE TABLE IF NOT EXISTS snapshots(id INTEGER PRIMARY KEY,account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,imported_at TEXT NOT NULL,source_name TEXT NOT NULL,state_hash TEXT NOT NULL,followers INTEGER NOT NULL,following INTEGER NOT NULL,UNIQUE(account_id,state_hash));
         CREATE TABLE IF NOT EXISTS relationships(snapshot_id INTEGER NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,kind TEXT NOT NULL,norm TEXT NOT NULL,username TEXT NOT NULL,profile_url TEXT,source_timestamp INTEGER,PRIMARY KEY(snapshot_id,kind,norm));
         CREATE INDEX IF NOT EXISTS idx_rel_snapshot_kind ON relationships(snapshot_id,kind);
         CREATE TABLE IF NOT EXISTS fame_observations(
           id INTEGER PRIMARY KEY,
           norm TEXT NOT NULL,
           followers INTEGER NOT NULL CHECK(followers >= 0),
           following INTEGER NOT NULL CHECK(following >= 0),
           precision TEXT NOT NULL CHECK(precision IN ('exact','approximate')),
           observed_at TEXT NOT NULL,
           source TEXT NOT NULL,
           corpus_release TEXT NOT NULL,
           authenticated INTEGER NOT NULL CHECK(authenticated=1),
           UNIQUE(id,norm,precision),
           UNIQUE(norm,observed_at,source,corpus_release)
         );
         CREATE TABLE IF NOT EXISTS fame_runs(
           id INTEGER PRIMARY KEY,
           snapshot_id INTEGER NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,
           created_at TEXT NOT NULL,
           completed_at TEXT,
           status TEXT NOT NULL CHECK(status IN ('pending','active','paused','completed','cancelled','blocked','failed')),
           formula_version TEXT NOT NULL,
           corpus_release TEXT,
           profile_version TEXT,
           refresh_all INTEGER NOT NULL DEFAULT 0 CHECK(refresh_all IN (0,1))
         );
         CREATE UNIQUE INDEX IF NOT EXISTS idx_one_active_fame_run ON fame_runs((1)) WHERE status='active';
         CREATE TABLE IF NOT EXISTS fame_run_members(
           run_id INTEGER NOT NULL REFERENCES fame_runs(id) ON DELETE CASCADE,
           norm TEXT NOT NULL,
           username TEXT NOT NULL,
           status TEXT NOT NULL CHECK(status IN ('pending','exact','approximate','private','missing','blocked','failed','cancelled')),
           observation_id INTEGER,
           PRIMARY KEY(run_id,norm),
           FOREIGN KEY(observation_id,norm,status) REFERENCES fame_observations(id,norm,precision) ON DELETE RESTRICT,
           CHECK((status IN ('exact','approximate') AND observation_id IS NOT NULL) OR (status NOT IN ('exact','approximate') AND observation_id IS NULL))
         );
         CREATE INDEX IF NOT EXISTS idx_fame_members_status ON fame_run_members(run_id,status);
         PRAGMA user_version=2;
         COMMIT;",
    )
    .map_err(err)?;
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
    let owner = p
        .detected_username
        .as_deref()
        .filter(|username| crate::parser::is_valid_username(username))
        .ok_or("A validated archive owner is required")?;
    let tx = c.transaction().map_err(err)?;
    let aid = match account {
        Some(id) => {
            let existing: Option<(i64, Option<String>)> = tx
                .query_row("SELECT id,username FROM accounts WHERE id=?", [id], |r| {
                    Ok((r.get(0)?, r.get(1)?))
                })
                .optional()
                .map_err(err)?;
            let (id, existing_username) = existing.ok_or("Account no longer exists")?;
            if let (Some(existing), Some(imported)) =
                (existing_username.as_deref(), p.detected_username.as_deref())
            {
                if crate::parser::normalize(existing) != crate::parser::normalize(imported) {
                    return Err("The archive owner does not match the selected account".into());
                }
            } else if existing_username.is_none() && p.detected_username.is_some() {
                tx.execute(
                    "UPDATE accounts SET username=? WHERE id=?",
                    params![p.detected_username, id],
                )
                .map_err(err)?;
            }
            id
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
                    owner,
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

fn valid_relationship_kind(kind: &str) -> bool {
    matches!(
        kind,
        "followers"
            | "following"
            | "mutuals"
            | "not_following_back"
            | "followers_not_followed_back"
    )
}
pub fn relationships(
    c: &Connection,
    sid: i64,
    kind: &str,
    search: &str,
) -> Result<Vec<Relationship>, String> {
    if !valid_relationship_kind(kind) {
        return Err("Unsupported relationship category".into());
    }
    let exists = c
        .query_row("SELECT 1 FROM snapshots WHERE id=?", [sid], |_| Ok(()))
        .optional()
        .map_err(err)?
        .is_some();
    if !exists {
        return Err("Snapshot no longer exists".into());
    }
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
    if !ids.iter().any(|snapshot| snapshot.id == current) {
        return Err("Snapshot does not belong to the selected account".into());
    }
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
        new_followers: previous.map_or(0, |_| fs.difference(&ps).count()),
        lost_followers: previous.map_or(0, |_| ps.difference(&fs).count()),
        has_previous_snapshot: previous.is_some(),
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
fn delete_orphaned_observations(c: &Connection) -> Result<(), String> {
    c.execute(
        "DELETE FROM fame_observations WHERE NOT EXISTS (SELECT 1 FROM fame_run_members WHERE observation_id=fame_observations.id)",
        [],
    )
    .map_err(err)?;
    Ok(())
}

pub fn delete_snapshot(c: &mut Connection, id: i64) -> Result<(), String> {
    let tx = c.transaction().map_err(err)?;
    tx.execute("DELETE FROM snapshots WHERE id=?", [id])
        .map_err(err)?;
    delete_orphaned_observations(&tx)?;
    tx.commit().map_err(err)?;
    Ok(())
}
pub fn delete_account(c: &mut Connection, id: i64) -> Result<(), String> {
    let tx = c.transaction().map_err(err)?;
    tx.execute("DELETE FROM accounts WHERE id=?", [id])
        .map_err(err)?;
    delete_orphaned_observations(&tx)?;
    tx.commit().map_err(err)?;
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
            detected_username: Some("owner".into()),
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
    fn baseline_has_no_change_counts() {
        let mut c = open(Path::new(":memory:")).unwrap();
        let snapshot = commit(&mut c, &parsed(&["a", "b"], &["b"], "1"), None, "me").unwrap();
        let summary = summary(&c, snapshot.account_id, Some(snapshot.id)).unwrap();
        assert!(!summary.has_previous_snapshot);
        assert_eq!((summary.new_followers, summary.lost_followers), (0, 0));
    }

    #[test]
    fn migrates_v1_and_cascades_fame_runs() {
        let path = tempfile::NamedTempFile::new().unwrap();
        let c = Connection::open(path.path()).unwrap();
        c.execute_batch(
            "PRAGMA foreign_keys=ON;
             CREATE TABLE accounts(id INTEGER PRIMARY KEY,label TEXT NOT NULL,username TEXT,created_at TEXT NOT NULL);
             CREATE TABLE snapshots(id INTEGER PRIMARY KEY,account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,imported_at TEXT NOT NULL,source_name TEXT NOT NULL,state_hash TEXT NOT NULL,followers INTEGER NOT NULL,following INTEGER NOT NULL,UNIQUE(account_id,state_hash));
             CREATE TABLE relationships(snapshot_id INTEGER NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,kind TEXT NOT NULL,norm TEXT NOT NULL,username TEXT NOT NULL,profile_url TEXT,source_timestamp INTEGER,PRIMARY KEY(snapshot_id,kind,norm));
             PRAGMA user_version=1;",
        )
        .unwrap();
        drop(c);
        let c = open(path.path()).unwrap();
        assert_eq!(
            c.pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
                .unwrap(),
            2
        );
        c.execute("INSERT INTO accounts VALUES(1,'me',NULL,'now')", [])
            .unwrap();
        c.execute(
            "INSERT INTO snapshots VALUES(1,1,'now','test','hash',1,1)",
            [],
        )
        .unwrap();
        c.execute(
            "INSERT INTO fame_runs(id,snapshot_id,created_at,status,formula_version,refresh_all) VALUES(1,1,'now','pending','fame-v1',0)",
            [],
        )
        .unwrap();
        c.execute("DELETE FROM snapshots WHERE id=1", []).unwrap();
        let remaining: i64 = c
            .query_row("SELECT COUNT(*) FROM fame_runs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(remaining, 0);
    }

    #[test]
    fn deletion_collects_only_unreferenced_observations() {
        let mut c = open(Path::new(":memory:")).unwrap();
        let first = commit(&mut c, &parsed(&["a"], &["b"], "1"), None, "me").unwrap();
        let second = commit(
            &mut c,
            &parsed(&["a", "c"], &["b"], "2"),
            Some(first.account_id),
            "me",
        )
        .unwrap();
        c.execute(
            "INSERT INTO fame_observations(id,norm,followers,following,precision,observed_at,source,corpus_release,authenticated) VALUES(1,'a',10,1,'exact','now','test','r1',1)",
            [],
        )
        .unwrap();
        for (run, snapshot) in [(1, first.id), (2, second.id)] {
            c.execute(
                "INSERT INTO fame_runs(id,snapshot_id,created_at,status,formula_version,refresh_all) VALUES(?,?,'now','completed','fame-v1',0)",
                params![run, snapshot],
            )
            .unwrap();
            c.execute(
                "INSERT INTO fame_run_members(run_id,norm,username,status,observation_id) VALUES(?,'a','a','exact',1)",
                [run],
            )
            .unwrap();
        }
        delete_snapshot(&mut c, first.id).unwrap();
        assert_eq!(
            c.query_row("SELECT COUNT(*) FROM fame_observations", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
        delete_snapshot(&mut c, second.id).unwrap();
        assert_eq!(
            c.query_row("SELECT COUNT(*) FROM fame_observations", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn fame_member_precision_must_match_observation() {
        let mut c = open(Path::new(":memory:")).unwrap();
        let snapshot = commit(&mut c, &parsed(&["a"], &["b"], "1"), None, "me").unwrap();
        c.execute(
            "INSERT INTO fame_observations(id,norm,followers,following,precision,observed_at,source,corpus_release,authenticated) VALUES(1,'a',10,1,'approximate','now','test','r1',1)",
            [],
        )
        .unwrap();
        c.execute(
            "INSERT INTO fame_runs(id,snapshot_id,created_at,status,formula_version,refresh_all) VALUES(1,?,'now','completed','fame-v1',0)",
            [snapshot.id],
        )
        .unwrap();
        let mismatch = c.execute(
            "INSERT INTO fame_run_members(run_id,norm,username,status,observation_id) VALUES(1,'a','a','exact',1)",
            [],
        );
        assert!(mismatch.is_err());
        c.execute(
            "INSERT INTO fame_run_members(run_id,norm,username,status,observation_id) VALUES(1,'a','a','approximate',1)",
            [],
        )
        .unwrap();
    }

    #[test]
    #[ignore = "requires INSIGHT_REAL_EXPORT to point to a private local export"]
    fn imports_real_export_through_sqlite() {
        let source = std::env::var("INSIGHT_REAL_EXPORT").expect("INSIGHT_REAL_EXPORT is required");
        let mut parsed = parser::parse_path(Path::new(&source)).expect("real export should parse");
        if parsed.detected_username.is_none() {
            parsed.detected_username = Some("e2e_owner".into());
        }
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
