use tauri::{AppHandle, Manager};

/// Await the actual OS transport before acknowledging delivery. The Tauri
/// notification plugin currently detaches its send task and discards errors;
/// direct notify-rust also avoids its nested-runtime issue on Linux.
pub async fn send(app: AppHandle, title: String, body: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let send = || -> Result<(), Box<dyn std::error::Error>> {
            let icon = app.path().app_data_dir()?.join("notification-icon.png");
            if !icon.exists() {
                std::fs::write(&icon, include_bytes!("../icons/128x128.png"))?;
            }
            let mut notification = notify_rust::Notification::new();
            notification
                .appname("Keepr")
                .summary(&title)
                .body(
                    &body
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;"),
                )
                .icon(&icon.to_string_lossy());
            #[cfg(target_os = "windows")]
            notification.app_id(&app.config().identifier);
            notification.show()?;
            Ok(())
        };
        send().map_err(|error| {
            tracing::warn!(%error, "native notification failed");
            "notification_error".to_owned()
        })
    })
    .await
    .map_err(|error| {
        tracing::error!(%error, "notification worker failed");
        "notification_error".to_owned()
    })?
}
