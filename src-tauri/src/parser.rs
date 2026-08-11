use crate::model::{ParsedImport, Person};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;
const MAX_FILES: usize = 2_000;
const MAX_ARCHIVE_ENTRIES: usize = 10_000;
const MAX_CENTRAL_DIRECTORY_BYTES: u64 = 16 * 1024 * 1024;
const MAX_ZIP64_END_RECORD_BYTES: u64 = 64 * 1024;
const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 128 * 1024 * 1024;
const MAX_COMPRESSED_FILE_BYTES: u64 = 32 * 1024 * 1024;
const MAX_TOTAL_COMPRESSED_BYTES: u64 = 256 * 1024 * 1024;
const MAX_RELATIONSHIP_RECORDS: usize = 500_000;
pub fn parse_path(path: &Path) -> Result<ParsedImport, String> {
    if !path.exists() {
        return Err("Selected import does not exist".into());
    }
    let name = path
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap_or("Instagram export")
        .to_string();
    let mut files = Vec::new();
    if path.is_dir() {
        let mut total = 0_u64;
        let mut entries = 0_usize;
        for entry in WalkDir::new(path)
            .follow_links(false)
            .follow_root_links(false)
            .max_depth(20)
        {
            let entry = entry.map_err(|e| format!("Unable to read import folder: {e}"))?;
            entries += 1;
            if entries > MAX_ARCHIVE_ENTRIES {
                return Err("Import folder contains too many entries".into());
            }
            if entry.file_type().is_file() && is_import_candidate(entry.path()) {
                if files.len() >= MAX_FILES {
                    return Err("Import contains too many files".into());
                }
                let meta = entry.metadata().map_err(|e| e.to_string())?;
                if meta.len() > MAX_FILE_BYTES {
                    return Err("A JSON file exceeds the 16 MB safety limit".into());
                }
                let file = File::open(entry.path()).map_err(|e| e.to_string())?;
                let data = read_bounded(file)?;
                total = total
                    .checked_add(data.len() as u64)
                    .ok_or("Import size overflow")?;
                if total > MAX_TOTAL_BYTES {
                    return Err("Import JSON data exceeds the 128 MB safety limit".into());
                }
                files.push((entry.path().to_path_buf(), data));
            }
        }
    } else if path
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x.eq_ignore_ascii_case("zip"))
    {
        let mut file = File::open(path).map_err(|e| e.to_string())?;
        preflight_zip(&mut file)?;
        let mut zip = zip::ZipArchive::new(file)
            .map_err(|_| "This is not a valid ZIP archive".to_string())?;
        if zip.len() > MAX_ARCHIVE_ENTRIES {
            return Err("Import archive contains too many entries".into());
        }
        let mut total = 0_u64;
        let mut total_compressed = 0_u64;
        let mut names = HashSet::new();
        for i in 0..zip.len() {
            let mut item = zip.by_index(i).map_err(|e| e.to_string())?;
            let Some(safe) = item.enclosed_name() else {
                return Err("Archive contains an unsafe path".into());
            };
            if !item.is_dir() {
                let canonical = safe.to_string_lossy().replace('\\', "/").to_lowercase();
                if !names.insert(canonical) {
                    return Err("Archive contains duplicate file names".into());
                }
            }
            if !is_import_candidate(&safe) {
                continue;
            }
            if files.len() >= MAX_FILES {
                return Err("Import contains too many relationship files".into());
            }
            validate_zip_candidate_sizes(
                item.size(),
                item.compressed_size(),
                &mut total_compressed,
            )?;
            let data = read_bounded(&mut item)?;
            total = total
                .checked_add(data.len() as u64)
                .ok_or("Import size overflow")?;
            if total > MAX_TOTAL_BYTES {
                return Err("Archive JSON data exceeds the 128 MB safety limit".into());
            }
            files.push((safe, data));
        }
    } else {
        return Err("Choose a complete Instagram JSON export ZIP or folder".into());
    }
    parse_files(name, files)
}

