use vibe_core::config::AppConfig;
use vibe_core::ipc::VibeControlProxy;
use zbus::Connection;

#[tauri::command]
async fn next_wallpaper() -> Result<(), String> {
    let conn = Connection::session()
        .await
        .map_err(|e| format!("Failed to open D-Bus session connection: {}", e))?;
    let proxy = VibeControlProxy::new(&conn)
        .await
        .map_err(|e| format!("Failed to create daemon proxy: {}", e))?;
    proxy
        .next()
        .await
        .map_err(|e| format!("Daemon Communication Error: {}. Is vibe-daemon running?", e))?;
    Ok(())
}

#[tauri::command]
async fn pause_wallpaper() -> Result<(), String> {
    let conn = Connection::session()
        .await
        .map_err(|e| format!("Failed to open D-Bus session connection: {}", e))?;
    let proxy = VibeControlProxy::new(&conn)
        .await
        .map_err(|e| format!("Failed to create daemon proxy: {}", e))?;
    proxy
        .pause()
        .await
        .map_err(|e| format!("Daemon Communication Error: {}. Is vibe-daemon running?", e))?;
    Ok(())
}

#[tauri::command]
async fn resume_wallpaper() -> Result<(), String> {
    let conn = Connection::session()
        .await
        .map_err(|e| format!("Failed to open D-Bus session connection: {}", e))?;
    let proxy = VibeControlProxy::new(&conn)
        .await
        .map_err(|e| format!("Failed to create daemon proxy: {}", e))?;
    proxy
        .resume()
        .await
        .map_err(|e| format!("Daemon Communication Error: {}. Is vibe-daemon running?", e))?;
    Ok(())
}

#[tauri::command]
async fn reload_config() -> Result<(), String> {
    let conn = Connection::session()
        .await
        .map_err(|e| format!("Failed to open D-Bus session connection: {}", e))?;
    let proxy = VibeControlProxy::new(&conn)
        .await
        .map_err(|e| format!("Failed to create daemon proxy: {}", e))?;
    proxy
        .reload_config()
        .await
        .map_err(|e| format!("Failed to reload daemon config: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn get_status() -> Result<String, String> {
    let conn = Connection::session()
        .await
        .map_err(|e| format!("Failed to open D-Bus session connection: {}", e))?;
    let proxy = VibeControlProxy::new(&conn)
        .await
        .map_err(|e| format!("Failed to create daemon proxy: {}", e))?;
    let status = proxy
        .status()
        .await
        .map_err(|e| format!("Offline ({})", e))?;
    Ok(status)
}

#[tauri::command]
async fn favorite_wallpaper() -> Result<(), String> {
    let conn = Connection::session()
        .await
        .map_err(|e| format!("Failed to open D-Bus session connection: {}", e))?;
    let proxy = VibeControlProxy::new(&conn)
        .await
        .map_err(|e| format!("Failed to create daemon proxy: {}", e))?;
    proxy
        .favorite()
        .await
        .map_err(|e| format!("Daemon Communication Error: {}", e))?;
    Ok(())
}

#[tauri::command]
fn get_config() -> Result<AppConfig, String> {
    AppConfig::load_strict().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_config(config: AppConfig) -> Result<(), String> {
    config.validate().map_err(|e| e.to_string())?;
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            next_wallpaper,
            pause_wallpaper,
            resume_wallpaper,
            reload_config,
            get_status,
            favorite_wallpaper,
            get_config,
            save_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
