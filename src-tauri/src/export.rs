use crate::{db, model::Change, model::Relationship};
use rusqlite::Connection;
use serde::Serialize;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

const EXPORT_PAGE_SIZE: usize = 500;

fn write_atomically(
    path: &Path,
    operation: impl FnOnce(&mut File) -> Result<(), String>,
) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        temporary
            .as_file()
            .set_permissions(std::fs::Permissions::from_mode(0o600))
            .map_err(|error| error.to_string())?;
    }
    operation(temporary.as_file_mut())?;
    temporary
        .as_file_mut()
        .sync_all()
        .map_err(|error| error.to_string())?;
    temporary.persist(path).map_err(|error| error.to_string())?;
    Ok(())
}

fn json_value(writer: &mut impl Write, value: &impl Serialize) -> Result<(), String> {
    serde_json::to_writer(writer, value).map_err(|error| error.to_string())
}

pub fn relationships(
    connection: &Connection,
    path: &Path,
    snapshot_id: i64,
    kind: &str,
    format: &str,
) -> Result<(), String> {
    if !matches!(format, "json" | "csv") {
        return Err("Unsupported export format".into());
    }
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    db::relationships_page(&transaction, snapshot_id, kind, "", None, 1)?;
    let result = match format {
        "json" => write_atomically(path, |file| {
            relationship_json(&transaction, file, snapshot_id, kind)
        }),
        "csv" => write_atomically(path, |file| {
            relationship_csv(&transaction, file, snapshot_id, kind)
        }),
        _ => Err("Unsupported export format".into()),
    };
    result?;
    transaction.commit().map_err(|error| error.to_string())
}

fn relationship_json(
    connection: &Connection,
    file: &mut File,
    snapshot_id: i64,
    kind: &str,
) -> Result<(), String> {
    let mut writer = BufWriter::new(file);
    write!(writer, "{{\"schemaVersion\":1,\"generatedAt\":").map_err(|error| error.to_string())?;
    json_value(&mut writer, &chrono::Utc::now().to_rfc3339())?;
    write!(writer, ",\"snapshotId\":{snapshot_id},\"category\":")
        .map_err(|error| error.to_string())?;
    json_value(&mut writer, &kind)?;
    writer
        .write_all(b",\"relationships\":[")
        .map_err(|error| error.to_string())?;
    stream_relationships(connection, snapshot_id, kind, |row, first| {
        if !first {
            writer.write_all(b",").map_err(|error| error.to_string())?;
        }
        json_value(&mut writer, row)
    })?;
    writer
        .write_all(b"]}\n")
        .and_then(|_| writer.flush())
        .map_err(|error| error.to_string())
}

fn relationship_csv(
    connection: &Connection,
    file: &mut File,
    snapshot_id: i64,
    kind: &str,
) -> Result<(), String> {
    let mut writer = csv::Writer::from_writer(file);
    writer
        .write_record(["username", "profile_url", "category"])
        .map_err(|error| error.to_string())?;
    stream_relationships(connection, snapshot_id, kind, |row, _| {
        writer
            .write_record([
                csv_safe(&row.username),
                csv_safe(row.profile_url.as_deref().unwrap_or_default()),
                csv_safe(&row.kind),
            ])
            .map_err(|error| error.to_string())
    })?;
    writer.flush().map_err(|error| error.to_string())
}

fn stream_relationships(
    connection: &Connection,
    snapshot_id: i64,
    kind: &str,
    mut consume: impl FnMut(&Relationship, bool) -> Result<(), String>,
) -> Result<(), String> {
    let mut cursor = None;
    let mut first = true;
    loop {
        let page = db::relationships_page(
            connection,
            snapshot_id,
            kind,
            "",
            cursor.as_deref(),
            EXPORT_PAGE_SIZE,
        )?;
        for row in &page.items {
            consume(row, first)?;
            first = false;
        }
        let Some(next) = page.next_cursor else {
            return Ok(());
        };
        cursor = Some(next);
    }
}

