use crate::model::*;
use rusqlite::{backup::Backup, params, Connection, OpenFlags, OptionalExtension};
#[cfg(test)]
use std::collections::{BTreeMap, BTreeSet};
use std::{path::Path, time::Duration};
pub const APPLICATION_ID: i64 = 0x4E49_5655;
const LEGACY_APPLICATION_ID: i64 = 0x494E_5347;

pub fn is_compatible_application_id(value: i64) -> bool {
    matches!(value, APPLICATION_ID | LEGACY_APPLICATION_ID)
}
pub fn open(path: &Path) -> Result<Connection, String> {
    prepare_private_database_file(path)?;
    let c = Connection::open(path).map_err(err)?;
    c.busy_timeout(Duration::from_secs(5)).map_err(err)?;
    c.pragma_update(None, "foreign_keys", "ON").map_err(err)?;
    c.pragma_update(None, "secure_delete", "ON").map_err(err)?;
    c.pragma_update(None, "trusted_schema", "OFF")
        .map_err(err)?;
    c.pragma_update(None, "journal_mode", "WAL").map_err(err)?;
    set_private_file_permissions(path)?;
    let version: i64 = c
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(err)?;
    if version > 3 {
        return Err(format!(
            "Database schema version {version} is newer than this app supports"
        ));
    }
    let application_id: i64 = c
        .pragma_query_value(None, "application_id", |row| row.get(0))
        .map_err(err)?;
    if application_id != 0 && !is_compatible_application_id(application_id) {
        return Err("Database belongs to a different application".into());
    }
    c.execute_batch(
        "BEGIN IMMEDIATE;
         CREATE TABLE IF NOT EXISTS accounts(id INTEGER PRIMARY KEY,label TEXT NOT NULL,username TEXT,created_at TEXT NOT NULL);
         CREATE TABLE IF NOT EXISTS snapshots(id INTEGER PRIMARY KEY,account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,imported_at TEXT NOT NULL,observed_at TEXT,observed_at_source TEXT,source_name TEXT NOT NULL,state_hash TEXT NOT NULL,followers INTEGER NOT NULL,following INTEGER NOT NULL,UNIQUE(account_id,state_hash));
         CREATE TABLE IF NOT EXISTS relationships(snapshot_id INTEGER NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,kind TEXT NOT NULL,norm TEXT NOT NULL,username TEXT NOT NULL,profile_url TEXT,source_timestamp INTEGER,PRIMARY KEY(snapshot_id,kind,norm));
         CREATE INDEX IF NOT EXISTS idx_rel_snapshot_kind ON relationships(snapshot_id,kind);
         COMMIT;",
    )
    .map_err(err)?;
    if version < 2 {
        crate::fame_store::migrate(&c)?;
    }
    if version < 3 {
        let has_observed_at = c
            .prepare("PRAGMA table_info(snapshots)")
            .map_err(err)?
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?
            .iter()
            .any(|column| column == "observed_at");
        if !has_observed_at {
            c.execute_batch(
                "ALTER TABLE snapshots ADD COLUMN observed_at TEXT;
                 ALTER TABLE snapshots ADD COLUMN observed_at_source TEXT;",
            )
            .map_err(err)?;
        }
        c.execute_batch(
            "BEGIN IMMEDIATE;
             UPDATE snapshots SET observed_at=substr(imported_at,1,10), observed_at_source='legacy_import_time' WHERE observed_at IS NULL;
             CREATE INDEX IF NOT EXISTS idx_snapshots_account_observed ON snapshots(account_id,observed_at DESC,id DESC);
             CREATE INDEX IF NOT EXISTS idx_rel_kind_norm_snapshot ON relationships(kind,norm,snapshot_id);
             PRAGMA user_version=3;
             COMMIT;",
        )
        .map_err(err)?;
    }
    c.pragma_update(None, "application_id", APPLICATION_ID)
        .map_err(err)?;
    Ok(c)
}