fn preflight_zip(file: &mut File) -> Result<(), String> {
    const EOCD_MIN: u64 = 22;
    const MAX_COMMENT: u64 = u16::MAX as u64;
    let length = file.metadata().map_err(|e| e.to_string())?.len();
    if length < EOCD_MIN {
        return Err("This is not a valid ZIP archive".into());
    }
    let tail_len = length.min(EOCD_MIN + MAX_COMMENT + 20);
    file.seek(SeekFrom::End(-(tail_len as i64)))
        .map_err(|e| e.to_string())?;
    let mut tail = vec![0; tail_len as usize];
    file.read_exact(&mut tail).map_err(|e| e.to_string())?;
    let eocd = (0..=tail.len() - EOCD_MIN as usize)
        .rev()
        .find(|offset| {
            &tail[*offset..*offset + 4] == b"PK\x05\x06"
                && *offset
                    + EOCD_MIN as usize
                    + usize::from(le_u16(&tail[*offset + 20..*offset + 22]))
                    == tail.len()
        })
        .ok_or("This is not a valid ZIP archive")?;
    let eocd_absolute = length - tail_len + eocd as u64;
    let disk = le_u16(&tail[eocd + 4..eocd + 6]);
    let central_disk = le_u16(&tail[eocd + 6..eocd + 8]);
    let entries_on_disk = le_u16(&tail[eocd + 8..eocd + 10]);
    let entries = le_u16(&tail[eocd + 10..eocd + 12]);
    let central_size = le_u32(&tail[eocd + 12..eocd + 16]);
    let central_offset = le_u32(&tail[eocd + 16..eocd + 20]);
    let needs_zip64 = entries == u16::MAX || central_size == u32::MAX || central_offset == u32::MAX;
    let (entries, central_size, central_offset, central_end_boundary, is_single_disk) =
        if needs_zip64 {
            if eocd < 20 || &tail[eocd - 20..eocd - 16] != b"PK\x06\x07" {
                return Err("ZIP64 archive is missing its locator".into());
            }
            let locator_disk = le_u32(&tail[eocd - 16..eocd - 12]);
            let zip64_offset = le_u64(&tail[eocd - 12..eocd - 4]);
            let locator_disks = le_u32(&tail[eocd - 4..eocd]);
            let mut record = [0_u8; 56];
            file.seek(SeekFrom::Start(zip64_offset))
                .map_err(|e| e.to_string())?;
            file.read_exact(&mut record).map_err(|e| e.to_string())?;
            if &record[..4] != b"PK\x06\x06" {
                return Err("ZIP64 archive is missing its end record".into());
            }
            let record_size = le_u64(&record[4..12]);
            let locator_absolute = eocd_absolute
                .checked_sub(20)
                .ok_or("ZIP64 locator is outside the archive")?;
            if !(44..=MAX_ZIP64_END_RECORD_BYTES).contains(&record_size)
                || zip64_offset
                    .checked_add(12)
                    .and_then(|value| value.checked_add(record_size))
                    != Some(locator_absolute)
            {
                return Err("ZIP64 end record is too large or misplaced".into());
            }
            let zip64_disk = le_u32(&record[16..20]);
            let zip64_central_disk = le_u32(&record[20..24]);
            let zip64_entries_on_disk = le_u64(&record[24..32]);
            let zip64_entries = le_u64(&record[32..40]);
            (
                zip64_entries,
                le_u64(&record[40..48]),
                le_u64(&record[48..56]),
                zip64_offset,
                locator_disk == 0
                    && locator_disks == 1
                    && zip64_disk == 0
                    && zip64_central_disk == 0
                    && zip64_entries_on_disk == zip64_entries,
            )
        } else {
            (
                u64::from(entries),
                u64::from(central_size),
                u64::from(central_offset),
                eocd_absolute,
                disk == 0 && central_disk == 0 && entries_on_disk == entries,
            )
        };
    if !is_single_disk {
        return Err("Multi-disk ZIP archives are not supported".into());
    }
    if entries > MAX_ARCHIVE_ENTRIES as u64 {
        return Err("Import archive contains too many entries".into());
    }
    if central_size > MAX_CENTRAL_DIRECTORY_BYTES {
        return Err("Import archive directory is too large".into());
    }
    if central_offset
        .checked_add(central_size)
        .is_none_or(|end| end != central_end_boundary)
    {
        return Err("ZIP central directory layout is unsupported".into());
    }
    file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
    Ok(())
}

fn validate_zip_candidate_sizes(
    uncompressed: u64,
    compressed: u64,
    total_compressed: &mut u64,
) -> Result<(), String> {
    if uncompressed > MAX_FILE_BYTES {
        return Err("A JSON file exceeds the 16 MB safety limit".into());
    }
    if compressed > MAX_COMPRESSED_FILE_BYTES {
        return Err("A compressed JSON entry exceeds the 32 MB safety limit".into());
    }
    *total_compressed = total_compressed
        .checked_add(compressed)
        .ok_or("Compressed import size overflow")?;
    if *total_compressed > MAX_TOTAL_COMPRESSED_BYTES {
        return Err("Archive compressed JSON data exceeds the 256 MB safety limit".into());
    }
    Ok(())
}

