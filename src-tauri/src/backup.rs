use age::{secrecy::SecretString, Decryptor, Encryptor};
use rusqlite::{backup::Backup, Connection, OpenFlags};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    iter,
    path::Path,
    time::Duration,
};

const MAX_BACKUP_BYTES: u64 = 2 * 1024 * 1024 * 1024;

fn error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn private_temp_in(parent: &Path) -> Result<tempfile::NamedTempFile, String> {
    let file = tempfile::NamedTempFile::new_in(parent).map_err(error)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.as_file()
            .set_permissions(std::fs::Permissions::from_mode(0o600))
            .map_err(error)?;
    }
    Ok(file)
}

fn snapshot(source: &Connection) -> Result<tempfile::NamedTempFile, String> {
    let file = private_temp_in(&std::env::temp_dir())?;
    let mut destination = Connection::open(file.path()).map_err(error)?;
    Backup::new(source, &mut destination)
        .and_then(|backup| backup.run_to_completion(128, Duration::from_millis(2), None))
        .map_err(error)?;
    destination
        .close()
        .map_err(|(_, error)| error.to_string())?;
    Ok(file)
}

pub fn create(source: &Connection, path: &Path, passphrase: String) -> Result<(), String> {
    let plaintext = snapshot(source)?;
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut output = private_temp_in(parent)?;
    let input = File::open(plaintext.path()).map_err(error)?;
    let encryptor = Encryptor::with_user_passphrase(SecretString::from(passphrase));
    let mut writer = encryptor
        .wrap_output(BufWriter::new(output.as_file_mut()))
        .map_err(error)?;
    std::io::copy(&mut BufReader::new(input), &mut writer).map_err(error)?;
    writer.finish().map_err(error)?.flush().map_err(error)?;
    output.as_file_mut().sync_all().map_err(error)?;
    output.persist(path).map_err(error)?;
    Ok(())
}

pub fn restore(
    destination: &mut Connection,
    path: &Path,
    passphrase: String,
) -> Result<(), String> {
    let metadata = std::fs::metadata(path).map_err(error)?;
    if metadata.len() == 0 || metadata.len() > MAX_BACKUP_BYTES {
        return Err("Backup file is empty or exceeds the 2 GB safety limit".into());
    }
    let encrypted = File::open(path).map_err(error)?;
    let decryptor = Decryptor::new(BufReader::new(encrypted))
        .map_err(|_| "Backup is not a valid age file".to_string())?;
    let identity = age::scrypt::Identity::new(SecretString::from(passphrase));
    let mut reader = decryptor
        .decrypt(iter::once(&identity as &dyn age::Identity))
        .map_err(|_| "Backup passphrase is incorrect or the file is damaged".to_string())?;
    let plaintext = private_temp_in(&std::env::temp_dir())?;
    let mut written = 0_u64;
    let mut output = BufWriter::new(plaintext.as_file());
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = reader.read(&mut buffer).map_err(error)?;
        if count == 0 {
            break;
        }
        written = written
            .checked_add(count as u64)
            .ok_or("Backup size overflow")?;
        if written > MAX_BACKUP_BYTES {
            return Err("Decrypted backup exceeds the 2 GB safety limit".into());
        }
        output.write_all(&buffer[..count]).map_err(error)?;
    }
    output.flush().map_err(error)?;
    plaintext.as_file().sync_all().map_err(error)?;
    let source = Connection::open_with_flags(plaintext.path(), OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(error)?;
    let integrity: String = source
        .pragma_query_value(None, "quick_check", |row| row.get(0))
        .map_err(error)?;
    if integrity != "ok" {
        return Err("Backup database failed its integrity check".into());
    }
    let version: i64 = source
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(error)?;
    if !(2..=3).contains(&version) {
        return Err("Backup uses an unsupported database schema".into());
    }
    let application_id: i64 = source
        .pragma_query_value(None, "application_id", |row| row.get(0))
        .map_err(error)?;
    if !crate::db::is_compatible_application_id(application_id) {
        return Err("Backup was not created by this application".into());
    }
    for table in ["accounts", "snapshots", "relationships"] {
        let exists: i64 = source
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?",
                [table],
                |row| row.get(0),
            )
            .map_err(error)?;
        if exists != 1 {
            return Err("Backup is missing required application data".into());
        }
    }
    drop(source);
    let source = crate::db::open(plaintext.path())?;
    Backup::new(&source, destination)
        .and_then(|backup| backup.run_to_completion(128, Duration::from_millis(2), None))
        .map_err(error)?;
    destination
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(error)?;
    destination
        .pragma_update(None, "secure_delete", "ON")
        .map_err(error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypted_backup_round_trip_and_wrong_passphrase() {
        let source = crate::db::open(Path::new(":memory:")).unwrap();
        source
            .execute(
                "INSERT INTO accounts(label,username,created_at) VALUES('test','owner','now')",
                [],
            )
            .unwrap();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("backup.age");
        create(&source, &path, "a sufficiently long passphrase".into()).unwrap();
        let mut restored = crate::db::open(Path::new(":memory:")).unwrap();
        assert!(restore(&mut restored, &path, "incorrect passphrase".into()).is_err());
        restore(
            &mut restored,
            &path,
            "a sufficiently long passphrase".into(),
        )
        .unwrap();
        let label: String = restored
            .query_row("SELECT label FROM accounts", [], |row| row.get(0))
            .unwrap();
        assert_eq!(label, "test");
    }
}
