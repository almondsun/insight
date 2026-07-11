mod db;
mod model;
mod parser;
use model::*;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
};
use tauri::{Manager, State};
struct AppState {
    db: Mutex<rusqlite::Connection>,
    pending: Mutex<HashMap<String, ParsedImport>>,
    tokens: AtomicU64,
}
#[tauri::command]
fn list_accounts(s: State<AppState>) -> Result<Vec<Account>, String> {
    db::accounts(&s.db.lock().unwrap())
}
#[tauri::command]
fn list_snapshots(account_id: i64, s: State<AppState>) -> Result<Vec<Snapshot>, String> {
    db::snapshots(&s.db.lock().unwrap(), account_id)
}
#[tauri::command]
fn get_summary(
    account_id: i64,
    snapshot_id: Option<i64>,
    s: State<AppState>,
) -> Result<Summary, String> {
    db::summary(&s.db.lock().unwrap(), account_id, snapshot_id)
}
#[tauri::command]
fn get_relationships(
    snapshot_id: i64,
    kind: String,
    search: String,
    s: State<AppState>,
) -> Result<Vec<Relationship>, String> {
    db::relationships(&s.db.lock().unwrap(), snapshot_id, &kind, &search)
}
#[tauri::command]
fn compare_snapshots(
    from_snapshot_id: i64,
    to_snapshot_id: i64,
    s: State<AppState>,
) -> Result<Vec<Change>, String> {
    db::compare(&s.db.lock().unwrap(), from_snapshot_id, to_snapshot_id)
}
#[tauri::command]
fn inspect_import(path: String, s: State<AppState>) -> Result<ImportPreview, String> {
    let parsed = parser::parse_path(&PathBuf::from(path))?;
    let token = format!("{}", s.tokens.fetch_add(1, Ordering::Relaxed));
    let preview = ImportPreview {
        token: token.clone(),
        source_name: parsed.source_name.clone(),
        detected_username: parsed.detected_username.clone(),
        followers: parsed.followers.len(),
        following: parsed.following.len(),
        warnings: parsed.warnings.clone(),
    };
    s.pending.lock().unwrap().insert(token, parsed);
    Ok(preview)
}
#[tauri::command]
fn commit_import(
    token: String,
    account_id: Option<i64>,
    label: String,
    s: State<AppState>,
) -> Result<Snapshot, String> {
    let parsed = s
        .pending
        .lock()
        .unwrap()
        .remove(&token)
        .ok_or("Import preview expired")?;
    db::commit(&mut s.db.lock().unwrap(), &parsed, account_id, &label)
}
#[tauri::command]
fn delete_snapshot(snapshot_id: i64, s: State<AppState>) -> Result<(), String> {
    db::delete_snapshot(&s.db.lock().unwrap(), snapshot_id)
}
#[tauri::command]
fn delete_account(account_id: i64, s: State<AppState>) -> Result<(), String> {
    db::delete_account(&s.db.lock().unwrap(), account_id)
}
#[tauri::command]
fn export_report(
    snapshot_id: i64,
    kind: String,
    format: String,
    path: String,
    s: State<AppState>,
) -> Result<(), String> {
    let rows = db::relationships(&s.db.lock().unwrap(), snapshot_id, &kind, "")?;
    if format == "json" {
        let body = serde_json::json!({"schemaVersion":1,"generatedAt":chrono::Utc::now().to_rfc3339(),"snapshotId":snapshot_id,"category":kind,"relationships":rows});
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&body).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())
    } else if format == "csv" {
        let mut w = csv::Writer::from_path(path).map_err(|e| e.to_string())?;
        w.write_record(["username", "profile_url", "category"])
            .map_err(|e| e.to_string())?;
        for x in rows {
            w.write_record([x.username, x.profile_url.unwrap_or_default(), x.kind])
                .map_err(|e| e.to_string())?;
        }
        w.flush().map_err(|e| e.to_string())
    } else {
        Err("Unsupported export format".into())
    }
}
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let conn = db::open(&dir.join("insight.db")).map_err(std::io::Error::other)?;
            app.manage(AppState {
                db: Mutex::new(conn),
                pending: Mutex::new(HashMap::new()),
                tokens: AtomicU64::new(1),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_accounts,
            list_snapshots,
            get_summary,
            get_relationships,
            compare_snapshots,
            inspect_import,
            commit_import,
            delete_snapshot,
            delete_account,
            export_report
        ])
        .run(tauri::generate_context!())
        .expect("failed to run insIGht")
}
