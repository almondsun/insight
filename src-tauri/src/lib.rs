mod backup;
mod db;
mod export;
mod fame_store;
mod model;
mod parser;
mod source;
pub use fame_core::{agent, corpus, fame, protocol};
use model::*;
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
};
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;
struct AppState {
    db: Mutex<rusqlite::Connection>,
    db_path: PathBuf,
    pending: Mutex<Option<(String, ParsedImport)>>,
    tokens: AtomicU64,
}

#[cfg(unix)]
fn set_private_permissions(path: &Path, mode: u32) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
        .map_err(|error| error.to_string())
}

#[cfg(not(unix))]
fn set_private_permissions(_path: &Path, _mode: u32) -> Result<(), String> {
    Ok(())
}
#[tauri::command]
fn list_accounts(s: State<AppState>) -> Result<Vec<Account>, String> {
    with_db(&s, db::accounts)
}
#[tauri::command]
fn list_snapshots(account_id: i64, s: State<AppState>) -> Result<Vec<Snapshot>, String> {
    with_db(&s, |connection| db::snapshots(connection, account_id))
}
#[tauri::command]
fn get_summary(
    account_id: i64,
    snapshot_id: Option<i64>,
    s: State<AppState>,
) -> Result<Summary, String> {
    with_db(&s, |connection| {
        db::summary(connection, account_id, snapshot_id)
    })
}
#[tauri::command]
fn get_trends(account_id: i64, s: State<AppState>) -> Result<Vec<TrendPoint>, String> {
    with_db(&s, |connection| db::trends(connection, account_id))
}
#[tauri::command]
fn get_relationship_history(
    account_id: i64,
    username: String,
    s: State<AppState>,
) -> Result<Vec<RelationshipHistoryPoint>, String> {
    with_db(&s, |connection| {
        db::relationship_history(connection, account_id, &username)
    })
}
#[tauri::command]
fn get_relationships(
    snapshot_id: i64,
    kind: String,
    search: String,
    after: Option<String>,
    limit: Option<usize>,
    s: State<AppState>,
) -> Result<RelationshipPage, String> {
    with_db(&s, |connection| {
        db::relationships_page(
            connection,
            snapshot_id,
            &kind,
            &search,
            after.as_deref(),
            limit.unwrap_or(200),
        )
    })
}
#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn compare_snapshots(
    from_snapshot_id: i64,
    to_snapshot_id: i64,
    category: String,
    search: String,
    after: Option<String>,
    direction: Option<String>,
    limit: Option<usize>,
    s: State<AppState>,
) -> Result<ChangePage, String> {
    with_db(&s, |connection| {
        db::changes_page(
            connection,
            from_snapshot_id,
            to_snapshot_id,
            &category,
            &search,
            after.as_deref(),
            direction.as_deref(),
            limit.unwrap_or(200),
        )
    })
}
#[tauri::command]
fn rename_account(account_id: i64, label: String, s: State<AppState>) -> Result<Account, String> {
    with_db(&s, |connection| {
        db::rename_account(connection, account_id, &label)
    })
}

#[tauri::command]
fn get_fame_foundation_status() -> FameFoundationStatus {
    FameFoundationStatus {
        implementation_stage: "synthetic_foundation",
        formula_version: fame::FORMULA_VERSION,
        protocol_schema_version: protocol::PROTOCOL_SCHEMA_VERSION,
        fixed_corpus_record_bytes: corpus::FIXED_RECORD_BYTES,
        network_retrieval_available: false,
        architecture_status: "frozen",
        next_stage: "evidence_collection_and_feasibility_validation",
        completed_foundations: vec![
            "versioned fame-v1 scoring",
            "fixed-size committed synthetic corpus records",
            "query-independent scheduler model",
            "frozen threat model and experiment specifications",
        ],
        blocked_gates: vec![
            "licensed query-independent corpus provider",
            "audited two-server PIR deployment with independent operators",
            "qualified mixed request and reply paths",
            "witnessed governance and fresh-time infrastructure",
            "preregistered traffic-analysis thresholds and independent audit",
        ],
    }
}
fn lock_error<T>(_error: std::sync::PoisonError<T>) -> String {
    "Application state lock is unavailable".into()
}