pub fn changes(
    connection: &Connection,
    path: &Path,
    from_snapshot_id: i64,
    to_snapshot_id: i64,
    category: &str,
    format: &str,
) -> Result<(), String> {
    if !matches!(format, "json" | "csv") {
        return Err("Unsupported export format".into());
    }
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    db::changes_page(
        &transaction,
        from_snapshot_id,
        to_snapshot_id,
        category,
        "",
        None,
        1,
    )?;
    let result = match format {
        "json" => write_atomically(path, |file| {
            change_json(
                &transaction,
                file,
                from_snapshot_id,
                to_snapshot_id,
                category,
            )
        }),
        "csv" => write_atomically(path, |file| {
            change_csv(
                &transaction,
                file,
                from_snapshot_id,
                to_snapshot_id,
                category,
            )
        }),
        _ => Err("Unsupported export format".into()),
    };
    result?;
    transaction.commit().map_err(|error| error.to_string())
}

fn change_json(
    connection: &Connection,
    file: &mut File,
    from_snapshot_id: i64,
    to_snapshot_id: i64,
    category: &str,
) -> Result<(), String> {
    let mut writer = BufWriter::new(file);
    write!(writer, "{{\"schemaVersion\":1,\"generatedAt\":").map_err(|error| error.to_string())?;
    json_value(&mut writer, &chrono::Utc::now().to_rfc3339())?;
    write!(
        writer,
        ",\"fromSnapshotId\":{from_snapshot_id},\"toSnapshotId\":{to_snapshot_id},\"category\":"
    )
    .map_err(|error| error.to_string())?;
    json_value(&mut writer, &category)?;
    writer
        .write_all(b",\"changes\":[")
        .map_err(|error| error.to_string())?;
    stream_changes(
        connection,
        from_snapshot_id,
        to_snapshot_id,
        category,
        |row, first| {
            if !first {
                writer.write_all(b",").map_err(|error| error.to_string())?;
            }
            json_value(&mut writer, row)
        },
    )?;
    writer
        .write_all(b"]}\n")
        .and_then(|_| writer.flush())
        .map_err(|error| error.to_string())
}

fn change_csv(
    connection: &Connection,
    file: &mut File,
    from_snapshot_id: i64,
    to_snapshot_id: i64,
    category: &str,
) -> Result<(), String> {
    let mut writer = csv::Writer::from_writer(file);
    writer
        .write_record(["username", "profile_url", "category", "direction"])
        .map_err(|error| error.to_string())?;
    stream_changes(
        connection,
        from_snapshot_id,
        to_snapshot_id,
        category,
        |row, _| {
            writer
                .write_record([
                    csv_safe(&row.username),
                    csv_safe(row.profile_url.as_deref().unwrap_or_default()),
                    csv_safe(&row.category),
                    csv_safe(&row.direction),
                ])
                .map_err(|error| error.to_string())
        },
    )?;
    writer.flush().map_err(|error| error.to_string())
}

fn stream_changes(
    connection: &Connection,
    from_snapshot_id: i64,
    to_snapshot_id: i64,
    category: &str,
    mut consume: impl FnMut(&Change, bool) -> Result<(), String>,
) -> Result<(), String> {
    let mut cursor = None;
    let mut first = true;
    loop {
        let page = db::changes_page(
            connection,
            from_snapshot_id,
            to_snapshot_id,
            category,
            "",
            cursor.as_deref(),
            EXPORT_PAGE_SIZE,
        )?;
        for row in &page.items {
            consume(row, first)?;
            first = false;
        }
        let Some(next) = page.next_cursor else {
            return Ok(());
        };
        cursor = Some(next);
    }
}

