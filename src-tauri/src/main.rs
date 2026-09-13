#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod autostart;
mod background;
mod commands;
mod database;
mod notifications;
mod tray;

use database::Database;
use keepr_core::{Snapshot, Store};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, LazyLock, Mutex,
};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

pub struct AppState {
    pub db: Database,
    pub wake: Arc<tokio::sync::Notify>,
    pub exiting: AtomicBool,
    pub pending_route: Mutex<Option<String>>,
    pub window_lock: Mutex<()>,
}

pub fn text(language: &str, key: &str) -> String {
    static RU: LazyLock<serde_json::Value> = LazyLock::new(|| {
        serde_json::from_str(include_str!("../../resources/ru.json")).unwrap_or_default()
    });
    static EN: LazyLock<serde_json::Value> = LazyLock::new(|| {
        serde_json::from_str(include_str!("../../resources/en.json")).unwrap_or_default()
    });
    let map = if language == "en" { &*EN } else { &*RU };
    map.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(key)
        .to_owned()
}

pub fn open_window(app: &AppHandle, route: &str) {
    let app = app.clone();
    let route = route.to_owned();
    // Building a WebView from synchronous Windows event handlers can deadlock.
    std::thread::spawn(move || {
        let state = app.state::<AppState>();
        let Ok(_guard) = state.window_lock.lock() else {
            return;
        };
        if let Ok(mut pending) = state.pending_route.lock() {
            *pending = Some(route.clone());
        }
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
            let _ = window.emit("navigate", route);
        } else if let Err(e) =
            WebviewWindowBuilder::new(&app, "main", WebviewUrl::App("index.html".into()))
                .title("Keepr")
                .inner_size(1180.0, 800.0)
                .min_inner_size(760.0, 560.0)
                .on_navigation(|url| {
                    matches!(url.scheme(), "tauri" | "http" | "https")
                        && matches!(
                            url.host_str(),
                            Some("tauri.localhost" | "127.0.0.1" | "localhost")
                        )
                })
                .build()
        {
            tracing::error!(%e,"could not create main window");
        }
    });
}

pub fn refresh(app: &AppHandle, snapshot: &Snapshot) {
    if let Err(e) = tray::update(app, snapshot) {
        tracing::warn!(%e,"tray update");
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit("changed", ());
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let background = std::env::args().any(|a| a == "--background");
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _| {
            if !args.iter().any(|a| a == "--background") {
                open_window(app, "home");
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::snapshot,
            commands::presets,
            commands::save_item,
            commands::complete_item,
            commands::defer_item,
            commands::reopen_item,
            commands::delete_item,
            commands::undo,
            commands::delete_event,
            commands::save_room,
            commands::delete_room,
            commands::save_settings,
            commands::onboard,
            commands::reset_home,
            commands::initial_route,
            commands::autostart_status,
            commands::set_autostart,
            commands::test_notification,
            commands::export_backup,
            commands::import_backup,
            commands::preview_date
        ])
        .setup(move |app| {
            #[cfg(target_os = "windows")]
            app.handle().plugin(
                tauri_plugin_autostart::Builder::new()
                    .args(["--background"])
                    .build(),
            )?;
            let data = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data)?;
            let log_dir = app.path().app_log_dir()?;
            std::fs::create_dir_all(&log_dir)?;
            let file = tracing_appender::rolling::Builder::new()
                .rotation(tracing_appender::rolling::Rotation::DAILY)
                .filename_prefix("keepr")
                .max_log_files(7)
                .build(log_dir)?;
            let _ = tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "keepr=info".into()),
                )
                .with_writer(file)
                .with_ansi(false)
                .try_init();
            let store = Store::open(data.join("keepr.db"))?;
            let snapshot = store.snapshot(chrono::Utc::now())?;
            app.manage(AppState {
                db: Database::new(store),
                wake: Arc::new(tokio::sync::Notify::new()),
                exiting: AtomicBool::new(false),
                pending_route: Mutex::new(None),
                window_lock: Mutex::new(()),
            });
            if let Err(e) = tray::create(app.handle(), &snapshot) {
                tracing::warn!(%e,"tray unavailable");
            }
            tauri::async_runtime::spawn(background::run(app.handle().clone()));
            #[cfg(target_os = "linux")]
            tauri::async_runtime::spawn(background::watch_resume(app.handle().clone()));
            if !background {
                open_window(app.handle(), "home");
            }
            tracing::info!(background, "Keepr started");
            Ok(())
        })
        .build(tauri::generate_context!())?;
    app.run(|app, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            if !app.state::<AppState>().exiting.load(Ordering::SeqCst) {
                api.prevent_exit();
            }
        }
    });
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Keepr could not start: {error}");
        std::process::exit(1);
    }
}