fn with_db<T>(
    state: &AppState,
    operation: impl FnOnce(&rusqlite::Connection) -> Result<T, String>,
) -> Result<T, String> {
    let connection = state.db.lock().map_err(lock_error)?;
    operation(&connection)
}

fn with_db_mut<T>(
    state: &AppState,
    operation: impl FnOnce(&mut rusqlite::Connection) -> Result<T, String>,
) -> Result<T, String> {
    let mut connection = state.db.lock().map_err(lock_error)?;
    operation(&mut connection)
}

fn store_preview(parsed: ParsedImport, s: State<AppState>) -> Result<ImportPreview, String> {
    let token = format!("{}", s.tokens.fetch_add(1, Ordering::Relaxed));
    let preview = ImportPreview {
        token: token.clone(),
        source_name: parsed.source_name.clone(),
        detected_username: parsed.detected_username.clone(),
        followers: parsed.followers.len(),
        following: parsed.following.len(),
        warnings: parsed.warnings.clone(),
    };
    *s.pending.lock().map_err(lock_error)? = Some((token, parsed));
    Ok(preview)
}

#[tauri::command]
async fn choose_import(
    directory: bool,
    app: tauri::AppHandle,
    s: State<'_, AppState>,
) -> Result<Option<ImportPreview>, String> {
    let selected = tauri::async_runtime::spawn_blocking(move || {
        let dialog = app.dialog().file();
        if directory {
            dialog.blocking_pick_folder()
        } else {
            dialog
                .add_filter("Instagram ZIP export", &["zip"])
                .blocking_pick_file()
        }
    })
    .await
    .map_err(|e| e.to_string())?;
    let Some(selected) = selected else {
        return Ok(None);
    };
    let path = selected.into_path().map_err(|e| e.to_string())?;
    let parsed = tauri::async_runtime::spawn_blocking(move || source::parse(&path))
        .await
        .map_err(|e| e.to_string())??;
    Ok(Some(store_preview(parsed, s)?))
}
#[tauri::command]
fn commit_import(
    token: String,
    account_id: Option<i64>,
    label: String,
    owner_username: String,
    observed_at: String,
    s: State<AppState>,
) -> Result<Snapshot, String> {
    let mut parsed = {
        let pending = s.pending.lock().map_err(lock_error)?;
        pending
            .as_ref()
            .filter(|(pending_token, _)| pending_token == &token)
            .map(|(_, parsed)| parsed.clone())
            .ok_or("Import preview expired")?
    };
    let owner_username = owner_username.trim();
    if !parser::is_valid_username(owner_username) {
        return Err("Enter the Instagram username that owns this archive".into());
    }
    if parsed
        .detected_username
        .as_ref()
        .is_some_and(|detected| parser::normalize(detected) != parser::normalize(owner_username))
    {
        return Err("Confirmed owner does not match the archive metadata".into());
    }
    parsed.detected_username = Some(owner_username.to_string());
    let snapshot = with_db_mut(&s, |connection| {
        db::commit_at(connection, &parsed, account_id, &label, &observed_at)
    })?;
    let mut pending = s.pending.lock().map_err(lock_error)?;
    if pending
        .as_ref()
        .is_some_and(|(pending_token, _)| pending_token == &token)
    {
        *pending = None;
    }
    Ok(snapshot)
}

#[tauri::command]
fn cancel_import(token: String, s: State<AppState>) -> Result<(), String> {
    let mut pending = s.pending.lock().map_err(lock_error)?;
    if pending
        .as_ref()
        .is_some_and(|(pending_token, _)| pending_token == &token)
    {
        *pending = None;
    }
    Ok(())
}
#[tauri::command]
fn delete_snapshot(snapshot_id: i64, s: State<AppState>) -> Result<(), String> {
    with_db_mut(&s, |connection| {
        db::delete_snapshot(connection, snapshot_id)
    })
}
#[tauri::command]
fn delete_account(account_id: i64, s: State<AppState>) -> Result<(), String> {
    with_db_mut(&s, |connection| db::delete_account(connection, account_id))
}
#[tauri::command]
async fn export_report(
    snapshot_id: i64,
    kind: String,
    format: String,
    app: tauri::AppHandle,
    s: State<'_, AppState>,
) -> Result<bool, String> {
    if format != "json" && format != "csv" {
        return Err("Unsupported export format".into());
    }
    if !db::valid_relationship_kind(&kind) {
        return Err("Unsupported relationship category".into());
    }
    let db_path = s.db_path.clone();
    let export_format = format.clone();
    let default_name = format!("nivune-{kind}.{format}");
    let selected = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter(export_format.to_uppercase(), &[export_format.as_str()])
            .set_file_name(default_name)
            .blocking_save_file()
    })
    .await
    .map_err(|e| e.to_string())?;
    let Some(selected) = selected else {
        return Ok(false);
    };
    let path = selected.into_path().map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        let connection = db::open(&db_path)?;
        export::relationships(&connection, &path, snapshot_id, &kind, &format)
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(true)
}

