use crate::{refresh, text, AppState};
use chrono::Utc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

pub async fn run(app: AppHandle) {
    let state = app.state::<AppState>();
    loop {
        let now = Utc::now();
        let work = state
            .db
            .run(move |s| Ok((s.reminders(now)?, s.snapshot(now)?)))
            .await;
        let next = match work {
            Ok(((reminders, next), snapshot)) => {
                refresh(&app, &snapshot);
                if !reminders.is_empty() {
                    let title = if reminders.len() == 1 {
                        reminders[0].name.clone()
                    } else {
                        format!("Keepr · {}", reminders.len())
                    };
                    let body = if reminders.len() == 1 {
                        format!(
                            "{} · {}",
                            text(&snapshot.settings.language, "notification_due"),
                            reminders[0].due_on
                        )
                    } else {
                        format!(
                            "{}\n{}",
                            text(&snapshot.settings.language, "notification_summary"),
                            reminders
                                .iter()
                                .take(4)
                                .map(|r| r.name.as_str())
                                .collect::<Vec<_>>()
                                .join(" · ")
                        )
                    };
                    let sent = match crate::notifications::send(app.clone(), title, body).await {
                        Ok(()) => true,
                        Err(e) => {
                            tracing::warn!(%e,"notification delivery failed");
                            false
                        }
                    };
                    if let Err(e) = state
                        .db
                        .run(move |s| s.record_delivery(&reminders, sent, Utc::now()))
                        .await
                    {
                        tracing::error!(%e,"notification receipt failed");
                        tokio::time::sleep(Duration::from_secs(300)).await;
                    }
                    // Recompute retry time and new receipts before sleeping.
                    continue;
                }
                next
            }
            Err(e) => {
                tracing::error!(%e,"scheduler failed");
                now + chrono::Duration::minutes(5)
            }
        };
        let wall = Utc::now();
        let monotonic = Instant::now();
        loop {
            let duration = (next - Utc::now())
                .to_std()
                .unwrap_or(Duration::ZERO)
                .min(Duration::from_secs(60));
            if duration.is_zero() {
                break;
            }
            tokio::select! {
                _ = state.wake.notified() => break,
                _ = tokio::time::sleep(duration) => {
                    // Cheap clock guard, no database query. Catches clock jumps and
                    // resume where an OS power event isn't available.
                    let wall_elapsed = (Utc::now()-wall).num_seconds();
                    let drift = (wall_elapsed-monotonic.elapsed().as_secs() as i64).abs();
                    if drift > 2 || Utc::now() >= next {break;}
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub async fn watch_resume(app: AppHandle) {
    use futures_util::StreamExt;
    let result: Result<(), zbus::Error> = async {
        let conn = zbus::Connection::system().await?;
        let proxy = zbus::Proxy::new(
            &conn,
            "org.freedesktop.login1",
            "/org/freedesktop/login1",
            "org.freedesktop.login1.Manager",
        )
        .await?;
        let mut signals = proxy.receive_signal("PrepareForSleep").await?;
        while let Some(message) = signals.next().await {
            if matches!(message.body().deserialize::<bool>(), Ok(false)) {
                app.state::<AppState>().wake.notify_one();
            }
        }
        Ok(())
    }
    .await;
    if let Err(e) = result {
        tracing::debug!(%e,"resume events unavailable; clock guard active");
    }
}
