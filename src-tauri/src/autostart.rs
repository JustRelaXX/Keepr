use tauri::AppHandle;

#[cfg(target_os = "linux")]
fn entry_path(app: &AppHandle) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    use tauri::Manager;
    Ok(app
        .path()
        .config_dir()?
        .join("autostart")
        .join("keepr.desktop"))
}

#[cfg(target_os = "linux")]
pub fn enabled(app: &AppHandle) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(entry_path(app)?.is_file())
}

#[cfg(target_os = "linux")]
pub fn set(app: &AppHandle, enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::Manager;
    let path = entry_path(app)?;
    if !enabled {
        match std::fs::remove_file(path) {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e.into()),
        }
    }
    let executable = app
        .env()
        .appimage
        .unwrap_or(std::env::current_exe()?.into_os_string());
    let executable = executable.to_string_lossy();
    if executable.chars().any(char::is_control) {
        return Err("invalid executable path".into());
    }
    // Desktop Entry Exec quoting is not shell quoting; percent is a field code.
    let quoted = executable
        .replace('\\', "\\\\\\\\")
        .replace('"', "\\\"")
        .replace('`', "\\`")
        .replace('$', "\\$")
        .replace('%', "%%");
    let contents = format!("[Desktop Entry]\nType=Application\nVersion=1.0\nName=Keepr\nExec=\"{quoted}\" --background\nIcon=keepr\nTerminal=false\nStartupNotify=false\n");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("desktop.tmp");
    std::fs::write(&temporary, contents)?;
    std::fs::rename(temporary, path)?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn enabled(app: &AppHandle) -> Result<bool, Box<dyn std::error::Error>> {
    use tauri_plugin_autostart::ManagerExt;
    Ok(app.autolaunch().is_enabled()?)
}

#[cfg(target_os = "windows")]
pub fn set(app: &AppHandle, enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
    use tauri_plugin_autostart::ManagerExt;
    if enabled {
        app.autolaunch().enable()?;
    } else {
        app.autolaunch().disable()?;
    }
    Ok(())
}