fn le_u16(bytes: &[u8]) -> u16 {
    u16::from_le_bytes(bytes.try_into().expect("validated ZIP field width"))
}

fn le_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes.try_into().expect("validated ZIP field width"))
}

fn le_u64(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes.try_into().expect("validated ZIP field width"))
}

fn read_bounded(reader: impl Read) -> Result<Vec<u8>, String> {
    let mut data = Vec::new();
    reader
        .take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() as u64 > MAX_FILE_BYTES {
        return Err("A JSON file exceeds the 16 MB safety limit".into());
    }
    Ok(data)
}

fn is_import_candidate(path: &Path) -> bool {
    let lower = path.to_string_lossy().to_lowercase();
    let filename = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_lowercase();
    (lower.contains("connections/followers_and_following/")
        || lower.contains("connections\\followers_and_following\\"))
        && (filename == "following.json"
            || filename.starts_with("followers_") && filename.ends_with(".json")
            || filename == "followers.json")
        || lower.ends_with("personal_information/personal_information.json")
        || lower.ends_with("personal_information\\personal_information.json")
}
fn parse_files(name: String, files: Vec<(PathBuf, Vec<u8>)>) -> Result<ParsedImport, String> {
    let mut followers = BTreeMap::new();
    let mut following = BTreeMap::new();
    let mut detected = None;
    let mut relevant = 0;
    let mut relationship_records = 0_usize;
    let mut saw_followers = false;
    let mut saw_following = false;
    for (path, data) in files {
        let filename = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_lowercase();
        let target = if filename.starts_with("followers_") || filename == "followers.json" {
            Some(&mut followers)
        } else if filename == "following.json" {
            Some(&mut following)
        } else {
            None
        };
        let value = match serde_json::from_reader::<_, Value>(Cursor::new(data)) {
            Ok(v) => v,
            Err(_) => {
                if target.is_some() {
                    return Err(format!("Malformed JSON in {}", path.display()));
                }
                continue;
            }
        };
        if filename == "personal_information.json" && detected.is_none() {
            if let Some(owner) = find_owner(&value) {
                if !is_valid_username(&owner) {
                    return Err("Invalid owner username in personal information".into());
                }
                detected = Some(owner);
            }
        }
        if let Some(map) = target {
            relevant += 1;
            saw_followers |= filename.starts_with("followers_") || filename == "followers.json";
            saw_following |= filename == "following.json";
            let people = if filename == "following.json" {
                extract_following(&value)?
            } else {
                extract_followers(&value)?
            };
            relationship_records = relationship_records
                .checked_add(people.len())
                .ok_or("Relationship count overflow")?;
            if relationship_records > MAX_RELATIONSHIP_RECORDS {
                return Err("Import contains more than 500,000 relationship records".into());
            }
            for mut p in people {
                if !is_valid_username(&p.username) {
                    return Err(format!("Invalid Instagram username in {}", path.display()));
                }
                p.profile_url = p.profile_url.and_then(|url| canonical_profile_url(&url));
                map.insert(normalize(&p.username), p);
            }
        }
    }
    if relevant == 0 {
        return Err("No follower/following JSON files were found. Request a JSON export from Instagram Accounts Center.".into());
    }
    if !saw_followers || !saw_following {
        return Err("Import must contain both follower and following JSON files".into());
    }
    let warnings = if followers.is_empty() && following.is_empty() {
        vec!["The relationship files are empty".into()]
    } else {
        vec![]
    };
    let mut h = Sha256::new();
    for key in followers.keys() {
        h.update(b"f:");
        h.update(key.as_bytes())
    }
    for key in following.keys() {
        h.update(b"g:");
        h.update(key.as_bytes())
    }
    let hash = format!("{:x}", h.finalize());
    Ok(ParsedImport {
        source_name: name,
        detected_username: detected,
        followers,
        following,
        warnings,
        hash,
    })
}
fn extract_followers(value: &Value) -> Result<Vec<Person>, String> {
    let entries = value
        .as_array()
        .ok_or("Followers JSON has an unsupported schema")?;
    extract_entries(entries)
}

fn extract_following(value: &Value) -> Result<Vec<Person>, String> {
    let entries = value
        .get("relationships_following")
        .and_then(Value::as_array)
        .ok_or("Following JSON has an unsupported schema")?;
    extract_entries(entries)
}

