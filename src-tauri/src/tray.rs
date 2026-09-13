use crate::{open_window, refresh, text, AppState};
use keepr_core::Snapshot;
use std::sync::atomic::Ordering;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

fn menu(app: &AppHandle, snapshot: &Snapshot) -> tauri::Result<Menu<tauri::Wry>> {
    let lang = &snapshot.settings.language;
    let menu = Menu::new(app)?;
    let count = snapshot.health.overdue + snapshot.health.today;
    menu.append(&MenuItem::with_id(
        app,
        "home",
        format!("Keepr · {count}"),
        true,
        None::<&str>,
    )?)?;
    for key in ["today", "overdue", "add"] {
        menu.append(&MenuItem::with_id(
            app,
            key,
            text(lang, key),
            true,
            None::<&str>,
        )?)?;
    }
    let urgent: Vec<_> = snapshot
        .items
        .iter()
        .filter(|v| !v.item.completed && v.days_left <= 0)
        .take(5)
        .collect();
    if !urgent.is_empty() {
        menu.append(&PredefinedMenuItem::separator(app)?)?;
    }
    for view in urgent {
        let item = &view.item;
        let sub = Submenu::new(app, &item.input.name, true)?;
        for action in ["done", "hour", "evening", "tomorrow"] {
            sub.append(&MenuItem::with_id(
                app,
                format!("action|{}|{}|{}", item.id, item.revision, action),
                text(lang, action),
                true,
                None::<&str>,
            )?)?;
        }
        menu.append(&sub)?;
    }
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(
        app,
        "settings",
        text(lang, "settings"),
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        "quit",
        text(lang, "quit"),
        true,
        None::<&str>,
    )?)?;
    Ok(menu)
}

pub fn create(app: &AppHandle, snapshot: &Snapshot) -> tauri::Result<()> {
    let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))?;
    TrayIconBuilder::with_id("keepr")
        .icon(icon)
        .menu(&menu(app, snapshot)?)
        .tooltip("Keepr")
        .on_menu_event(|app, event| {
            let value = event.id().as_ref();
            if value == "quit" {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    // The barrier drains previously queued database transactions.
                    let _ = app.state::<AppState>().db.run(|_| Ok(())).await;
                    app.state::<AppState>()
                        .exiting
                        .store(true, Ordering::SeqCst);
                    app.exit(0);
                });
            } else if value.starts_with("action|") {
                let parts: Vec<_> = value.split('|').map(str::to_owned).collect();
                if parts.len() != 4 {
                    return;
                }
                let Ok(revision) = parts[2].parse::<u32>() else {
                    return;
                };
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let state = app.state::<AppState>();
                    let result = state
                        .db
                        .run(move |store| {
                            let now = chrono::Utc::now();
                            if parts[3] == "done" {
                                let today = now
                                    .with_timezone(&keepr_core::dates::zone(&store.settings()?)?)
                                    .date_naive()
                                    .to_string();
                                store.complete(&parts[1], revision, &today, now)
                            } else {
                                store.defer(&parts[1], revision, &parts[3], now)
                            }
                        })
                        .await;
                    match result {
                        Ok(m) => {
                            refresh(&app, &m.snapshot);
                            state.wake.notify_one();
                        }
                        Err(e) => {
                            tracing::warn!(%e,"tray action failed");
                            open_window(&app, "home");
                        }
                    }
                });
            } else {
                open_window(app, value);
            }
        })
        .build(app)?;
    Ok(())
}

pub fn update(app: &AppHandle, snapshot: &Snapshot) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id("keepr") {
        let count = snapshot.health.overdue + snapshot.health.today;
        tray.set_tooltip(Some(format!("Keepr · {count}")))?;
        tray.set_title(if count > 0 {
            Some(count.to_string())
        } else {
            None
        })?;
        tray.set_menu(Some(menu(app, snapshot)?))?;
    }
    Ok(())
}