pub fn migrate_legacy_database(legacy: &Path, destination: &Path) -> Result<bool, String> {
    if destination.exists() || !legacy.exists() {
        return Ok(false);
    }
    let parent = destination
        .parent()
        .ok_or("Nivune database path has no parent directory")?;
    let source =
        Connection::open_with_flags(legacy, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(err)?;
    source.busy_timeout(Duration::from_secs(5)).map_err(err)?;
    let temporary = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    let mut target = Connection::open(temporary.path()).map_err(err)?;
    target
        .pragma_update(None, "journal_mode", "DELETE")
        .map_err(err)?;
    Backup::new(&source, &mut target)
        .and_then(|backup| backup.run_to_completion(128, Duration::from_millis(2), None))
        .map_err(err)?;
    target.close().map_err(|(_, error)| error.to_string())?;
    source.close().map_err(|(_, error)| error.to_string())?;
    let migrated = open(temporary.path())?;
    migrated
        .pragma_update(None, "journal_mode", "DELETE")
        .map_err(err)?;
    migrated.close().map_err(|(_, error)| error.to_string())?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|error| error.to_string())?;
    temporary
        .persist_noclobber(destination)
        .map_err(|error| error.to_string())?;
    set_private_file_permissions(destination)?;
    Ok(true)
}