fn extract_entries(entries: &[Value]) -> Result<Vec<Person>, String> {
    let mut people = Vec::new();
    for entry in entries {
        let object = entry
            .as_object()
            .ok_or("Relationship entry has an unsupported schema")?;
        let data = object
            .get("string_list_data")
            .and_then(Value::as_array)
            .ok_or("Relationship entry is missing string_list_data")?;
        for item in data {
            let username = item
                .get("value")
                .and_then(Value::as_str)
                .or_else(|| object.get("title").and_then(Value::as_str))
                .filter(|value| !value.trim().is_empty())
                .ok_or("Relationship entry is missing a username")?;
            people.push(Person {
                username: username.trim().to_string(),
                profile_url: item.get("href").and_then(Value::as_str).map(str::to_string),
                timestamp: item.get("timestamp").and_then(Value::as_i64),
            });
        }
    }
    Ok(people)
}
fn find_owner(v: &Value) -> Option<String> {
    v.get("profile_user")?
        .as_array()?
        .first()?
        .get("string_map_data")?
        .get("Username")?
        .get("value")?
        .as_str()
        .map(str::to_string)
}
pub fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

pub fn is_valid_username(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.len() <= 30
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'_')
}

fn canonical_profile_url(value: &str) -> Option<String> {
    let username = value
        .strip_prefix("https://www.instagram.com/")
        .or_else(|| value.strip_prefix("https://instagram.com/"))?
        .trim_end_matches('/');
    if username.contains('/') || !is_valid_username(username) {
        return None;
    }
    Some(format!("https://www.instagram.com/{username}/"))
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const FOLLOWERS: &[u8] = br#"[{"string_list_data":[{"href":"https://instagram.com/synthetic_alice","value":"synthetic_alice","timestamp":1}]}]"#;
    const FOLLOWING: &[u8] = br#"{"relationships_following":[{"title":"synthetic_bob","string_list_data":[{"href":"https://instagram.com/synthetic_bob","timestamp":2}]}]}"#;
    #[test]
    fn parses_common_shapes() {
        let a=br#"[{"string_list_data":[{"href":"https://instagram.com/Alice","value":"Alice","timestamp":1}]}]"#.to_vec();
        let b = br#"{"relationships_following":[{"title":"bob","string_list_data":[{"href":"https://instagram.com/bob","timestamp":2}]}]}"#.to_vec();
        let got = parse_files(
            "x".into(),
            vec![
                (PathBuf::from("followers_1.json"), a),
                (PathBuf::from("following.json"), b),
            ],
        )
        .unwrap();
        assert!(got.followers.contains_key("alice"));
        assert!(got.following.contains_key("bob"));
    }
    #[test]
    fn rejects_irrelevant() {
        assert!(parse_files(
            "x".into(),
            vec![(PathBuf::from("posts.json"), b"[]".to_vec())]
        )
        .is_err())
    }

    #[test]
    fn rejects_partial_relationship_exports() {
        let followers = br#"[{"string_list_data":[{"value":"alice"}]}]"#.to_vec();
        let result = parse_files(
            "x".into(),
            vec![(PathBuf::from("followers_1.json"), followers)],
        );
        assert!(result.is_err());
    }

    #[test]
    fn sanitizes_profile_urls_and_rejects_invalid_usernames() {
        assert_eq!(
            canonical_profile_url("https://instagram.com/alice/"),
            Some("https://www.instagram.com/alice/".into())
        );
        assert_eq!(canonical_profile_url("javascript:alert(1)"), None);
        assert!(!is_valid_username("=WEBSERVICE(1)"));
    }

    #[test]
    fn limits_reads_to_relationship_and_owner_files() {
        assert!(is_import_candidate(Path::new(
            "connections/followers_and_following/followers_1.json"
        )));
        assert!(is_import_candidate(Path::new(
            "connections/followers_and_following/following.json"
        )));
        assert!(is_import_candidate(Path::new(
            "personal_information/personal_information/personal_information.json"
        )));
        assert!(!is_import_candidate(Path::new(
            "your_instagram_activity/messages/inbox/message_1.json"
        )));
        assert!(!is_import_candidate(Path::new("media/photo.jpg")));
        assert!(!is_import_candidate(Path::new("unrelated/following.json")));
    }

    #[test]
    fn parses_complete_folder_and_zip_end_to_end() {
        let dir = tempfile::tempdir().unwrap();
        let relationships = dir.path().join("connections/followers_and_following");
        std::fs::create_dir_all(&relationships).unwrap();
        std::fs::write(relationships.join("followers_1.json"), FOLLOWERS).unwrap();
        std::fs::write(relationships.join("following.json"), FOLLOWING).unwrap();
        let folder = parse_path(dir.path()).unwrap();
        assert_eq!((folder.followers.len(), folder.following.len()), (1, 1));

        let zip_path = dir.path().join("synthetic-export.zip");
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        archive
            .start_file(
                "connections/followers_and_following/followers_1.json",
                options,
            )
            .unwrap();
        archive.write_all(FOLLOWERS).unwrap();
        archive
            .start_file(
                "connections/followers_and_following/following.json",
                options,
            )
            .unwrap();
        archive.write_all(FOLLOWING).unwrap();
        archive.set_raw_comment(
            b"comment containing a misleading PK\x05\x06 signature"
                .to_vec()
                .into_boxed_slice(),
        );
        archive.finish().unwrap();
        let zipped = parse_path(&zip_path).unwrap();
        assert_eq!(folder.hash, zipped.hash);
    }

    #[test]
    fn rejects_duplicate_archive_paths() {
        let dir = tempfile::tempdir().unwrap();
        let zip_path = dir.path().join("duplicate.zip");
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        for name in ["followers_1.json", "FOLLOWERS_1.JSON"] {
            archive
                .start_file(
                    format!("connections/followers_and_following/{name}"),
                    options,
                )
                .unwrap();
            archive.write_all(FOLLOWERS).unwrap();
        }
        archive
            .start_file(
                "connections/followers_and_following/following.json",
                options,
            )
            .unwrap();
        archive.write_all(FOLLOWING).unwrap();
        archive.finish().unwrap();

        assert!(parse_path(&zip_path)
            .unwrap_err()
            .contains("duplicate file names"));
    }

    #[test]
    fn preflight_rejects_excessive_declared_entry_count() {
        let dir = tempfile::tempdir().unwrap();
        let zip_path = dir.path().join("too-many.zip");
        let file = std::fs::File::create(&zip_path).unwrap();
        let archive = zip::ZipWriter::new(file);
        archive.finish().unwrap();
        let mut bytes = std::fs::read(&zip_path).unwrap();
        let eocd = bytes
            .windows(4)
            .rposition(|window| window == b"PK\x05\x06")
            .unwrap();
        let declared = 10_001_u16.to_le_bytes();
        bytes[eocd + 8..eocd + 10].copy_from_slice(&declared);
        bytes[eocd + 10..eocd + 12].copy_from_slice(&declared);
        std::fs::write(&zip_path, bytes).unwrap();

        assert!(parse_path(&zip_path)
            .unwrap_err()
            .contains("too many entries"));
    }

    #[test]
    fn preflight_rejects_oversized_zip64_end_record() {
        let dir = tempfile::tempdir().unwrap();
        let zip_path = dir.path().join("oversized-zip64.zip");
        let mut bytes = vec![0_u8; 56 + 20 + 22];
        bytes[0..4].copy_from_slice(b"PK\x06\x06");
        bytes[4..12].copy_from_slice(&(MAX_ZIP64_END_RECORD_BYTES + 1).to_le_bytes());
        let locator = 56;
        bytes[locator..locator + 4].copy_from_slice(b"PK\x06\x07");
        bytes[locator + 8..locator + 16].copy_from_slice(&0_u64.to_le_bytes());
        bytes[locator + 16..locator + 20].copy_from_slice(&1_u32.to_le_bytes());
        let eocd = locator + 20;
        bytes[eocd..eocd + 4].copy_from_slice(b"PK\x05\x06");
        bytes[eocd + 8..eocd + 12].copy_from_slice(&[0xff; 4]);
        bytes[eocd + 12..eocd + 20].copy_from_slice(&[0xff; 8]);
        std::fs::write(&zip_path, bytes).unwrap();

        assert!(parse_path(&zip_path)
            .unwrap_err()
            .contains("end record is too large"));
    }

    #[test]
    fn rejects_excessive_compressed_candidate_work() {
        let mut total = 0;
        assert!(
            validate_zip_candidate_sizes(1, MAX_COMPRESSED_FILE_BYTES + 1, &mut total).is_err()
        );
        assert_eq!(total, 0);
    }

    #[test]
    #[ignore = "requires NIVUNE_REAL_EXPORT to point to a private local export"]
    fn parses_sanitized_counts_from_real_export() {
        let path = std::env::var("NIVUNE_REAL_EXPORT")
            .or_else(|_| std::env::var("INSIGHT_REAL_EXPORT"))
            .expect("NIVUNE_REAL_EXPORT is required");
        let parsed = parse_path(Path::new(&path)).expect("real export should parse");
        assert!(!parsed.followers.is_empty());
        assert!(!parsed.following.is_empty());
        eprintln!(
            "parsed {} followers and {} following",
            parsed.followers.len(),
            parsed.following.len()
        );
    }
}