fn csv_safe(value: &str) -> String {
    if value
        .as_bytes()
        .first()
        .is_some_and(|byte| matches!(byte, b'=' | b'+' | b'-' | b'@' | b'\t' | b'\r'))
    {
        format!("'{value}")
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{csv_safe, relationships, write_atomically};
    use crate::db;
    use rusqlite::params;
    use std::{io::Write, path::Path};

    #[test]
    fn neutralizes_spreadsheet_formulas() {
        for prefix in ['=', '+', '-', '@', '\t', '\r'] {
            let value = format!("{prefix}danger");
            assert_eq!(csv_safe(&value), format!("'{value}"));
        }
        assert_eq!(csv_safe("safe_user"), "safe_user");
    }

    #[test]
    fn streams_relationship_json_across_multiple_pages() {
        let connection = db::open(Path::new(":memory:")).unwrap();
        connection
            .execute("INSERT INTO accounts VALUES(1,'me','me','now')", [])
            .unwrap();
        connection
            .execute(
                "INSERT INTO snapshots VALUES(1,1,'now','test','hash',1201,0)",
                [],
            )
            .unwrap();
        let mut statement = connection
            .prepare(
                "INSERT INTO relationships(snapshot_id,kind,norm,username) VALUES(1,'followers',?,?)",
            )
            .unwrap();
        for index in 0..1_201 {
            let username = format!("user_{index:04}");
            statement.execute(params![username, username]).unwrap();
        }
        drop(statement);
        let destination = tempfile::NamedTempFile::new().unwrap();

        relationships(&connection, destination.path(), 1, "followers", "json").unwrap();

        let value: serde_json::Value =
            serde_json::from_reader(std::fs::File::open(destination.path()).unwrap()).unwrap();
        assert_eq!(value["schemaVersion"], 1);
        assert_eq!(value["relationships"].as_array().unwrap().len(), 1_201);
    }

    #[test]
    fn failed_export_preserves_existing_destination() {
        let connection = db::open(Path::new(":memory:")).unwrap();
        let destination = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(destination.path(), b"existing report").unwrap();

        assert!(relationships(&connection, destination.path(), 999, "followers", "json").is_err());
        assert_eq!(
            std::fs::read(destination.path()).unwrap(),
            b"existing report"
        );
    }

    #[test]
    fn midstream_failure_preserves_existing_destination() {
        let destination = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(destination.path(), b"existing report").unwrap();

        let result = write_atomically(destination.path(), |temporary| {
            temporary.write_all(b"partial report").unwrap();
            Err("simulated write failure".into())
        });

        assert!(result.is_err());
        assert_eq!(
            std::fs::read(destination.path()).unwrap(),
            b"existing report"
        );
    }

    #[test]
    fn read_transaction_keeps_pages_consistent_during_deletion() {
        let database = tempfile::NamedTempFile::new().unwrap();
        let writer = db::open(database.path()).unwrap();
        writer
            .execute("INSERT INTO accounts VALUES(1,'me','me','now')", [])
            .unwrap();
        writer
            .execute(
                "INSERT INTO snapshots VALUES(1,1,'now','test','hash',600,0)",
                [],
            )
            .unwrap();
        let mut statement = writer
            .prepare(
                "INSERT INTO relationships(snapshot_id,kind,norm,username) VALUES(1,'followers',?,?)",
            )
            .unwrap();
        for index in 0..600 {
            let username = format!("user_{index:04}");
            statement.execute(params![username, username]).unwrap();
        }
        drop(statement);
        let reader = db::open(database.path()).unwrap();
        let transaction = reader.unchecked_transaction().unwrap();
        let first = db::relationships_page(&transaction, 1, "followers", "", None, 500).unwrap();

        writer
            .execute("DELETE FROM snapshots WHERE id=1", [])
            .unwrap();
        let second = db::relationships_page(
            &transaction,
            1,
            "followers",
            "",
            first.next_cursor.as_deref(),
            500,
        )
        .unwrap();

        assert_eq!(second.items.len(), 100);
    }
}