#[cfg(unix)]
fn prepare_private_database_file(path: &Path) -> Result<(), String> {
    use std::{fs::OpenOptions, io::ErrorKind, os::unix::fs::OpenOptionsExt};
    if path == Path::new(":memory:") {
        return Ok(());
    }
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
    {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

#[cfg(not(unix))]
fn prepare_private_database_file(_path: &Path) -> Result<(), String> {
    Ok(())
}
fn err(e: rusqlite::Error) -> String {
    e.to_string()
}

#[cfg(unix)]
fn set_private_file_permissions(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    if path != Path::new(":memory:") {
        let permissions = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, permissions).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn set_private_file_permissions(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn validated_label(label: &str) -> Result<&str, String> {
    let label = label.trim();
    if label.is_empty() || label.chars().count() > 80 {
        return Err("Account label must contain between 1 and 80 characters".into());
    }
    Ok(label)
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
pub fn rename_account(c: &Connection, id: i64, label: &str) -> Result<Account, String> {
    let label = validated_label(label)?;
    if c.execute("UPDATE accounts SET label=? WHERE id=?", params![label, id])
        .map_err(err)?
        == 0
    {
        return Err("Account no longer exists".into());
    }
    c.query_row(
        "SELECT a.id,a.label,a.username,COUNT(s.id) FROM accounts a LEFT JOIN snapshots s ON s.account_id=a.id WHERE a.id=? GROUP BY a.id",
        [id],
        |r| Ok(Account { id: r.get(0)?, label: r.get(1)?, username: r.get(2)?, snapshot_count: r.get(3)? }),
    ).map_err(err)
}
pub fn snapshots(c: &Connection, account: i64) -> Result<Vec<Snapshot>, String> {
    let mut q=c.prepare("SELECT id,account_id,imported_at,observed_at,observed_at_source,source_name,followers,following FROM snapshots WHERE account_id=? ORDER BY observed_at DESC,id DESC").map_err(err)?;
    let rows = q
        .query_map([account], |r| {
            Ok(Snapshot {
                id: r.get(0)?,
                account_id: r.get(1)?,
                imported_at: r.get(2)?,
                observed_at: r.get(3)?,
                observed_at_source: r.get(4)?,
                source_name: r.get(5)?,
                followers: r.get::<_, i64>(6)? as usize,
                following: r.get::<_, i64>(7)? as usize,
            })
        })
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err);
    rows
}
pub fn commit_at(
    c: &mut Connection,
    p: &ParsedImport,
    account: Option<i64>,
    label: &str,
    observed_at: &str,
) -> Result<Snapshot, String> {
    let observed_date = chrono::NaiveDate::parse_from_str(observed_at, "%Y-%m-%d")
        .map_err(|_| "Observation date must be a valid calendar date".to_string())?;
    if observed_date > chrono::Utc::now().date_naive() {
        return Err("Observation date cannot be in the future".into());
    }
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
            let label = validated_label(label)?;
            tx.execute(
                "INSERT INTO accounts(label,username,created_at) VALUES(?,?,?)",
                params![label, owner, chrono::Utc::now().to_rfc3339()],
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
    tx.execute("INSERT INTO snapshots(account_id,imported_at,observed_at,observed_at_source,source_name,state_hash,followers,following) VALUES(?,?,?,?,?,?,?,?)",params![aid,now,observed_at,"user_confirmed",p.source_name,p.hash,p.followers.len() as i64,p.following.len() as i64]).map_err(err)?;
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
        observed_at: observed_at.to_string(),
        observed_at_source: "user_confirmed".into(),
        source_name: p.source_name.clone(),
        followers: p.followers.len(),
        following: p.following.len(),
    })
}

#[cfg(test)]
pub fn commit(
    c: &mut Connection,
    p: &ParsedImport,
    account: Option<i64>,
    label: &str,
) -> Result<Snapshot, String> {
    commit_at(c, p, account, label, "2026-01-01")
}

pub fn trends(c: &Connection, account: i64) -> Result<Vec<TrendPoint>, String> {
    let mut ordered = snapshots(c, account)?;
    ordered.reverse();
    ordered
        .into_iter()
        .map(|snapshot| {
            let values = summary(c, account, Some(snapshot.id))?;
            Ok(TrendPoint {
                snapshot_id: snapshot.id,
                observed_at: snapshot.observed_at,
                followers: values.followers,
                following: values.following,
                mutuals: values.mutuals,
                new_followers: values.new_followers,
                lost_followers: values.lost_followers,
            })
        })
        .collect()
}

pub fn relationship_history(
    c: &Connection,
    account: i64,
    username: &str,
) -> Result<Vec<RelationshipHistoryPoint>, String> {
    if !crate::parser::is_valid_username(username) {
        return Err("Invalid username".into());
    }
    let norm = crate::parser::normalize(username);
    let mut statement = c
        .prepare(
            "SELECT s.id,s.observed_at,
                    EXISTS(SELECT 1 FROM relationships r WHERE r.snapshot_id=s.id AND r.kind='followers' AND r.norm=?2),
                    EXISTS(SELECT 1 FROM relationships r WHERE r.snapshot_id=s.id AND r.kind='following' AND r.norm=?2)
             FROM snapshots s WHERE s.account_id=?1 ORDER BY s.observed_at,s.id",
        )
        .map_err(err)?;
    let points = statement
        .query_map(params![account, norm], |row| {
            Ok(RelationshipHistoryPoint {
                snapshot_id: row.get(0)?,
                observed_at: row.get(1)?,
                follows_you: row.get(2)?,
                you_follow: row.get(3)?,
            })
        })
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    Ok(points)
}
#[cfg(test)]
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
#[cfg(test)]
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

pub fn valid_relationship_kind(kind: &str) -> bool {
    matches!(
        kind,
        "followers"
            | "following"
            | "mutuals"
            | "not_following_back"
            | "followers_not_followed_back"
    )
}

fn category_predicate(kind: &str) -> Result<&'static str, String> {
    match kind {
        "followers" => Ok("has_follower = 1"),
        "following" => Ok("has_following = 1"),
        "mutuals" => Ok("has_follower = 1 AND has_following = 1"),
        "not_following_back" => Ok("has_follower = 0 AND has_following = 1"),
        "followers_not_followed_back" => Ok("has_follower = 1 AND has_following = 0"),
        _ => Err("Unsupported relationship category".into()),
    }
}

fn escape_like(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn validate_page_inputs(search: &str, after: Option<&str>) -> Result<(), String> {
    if search.len() > 30
        || !search
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'_')
    {
        return Err("Search must be at most 30 Instagram username characters".into());
    }
    if after.is_some_and(|cursor| {
        !crate::parser::is_valid_username(cursor) || crate::parser::normalize(cursor) != cursor
    }) {
        return Err("Invalid relationship page cursor".into());
    }
    Ok(())
}

pub fn relationships_page(
    c: &Connection,
    sid: i64,
    kind: &str,
    search: &str,
    after: Option<&str>,
    limit: usize,
) -> Result<RelationshipPage, String> {
    validate_page_inputs(search, after)?;
    category_predicate(kind)?;
    let exists = c
        .query_row("SELECT 1 FROM snapshots WHERE id=?", [sid], |_| Ok(()))
        .optional()
        .map_err(err)?
        .is_some();
    if !exists {
        return Err("Snapshot no longer exists".into());
    }
    let limit = limit.clamp(1, 500);
    let sql = match kind {
        "followers" | "following" => format!(
            "SELECT norm,username,profile_url FROM relationships
             WHERE snapshot_id=?1 AND kind='{kind}' AND norm LIKE ?2 ESCAPE '\\' AND norm > ?3
             ORDER BY norm LIMIT ?4"
        ),
        "mutuals" => "SELECT f.norm,f.username,COALESCE(f.profile_url,g.profile_url)
             FROM relationships f JOIN relationships g
               ON g.snapshot_id=f.snapshot_id AND g.kind='following' AND g.norm=f.norm
             WHERE f.snapshot_id=?1 AND f.kind='followers'
               AND f.norm LIKE ?2 ESCAPE '\\' AND f.norm > ?3
             ORDER BY f.norm LIMIT ?4"
            .into(),
        "not_following_back" => "SELECT g.norm,g.username,g.profile_url
             FROM relationships g LEFT JOIN relationships f
               ON f.snapshot_id=g.snapshot_id AND f.kind='followers' AND f.norm=g.norm
             WHERE g.snapshot_id=?1 AND g.kind='following' AND f.norm IS NULL
               AND g.norm LIKE ?2 ESCAPE '\\' AND g.norm > ?3
             ORDER BY g.norm LIMIT ?4"
            .into(),
        "followers_not_followed_back" => "SELECT f.norm,f.username,f.profile_url
             FROM relationships f LEFT JOIN relationships g
               ON g.snapshot_id=f.snapshot_id AND g.kind='following' AND g.norm=f.norm
             WHERE f.snapshot_id=?1 AND f.kind='followers' AND g.norm IS NULL
               AND f.norm LIKE ?2 ESCAPE '\\' AND f.norm > ?3
             ORDER BY f.norm LIMIT ?4"
            .into(),
        _ => unreachable!("category was validated"),
    };
    let pattern = format!("%{}%", escape_like(search));
    let cursor = after.unwrap_or_default();
    let mut statement = c.prepare(&sql).map_err(err)?;
    let mut rows = statement
        .query_map(params![sid, pattern, cursor, (limit + 1) as i64], |row| {
            let norm: String = row.get(0)?;
            Ok((
                norm,
                Relationship {
                    username: row.get(1)?,
                    profile_url: row.get(2)?,
                    kind: kind.to_string(),
                },
            ))
        })
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    let has_more = rows.len() > limit;
    rows.truncate(limit);
    let next_cursor = has_more
        .then(|| rows.last().map(|(norm, _)| norm.clone()))
        .flatten();
    Ok(RelationshipPage {
        items: rows
            .into_iter()
            .map(|(_, relationship)| relationship)
            .collect(),
        next_cursor,
    })
}
#[cfg(test)]
pub fn relationships(
    c: &Connection,
    sid: i64,
    kind: &str,
    search: &str,
) -> Result<Vec<Relationship>, String> {
    if !valid_relationship_kind(kind) {
        return Err("Unsupported relationship category".into());
    }
    let mut output = Vec::new();
    let mut cursor = None;
    loop {
        let page = relationships_page(c, sid, kind, search, cursor.as_deref(), 500)?;
        output.extend(page.items);
        let Some(next) = page.next_cursor else {
            break;
        };
        cursor = Some(next);
    }
    Ok(output)
}
pub fn summary(c: &Connection, account: i64, sid: Option<i64>) -> Result<Summary, String> {
    let ids = snapshots(c, account)?;
    let current = sid
        .or_else(|| ids.first().map(|s| s.id))
        .ok_or("No snapshots found")?;
    if !ids.iter().any(|snapshot| snapshot.id == current) {
        return Err("Snapshot does not belong to the selected account".into());
    }
    let previous = ids
        .iter()
        .position(|s| s.id == current)
        .and_then(|i| ids.get(i + 1));
    let counts: (i64, i64, i64, i64, i64) = c
        .query_row(
            "WITH membership AS (
               SELECT norm,MAX(kind='followers') AS is_follower,MAX(kind='following') AS is_following
               FROM relationships WHERE snapshot_id=? GROUP BY norm
             )
             SELECT
               COALESCE(SUM(is_follower),0),
               COALESCE(SUM(is_following),0),
               COALESCE(SUM(is_follower AND is_following),0),
               COALESCE(SUM(NOT is_follower AND is_following),0),
               COALESCE(SUM(is_follower AND NOT is_following),0)
             FROM membership",
            [current],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .map_err(err)?;
    let (new_followers, lost_followers) = if let Some(previous) = previous {
        let new_followers = count_membership_difference(c, previous.id, current)?;
        let lost_followers = count_membership_difference(c, current, previous.id)?;
        (new_followers, lost_followers)
    } else {
        (0, 0)
    };
    Ok(Summary {
        followers: count_to_usize(counts.0)?,
        following: count_to_usize(counts.1)?,
        mutuals: count_to_usize(counts.2)?,
        not_following_back: count_to_usize(counts.3)?,
        followers_not_followed_back: count_to_usize(counts.4)?,
        new_followers,
        lost_followers,
        has_previous_snapshot: previous.is_some(),
    })
}

fn count_membership_difference(
    c: &Connection,
    absent_from_snapshot: i64,
    present_in_snapshot: i64,
) -> Result<usize, String> {
    let count = c
        .query_row(
            "SELECT COUNT(*) FROM relationships present
             LEFT JOIN relationships absent
               ON absent.snapshot_id=?1 AND absent.kind='followers' AND absent.norm=present.norm
             WHERE present.snapshot_id=?2 AND present.kind='followers' AND absent.norm IS NULL",
            params![absent_from_snapshot, present_in_snapshot],
            |row| row.get::<_, i64>(0),
        )
        .map_err(err)?;
    count_to_usize(count)
}

fn count_to_usize(value: i64) -> Result<usize, String> {
    usize::try_from(value).map_err(|_| "Database count is outside the supported range".into())
}

#[cfg(test)]
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

fn ensure_comparable(c: &Connection, from: i64, to: i64) -> Result<(), String> {
    if from == to {
        return Err("Choose two different snapshots".into());
    }
    let account_from = c
        .query_row(
            "SELECT account_id FROM snapshots WHERE id=?",
            [from],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(err)?;
    let account_to = c
        .query_row("SELECT account_id FROM snapshots WHERE id=?", [to], |row| {
            row.get::<_, i64>(0)
        })
        .optional()
        .map_err(err)?;
    match (account_from, account_to) {
        (Some(left), Some(right)) if left == right => Ok(()),
        (None, _) | (_, None) => Err("Snapshot no longer exists".into()),
        _ => Err("Snapshots must belong to the same account".into()),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn changes_page(
    c: &Connection,
    from: i64,
    to: i64,
    category: &str,
    search: &str,
    after: Option<&str>,
    direction: Option<&str>,
    limit: usize,
) -> Result<ChangePage, String> {
    validate_page_inputs(search, after)?;
    category_predicate(category)?;
    ensure_comparable(c, from, to)?;
    if direction.is_some_and(|value| !matches!(value, "added" | "removed")) {
        return Err("Unsupported change direction".into());
    }
    let limit = limit.clamp(1, 500);
    let from_chosen = selected_relation_sql(category, "?1", "a");
    let to_chosen = selected_relation_sql(category, "?2", "a");
    let sql = format!(
        "WITH
         from_chosen AS ({from_chosen}),
         to_chosen AS ({to_chosen}),
         changes AS (
           SELECT f.norm,f.username,f.profile_url,'removed' AS direction FROM from_chosen f
             LEFT JOIN to_chosen t ON t.norm=f.norm WHERE t.norm IS NULL
           UNION ALL
           SELECT t.norm,t.username,t.profile_url,'added' AS direction FROM to_chosen t
             LEFT JOIN from_chosen f ON f.norm=t.norm WHERE f.norm IS NULL
         )
         SELECT norm,username,profile_url,direction FROM changes
         WHERE (?5='' OR direction=?5) ORDER BY norm LIMIT ?6"
    );
    let pattern = format!("%{}%", escape_like(search));
    let cursor = after.unwrap_or_default();
    let mut statement = c.prepare(&sql).map_err(err)?;
    let mut rows = statement
        .query_map(
            params![
                from,
                to,
                pattern,
                cursor,
                direction.unwrap_or_default(),
                (limit + 1) as i64
            ],
            |row| {
                let norm: String = row.get(0)?;
                Ok((
                    norm,
                    Change {
                        username: row.get(1)?,
                        profile_url: row.get(2)?,
                        category: category.to_string(),
                        direction: row.get(3)?,
                    },
                ))
            },
        )
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    let has_more = rows.len() > limit;
    rows.truncate(limit);
    let next_cursor = has_more
        .then(|| rows.last().map(|(norm, _)| norm.clone()))
        .flatten();
    Ok(ChangePage {
        items: rows.into_iter().map(|(_, change)| change).collect(),
        next_cursor,
    })
}

fn selected_relation_sql(category: &str, snapshot: &str, alias: &str) -> String {
    let filter = format!("{alias}.norm LIKE ?3 ESCAPE '\\' AND {alias}.norm > ?4");
    match category {
        "followers" | "following" => format!(
            "SELECT {alias}.norm,{alias}.username,{alias}.profile_url
             FROM relationships {alias}
             WHERE {alias}.snapshot_id={snapshot} AND {alias}.kind='{category}' AND {filter}"
        ),
        "mutuals" => format!(
            "SELECT {alias}.norm,{alias}.username,COALESCE({alias}.profile_url,b.profile_url)
             FROM relationships {alias} JOIN relationships b
               ON b.snapshot_id={alias}.snapshot_id AND b.kind='following' AND b.norm={alias}.norm
             WHERE {alias}.snapshot_id={snapshot} AND {alias}.kind='followers' AND {filter}"
        ),
        "not_following_back" => format!(
            "SELECT {alias}.norm,{alias}.username,{alias}.profile_url
             FROM relationships {alias} LEFT JOIN relationships b
               ON b.snapshot_id={alias}.snapshot_id AND b.kind='followers' AND b.norm={alias}.norm
             WHERE {alias}.snapshot_id={snapshot} AND {alias}.kind='following'
               AND b.norm IS NULL AND {filter}"
        ),
        "followers_not_followed_back" => format!(
            "SELECT {alias}.norm,{alias}.username,{alias}.profile_url
             FROM relationships {alias} LEFT JOIN relationships b
               ON b.snapshot_id={alias}.snapshot_id AND b.kind='following' AND b.norm={alias}.norm
             WHERE {alias}.snapshot_id={snapshot} AND {alias}.kind='followers'
               AND b.norm IS NULL AND {filter}"
        ),
        _ => unreachable!("category was validated"),
    }
}
pub fn delete_snapshot(c: &mut Connection, id: i64) -> Result<(), String> {
    let tx = c.transaction().map_err(err)?;
    if tx
        .execute("DELETE FROM snapshots WHERE id=?", [id])
        .map_err(err)?
        == 0
    {
        return Err("Snapshot no longer exists".into());
    }
    crate::fame_store::collect_unreferenced_observations(&tx)?;
    tx.commit().map_err(err)?;
    Ok(())
}
pub fn delete_account(c: &mut Connection, id: i64) -> Result<(), String> {
    let tx = c.transaction().map_err(err)?;
    if tx
        .execute("DELETE FROM accounts WHERE id=?", [id])
        .map_err(err)?
        == 0
    {
        return Err("Account no longer exists".into());
    }
    crate::fame_store::collect_unreferenced_observations(&tx)?;
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
    fn validates_account_and_snapshot_mutation_targets() {
        let mut c = open(Path::new(":memory:")).unwrap();
        let snapshot = commit(&mut c, &parsed(&["a"], &["b"], "1"), None, "me").unwrap();
        assert!(rename_account(&c, snapshot.account_id, "").is_err());
        assert!(rename_account(&c, snapshot.account_id, &"x".repeat(81)).is_err());
        assert_eq!(
            rename_account(&c, snapshot.account_id, " Renamed ")
                .unwrap()
                .label,
            "Renamed"
        );
        assert!(rename_account(&c, 999, "missing").is_err());
        assert!(delete_snapshot(&mut c, 999).is_err());
        assert!(delete_account(&mut c, 999).is_err());
    }

    #[test]
    fn rejects_invalid_query_targets() {
        let mut c = open(Path::new(":memory:")).unwrap();
        let first = commit(&mut c, &parsed(&["a"], &["b"], "1"), None, "one").unwrap();
        let second = commit(&mut c, &parsed(&["c"], &["d"], "2"), None, "two").unwrap();
        assert!(relationships(&c, first.id, "unknown", "").is_err());
        assert!(relationships(&c, 999, "followers", "").is_err());
        assert!(summary(&c, first.account_id, Some(second.id)).is_err());
        assert!(compare(&c, first.id, second.id).is_err());
        assert!(relationships_page(&c, first.id, "followers", &"a".repeat(31), None, 20).is_err());
        assert!(relationships_page(&c, first.id, "followers", "", Some("UPPER"), 20).is_err());
    }

    #[test]
    fn pages_relationships_with_literal_search_and_stable_cursors() {
        let mut c = open(Path::new(":memory:")).unwrap();
        let snapshot = commit(
            &mut c,
            &parsed(&["a_literal", "alice", "carol"], &["alice", "bob"], "1"),
            None,
            "me",
        )
        .unwrap();

        let first = relationships_page(&c, snapshot.id, "followers", "", None, 2).unwrap();
        assert_eq!(
            first
                .items
                .iter()
                .map(|row| row.username.as_str())
                .collect::<Vec<_>>(),
            ["a_literal", "alice"]
        );
        let second = relationships_page(
            &c,
            snapshot.id,
            "followers",
            "",
            first.next_cursor.as_deref(),
            2,
        )
        .unwrap();
        assert_eq!(second.items[0].username, "carol");
        assert!(second.next_cursor.is_none());

        let literal = relationships_page(&c, snapshot.id, "followers", "_", None, 20).unwrap();
        assert_eq!(literal.items.len(), 1);
        assert_eq!(literal.items[0].username, "a_literal");
    }

    #[test]
    fn pages_arbitrary_snapshot_changes() {
        let mut c = open(Path::new(":memory:")).unwrap();
        let first = commit(&mut c, &parsed(&["a"], &["b"], "1"), None, "me").unwrap();
        let second = commit(
            &mut c,
            &parsed(&["a", "c"], &["b"], "2"),
            Some(first.account_id),
            "me",
        )
        .unwrap();
        let third = commit(
            &mut c,
            &parsed(&["c", "d"], &["b"], "3"),
            Some(first.account_id),
            "me",
        )
        .unwrap();

        let page = changes_page(&c, second.id, third.id, "followers", "", None, None, 1).unwrap();
        assert_eq!(page.items.len(), 1);
        assert!(page.next_cursor.is_some());
        let full_range =
            changes_page(&c, first.id, third.id, "followers", "", None, None, 20).unwrap();
        assert_eq!(full_range.items.len(), 3);
        let gained = changes_page(
            &c,
            first.id,
            third.id,
            "followers",
            "",
            None,
            Some("added"),
            20,
        )
        .unwrap();
        assert!(gained.items.iter().all(|item| item.direction == "added"));
    }

    #[test]
    #[ignore = "manual 250k-account capacity validation"]
    fn handles_250k_unique_accounts_in_one_snapshot() {
        let names = (0..250_000)
            .map(|index| format!("user_{index:06}"))
            .collect::<Vec<_>>();
        let follower_names = names[..125_000]
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let following_names = names[125_000..]
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let mut c = open(Path::new(":memory:")).unwrap();
        let snapshot = commit(
            &mut c,
            &parsed(&follower_names, &following_names, "capacity"),
            None,
            "capacity",
        )
        .unwrap();

        let summary = summary(&c, snapshot.account_id, Some(snapshot.id)).unwrap();
        assert_eq!((summary.followers, summary.following), (125_000, 125_000));
        let page = relationships_page(&c, snapshot.id, "following", "", None, 500).unwrap();
        assert_eq!(page.items.len(), 500);
        assert!(page.next_cursor.is_some());
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
            3
        );
        assert_eq!(
            c.pragma_query_value(None, "application_id", |row| row.get::<_, i64>(0))
                .unwrap(),
            APPLICATION_ID
        );
        c.execute("INSERT INTO accounts VALUES(1,'me',NULL,'now')", [])
            .unwrap();
        c.execute(
            "INSERT INTO snapshots(id,account_id,imported_at,observed_at,observed_at_source,source_name,state_hash,followers,following) VALUES(1,1,'2026-01-02T10:00:00Z','2026-01-02','legacy_import_time','test','hash',1,1)",
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
    fn orders_trends_by_observation_date_and_tracks_relationship_history() {
        let mut c = open(Path::new(":memory:")).unwrap();
        let later = commit_at(
            &mut c,
            &parsed(&["alice", "bob"], &["alice"], "later"),
            None,
            "me",
            "2026-03-15",
        )
        .unwrap();
        let earlier = commit_at(
            &mut c,
            &parsed(&["bob"], &["alice"], "earlier"),
            Some(later.account_id),
            "me",
            "2026-02-01",
        )
        .unwrap();
        let points = trends(&c, later.account_id).unwrap();
        assert_eq!(
            points
                .iter()
                .map(|point| point.snapshot_id)
                .collect::<Vec<_>>(),
            [earlier.id, later.id]
        );
        assert_eq!((points[1].new_followers, points[1].lost_followers), (1, 0));
        let history = relationship_history(&c, later.account_id, "alice").unwrap();
        assert_eq!(history.len(), 2);
        assert!(!history[0].follows_you && history[0].you_follow);
        assert!(history[1].follows_you && history[1].you_follow);
        assert!(commit_at(
            &mut c,
            &parsed(&["x"], &["y"], "bad"),
            Some(later.account_id),
            "me",
            "not-a-date"
        )
        .is_err());
    }

    #[cfg(unix)]
    #[test]
    fn creates_database_with_private_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("nivune.db");
        let connection = open(&path).unwrap();
        drop(connection);
        assert_eq!(
            std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    #[test]
    fn migrates_legacy_database_once_without_overwriting_nivune_data() {
        let directory = tempfile::tempdir().unwrap();
        let legacy = directory.path().join("insight.db");
        let destination = directory.path().join("nivune.db");
        let legacy_connection = open(&legacy).unwrap();
        legacy_connection
            .execute(
                "INSERT INTO accounts(label,username,created_at) VALUES('legacy','owner','now')",
                [],
            )
            .unwrap();
        legacy_connection
            .pragma_update(None, "application_id", 0)
            .unwrap();
        legacy_connection
            .pragma_update(None, "journal_mode", "DELETE")
            .unwrap();
        drop(legacy_connection);
        let legacy_before = std::fs::read(&legacy).unwrap();

        assert!(migrate_legacy_database(&legacy, &destination).unwrap());
        assert_eq!(std::fs::read(&legacy).unwrap(), legacy_before);
        let nivune = open(&destination).unwrap();
        let label: String = nivune
            .query_row("SELECT label FROM accounts", [], |row| row.get(0))
            .unwrap();
        assert_eq!(label, "legacy");
        let application_id: i64 = nivune
            .pragma_query_value(None, "application_id", |row| row.get(0))
            .unwrap();
        assert_eq!(application_id, APPLICATION_ID);
        drop(nivune);
        assert!(!migrate_legacy_database(&legacy, &destination).unwrap());
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
    #[ignore = "requires NIVUNE_REAL_EXPORT to point to a private local export"]
    fn imports_real_export_through_sqlite() {
        let source = std::env::var("NIVUNE_REAL_EXPORT")
            .or_else(|_| std::env::var("INSIGHT_REAL_EXPORT"))
            .expect("NIVUNE_REAL_EXPORT is required");
        let mut parsed = parser::parse_path(Path::new(&source)).expect("real export should parse");
        if parsed.detected_username.is_none() {
            parsed.detected_username = Some("e2e_owner".into());
        }
        let expected_followers = parsed.followers.len();
        let expected_following = parsed.following.len();
        let dir = tempfile::tempdir().expect("temporary directory should be available");
        let database = dir.path().join("nivune.db");

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
