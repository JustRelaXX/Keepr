use crate::{refresh, AppState};
use chrono::Utc;
use keepr_core::{ItemInput, Mutation, Preset, Room, Settings, Snapshot, Store};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

async fn publish(app: &AppHandle, result: Result<Mutation, String>) -> Result<Mutation, String> {
    let result = result?;
    refresh(app, &result.snapshot);
    app.state::<AppState>().wake.notify_one();
    Ok(result)
}

#[tauri::command]
pub async fn snapshot(state: State<'_, AppState>) -> Result<Snapshot, String> {
    state.db.run(|s| s.snapshot(Utc::now())).await
}

#[tauri::command]
pub fn presets() -> Result<Vec<Preset>, String> {
    Store::presets().map_err(|e| e.code().into())
}

#[tauri::command]
pub async fn save_item(
    app: AppHandle,
    state: State<'_, AppState>,
    id: Option<String>,
    revision: Option<u32>,
    input: ItemInput,
) -> Result<Mutation, String> {
    publish(
        &app,
        state
            .db
            .run(move |s| s.save_item(id, revision, input, Utc::now()))
            .await,
    )
    .await
}

#[tauri::command]
pub async fn complete_item(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    revision: u32,
    completed_on: String,
) -> Result<Mutation, String> {
    publish(
        &app,
        state
            .db
            .run(move |s| s.complete(&id, revision, &completed_on, Utc::now()))
            .await,
    )
    .await
}

#[tauri::command]
pub async fn defer_item(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    revision: u32,
    choice: String,
) -> Result<Mutation, String> {
    publish(
        &app,
        state
            .db
            .run(move |s| s.defer(&id, revision, &choice, Utc::now()))
            .await,
    )
    .await
}

#[tauri::command]
pub async fn delete_item(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    revision: u32,
) -> Result<Mutation, String> {
    publish(
        &app,
        state
            .db
            .run(move |s| s.delete_item(&id, revision, Utc::now()))
            .await,
    )
    .await
}

#[tauri::command]
pub async fn reopen_item(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    revision: u32,
    due_on: String,
) -> Result<Mutation, String> {
    publish(
        &app,
        state
            .db
            .run(move |s| s.reopen(&id, revision, &due_on, Utc::now()))
            .await,
    )
    .await
}

#[tauri::command]
pub async fn undo(
    app: AppHandle,
    state: State<'_, AppState>,
    event_id: String,
) -> Result<Mutation, String> {
    publish(
        &app,
        state.db.run(move |s| s.undo(&event_id, Utc::now())).await,
    )
    .await
}

#[tauri::command]
pub async fn delete_event(
    app: AppHandle,
    state: State<'_, AppState>,
    event_id: String,
) -> Result<Mutation, String> {
    publish(
        &app,
        state
            .db
            .run(move |s| s.delete_event(&event_id, Utc::now()))
            .await,
    )
    .await
}

#[tauri::command]
pub async fn save_room(
    app: AppHandle,
    state: State<'_, AppState>,
    room: Room,
) -> Result<Mutation, String> {
    publish(
        &app,
        state.db.run(move |s| s.save_room(room, Utc::now())).await,
    )
    .await
}

#[tauri::command]
pub async fn delete_room(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<Mutation, String> {
    publish(
        &app,
        state.db.run(move |s| s.delete_room(&id, Utc::now())).await,
    )
    .await
}

#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Mutation, String> {
    publish(
        &app,
        state
            .db
            .run(move |s| s.save_settings(settings, Utc::now()))
            .await,
    )
    .await
}

#[tauri::command]
pub async fn onboard(
    app: AppHandle,
    state: State<'_, AppState>,
    demo: bool,
) -> Result<Mutation, String> {
    publish(
        &app,
        state.db.run(move |s| s.onboard(demo, Utc::now())).await,
    )
    .await
}

#[tauri::command]
pub async fn reset_home(app: AppHandle, state: State<'_, AppState>) -> Result<Mutation, String> {
    publish(&app, state.db.run(move |s| s.reset_home(Utc::now())).await).await
}

#[tauri::command]
pub fn initial_route(state: State<'_, AppState>) -> Option<String> {
    state.pending_route.lock().ok()?.take()
}

#[tauri::command]
pub fn autostart_status(app: AppHandle) -> Result<bool, String> {
    crate::autostart::enabled(&app).map_err(|e| {
        tracing::error!(%e,"autostart status");
        "system_error".into()
    })
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    crate::autostart::set(&app, enabled).map_err(|e| {
        tracing::error!(%e,"autostart change");
        "system_error".into()
    })
}

#[tauri::command]
pub async fn test_notification(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let s = state.db.run(|s| s.settings()).await?;
    crate::notifications::send(
        app,
        "Keepr".into(),
        crate::text(&s.language, "notification_test"),
    )
    .await
}

#[tauri::command]
pub async fn export_backup(app: AppHandle, state: State<'_, AppState>) -> Result<bool, String> {
    let app_clone = app.clone();
    let path = tauri::async_runtime::spawn_blocking(move || {
        app_clone
            .dialog()
            .file()
            .add_filter("Keepr", &["keepr"])
            .set_file_name(format!("Keepr-{}.keepr", Utc::now().format("%Y-%m-%d")))
            .blocking_save_file()
    })
    .await
    .map_err(|_| "system_error")?;
    let Some(path) = path else {
        return Ok(false);
    };
    let path = path.into_path().map_err(|_| "system_error")?;
    let live = app
        .path()
        .app_data_dir()
        .map_err(|_| "system_error")?
        .join("keepr.db");
    if path == live {
        return Err("invalid_backup".into());
    }
    state.db.run(move |s| s.backup(&path)).await?;
    Ok(true)
}

#[tauri::command]
pub async fn import_backup(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<Mutation>, String> {
    let app_clone = app.clone();
    let path = tauri::async_runtime::spawn_blocking(move || {
        app_clone
            .dialog()
            .file()
            .add_filter("Keepr", &["keepr"])
            .blocking_pick_file()
    })
    .await
    .map_err(|_| "system_error")?;
    let Some(path) = path else {
        return Ok(None);
    };
    let path = path.into_path().map_err(|_| "system_error")?;
    let safety = app
        .path()
        .app_data_dir()
        .map_err(|_| "system_error")?
        .join("before-restore.keepr");
    let result = state
        .db
        .run(move |s| {
            s.backup(&safety)?;
            s.restore(&path)?;
            Ok(Mutation {
                snapshot: s.snapshot(Utc::now())?,
                undo_id: None,
                item_id: None,
            })
        })
        .await;
    Ok(Some(publish(&app, result).await?))
}

#[tauri::command]
pub async fn preview_date(
    state: State<'_, AppState>,
    started_on: String,
    repeat: String,
    every: u32,
) -> Result<String, String> {
    state
        .db
        .run(move |_| {
            Ok(
                keepr_core::dates::next_date(
                    keepr_core::dates::date(&started_on)?,
                    &repeat,
                    every,
                )?
                .to_string(),
            )
        })
        .await
}