#[tauri::command]
async fn export_changes(
    from_snapshot_id: i64,
    to_snapshot_id: i64,
    category: String,
    direction: Option<String>,
    format: String,
    app: tauri::AppHandle,
    s: State<'_, AppState>,
) -> Result<bool, String> {
    if format != "json" && format != "csv" {
        return Err("Unsupported export format".into());
    }
    if !db::valid_relationship_kind(&category) {
        return Err("Unsupported relationship category".into());
    }
    let db_path = s.db_path.clone();
    let extension = format.clone();
    let default_name = format!("nivune-changes-{category}.{format}");
    let selected = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter(extension.to_uppercase(), &[extension.as_str()])
            .set_file_name(default_name)
            .blocking_save_file()
    })
    .await
    .map_err(|e| e.to_string())?;
    let Some(selected) = selected else {
        return Ok(false);
    };
    let path = selected.into_path().map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        let connection = db::open(&db_path)?;
        export::changes(
            &connection,
            &path,
            from_snapshot_id,
            to_snapshot_id,
            &category,
            direction.as_deref(),
            &format,
        )
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(true)
}

fn valid_passphrase(passphrase: &str) -> Result<(), String> {
    let length = passphrase.chars().count();
    if !(12..=1024).contains(&length) {
        return Err("Backup passphrase must contain between 12 and 1024 characters".into());
    }
    Ok(())
}

#[tauri::command]
async fn create_encrypted_backup(
    passphrase: String,
    app: tauri::AppHandle,
    s: State<'_, AppState>,
) -> Result<bool, String> {
    valid_passphrase(&passphrase)?;
    let selected = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter("Encrypted audience backup", &["age"])
            .set_file_name("audience-history.age")
            .blocking_save_file()
    })
    .await
    .map_err(|error| error.to_string())?;
    let Some(selected) = selected else {
        return Ok(false);
    };
    let path = selected.into_path().map_err(|error| error.to_string())?;
    let connection = s.db.lock().map_err(lock_error)?;
    backup::create(&connection, &path, passphrase)?;
    Ok(true)
}

#[tauri::command]
async fn restore_encrypted_backup(
    passphrase: String,
    app: tauri::AppHandle,
    s: State<'_, AppState>,
) -> Result<bool, String> {
    valid_passphrase(&passphrase)?;
    let selected = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter("Encrypted audience backup", &["age"])
            .blocking_pick_file()
    })
    .await
    .map_err(|error| error.to_string())?;
    let Some(selected) = selected else {
        return Ok(false);
    };
    let path = selected.into_path().map_err(|error| error.to_string())?;
    let mut connection = s.db.lock().map_err(lock_error)?;
    backup::restore(&mut connection, &path, passphrase)?;
    Ok(true)
}
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            set_private_permissions(&dir, 0o700).map_err(std::io::Error::other)?;
            let db_path = dir.join("nivune.db");
            if let Some(parent) = dir.parent() {
                let legacy_path = parent.join("app.insight.local").join("insight.db");
                db::migrate_legacy_database(&legacy_path, &db_path)
                    .map_err(std::io::Error::other)?;
            }
            let conn = db::open(&db_path).map_err(std::io::Error::other)?;
            app.manage(AppState {
                db: Mutex::new(conn),
                db_path,
                pending: Mutex::new(None),
                tokens: AtomicU64::new(1),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_accounts,
            list_snapshots,
            get_summary,
            get_trends,
            get_relationship_history,
            get_relationships,
            compare_snapshots,
            rename_account,
            get_fame_foundation_status,
            choose_import,
            commit_import,
            cancel_import,
            delete_snapshot,
            delete_account,
            export_report,
            export_changes,
            create_encrypted_backup,
            restore_encrypted_backup
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Nivune")
}
