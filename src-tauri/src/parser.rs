use crate::model::{ParsedImport, Person};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Cursor, Read},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;
const MAX_FILES: usize = 2_000;
const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 128 * 1024 * 1024;
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
        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_type().is_file() && is_import_candidate(entry.path()) {
                if files.len() >= MAX_FILES {
                    return Err("Import contains too many files".into());
                }
                let meta = entry.metadata().map_err(|e| e.to_string())?;
                if meta.len() > MAX_FILE_BYTES {
                    return Err("A JSON file exceeds the 16 MB safety limit".into());
                }
                files.push((
                    entry.path().to_path_buf(),
                    std::fs::read(entry.path()).map_err(|e| e.to_string())?,
                ));
            }
        }
    } else if path
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x.eq_ignore_ascii_case("zip"))
    {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipArchive::new(file)
            .map_err(|_| "This is not a valid ZIP archive".to_string())?;
        let mut total = 0;
        for i in 0..zip.len() {
            let mut item = zip.by_index(i).map_err(|e| e.to_string())?;
            let Some(safe) = item.enclosed_name() else {
                return Err("Archive contains an unsafe path".into());
            };
            if !is_import_candidate(&safe) {
                continue;
            }
            if files.len() >= MAX_FILES {
                return Err("Import contains too many relationship files".into());
            }
            if item.size() > MAX_FILE_BYTES {
                return Err("A JSON file exceeds the 16 MB safety limit".into());
            }
            total += item.size();
            if total > MAX_TOTAL_BYTES {
                return Err("Archive JSON data exceeds the 128 MB safety limit".into());
            }
            let mut data = Vec::new();
            item.read_to_end(&mut data).map_err(|e| e.to_string())?;
            files.push((safe, data));
        }
    } else if path
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x.eq_ignore_ascii_case("json"))
    {
        files.push((
            path.to_path_buf(),
            std::fs::read(path).map_err(|e| e.to_string())?,
        ));
    } else {
        return Err("Choose an Instagram JSON export ZIP, folder, or JSON file".into());
    }
    parse_files(name, files)
}

fn is_import_candidate(path: &Path) -> bool {
    let lower = path.to_string_lossy().to_lowercase();
    let filename = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_lowercase();
    filename == "following.json"
        || filename.starts_with("followers_") && filename.ends_with(".json")
        || filename == "followers.json"
        || lower.ends_with("personal_information/personal_information.json")
}
fn parse_files(name: String, files: Vec<(PathBuf, Vec<u8>)>) -> Result<ParsedImport, String> {
    let mut followers = BTreeMap::new();
    let mut following = BTreeMap::new();
    let mut detected = None;
    let mut relevant = 0;
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
        if detected.is_none() {
            detected = find_owner(&value)
        }
        if let Some(map) = target {
            relevant += 1;
            for p in extract_people(&value) {
                map.insert(normalize(&p.username), p);
            }
        }
    }
    if relevant == 0 {
        return Err("No follower/following JSON files were found. Request a JSON export from Instagram Accounts Center.".into());
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
fn extract_people(value: &Value) -> Vec<Person> {
    let mut out = Vec::new();
    visit(value, &mut out);
    out
}
fn visit(v: &Value, out: &mut Vec<Person>) {
    match v {
        Value::Object(obj) => {
            if let Some(arr) = obj.get("string_list_data").and_then(Value::as_array) {
                for x in arr {
                    if let Some(username) = x
                        .get("value")
                        .and_then(Value::as_str)
                        .or_else(|| obj.get("title").and_then(Value::as_str))
                        .filter(|x| !x.trim().is_empty())
                    {
                        out.push(Person {
                            username: username.trim().to_string(),
                            profile_url: x.get("href").and_then(Value::as_str).map(str::to_string),
                            timestamp: x.get("timestamp").and_then(Value::as_i64),
                        })
                    }
                }
            } else {
                for x in obj.values() {
                    visit(x, out)
                }
            }
        }
        Value::Array(a) => {
            for x in a {
                visit(x, out)
            }
        }
        _ => {}
    }
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
#[cfg(test)]
mod tests {
    use super::*;
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
    }

    #[test]
    #[ignore = "requires INSIGHT_REAL_EXPORT to point to a private local export"]
    fn parses_sanitized_counts_from_real_export() {
        let path = std::env::var("INSIGHT_REAL_EXPORT").expect("INSIGHT_REAL_EXPORT is required");
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
