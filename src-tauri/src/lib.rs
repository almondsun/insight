pub mod agent;
pub mod corpus;
mod db;
pub mod fame;
mod model;
mod parser;
pub mod protocol;
use model::*;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;
struct AppState {
    db: Mutex<rusqlite::Connection>,
    pending: Mutex<Option<(String, ParsedImport)>>,
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
fn get_fame_foundation_status() -> FameFoundationStatus {
    FameFoundationStatus {
        implementation_stage: "synthetic_foundation",
        formula_version: fame::FORMULA_VERSION,
        protocol_schema_version: protocol::PROTOCOL_SCHEMA_VERSION,
        fixed_corpus_record_bytes: corpus::FIXED_RECORD_BYTES,
        network_retrieval_available: false,
    }
}
fn store_preview(parsed: ParsedImport, s: State<AppState>) -> ImportPreview {
    let token = format!("{}", s.tokens.fetch_add(1, Ordering::Relaxed));
    let preview = ImportPreview {
        token: token.clone(),
        source_name: parsed.source_name.clone(),
        detected_username: parsed.detected_username.clone(),
        followers: parsed.followers.len(),
        following: parsed.following.len(),
        warnings: parsed.warnings.clone(),
    };
    *s.pending.lock().unwrap() = Some((token, parsed));
    preview
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
    let parsed = tauri::async_runtime::spawn_blocking(move || parser::parse_path(&path))
        .await
        .map_err(|e| e.to_string())??;
    Ok(Some(store_preview(parsed, s)))
}
#[tauri::command]
fn commit_import(
    token: String,
    account_id: Option<i64>,
    label: String,
    owner_username: String,
    s: State<AppState>,
) -> Result<Snapshot, String> {
    let mut parsed = {
        let pending = s.pending.lock().unwrap();
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
    let snapshot = db::commit(&mut s.db.lock().unwrap(), &parsed, account_id, &label)?;
    let mut pending = s.pending.lock().unwrap();
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
    let mut pending = s.pending.lock().unwrap();
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
    db::delete_snapshot(&mut s.db.lock().unwrap(), snapshot_id)
}
#[tauri::command]
fn delete_account(account_id: i64, s: State<AppState>) -> Result<(), String> {
    db::delete_account(&mut s.db.lock().unwrap(), account_id)
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
    let rows = db::relationships(&s.db.lock().unwrap(), snapshot_id, &kind, "")?;
    let export_format = format.clone();
    let default_name = format!("insight-{kind}.{format}");
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
    if format == "json" {
        let body = serde_json::json!({"schemaVersion":1,"generatedAt":chrono::Utc::now().to_rfc3339(),"snapshotId":snapshot_id,"category":kind,"relationships":rows});
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&body).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    } else {
        let mut w = csv::Writer::from_path(path).map_err(|e| e.to_string())?;
        w.write_record(["username", "profile_url", "category"])
            .map_err(|e| e.to_string())?;
        for x in rows {
            w.write_record([
                csv_safe(x.username),
                csv_safe(x.profile_url.unwrap_or_default()),
                csv_safe(x.kind),
            ])
            .map_err(|e| e.to_string())?;
        }
        w.flush().map_err(|e| e.to_string())?;
    }
    Ok(true)
}

fn csv_safe(value: String) -> String {
    if value
        .as_bytes()
        .first()
        .is_some_and(|byte| matches!(byte, b'=' | b'+' | b'-' | b'@' | b'\t' | b'\r'))
    {
        format!("'{value}")
    } else {
        value
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
                pending: Mutex::new(None),
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
            get_fame_foundation_status,
            choose_import,
            commit_import,
            cancel_import,
            delete_snapshot,
            delete_account,
            export_report
        ])
        .run(tauri::generate_context!())
        .expect("failed to run insIGht")
}
