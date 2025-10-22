use multishiva::core::config::Config;
use std::path::PathBuf;
use dirs::config_dir;

// Tauri commands
#[tauri::command]
fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to MultiShiva.", name)
}

/// Get the default config file path
fn get_default_config_path() -> PathBuf {
    let mut path = config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("multishiva");
    path.push("multishiva.yml");
    path
}

#[tauri::command]
fn load_config(custom_path: Option<String>) -> Result<Config, String> {
    let path = if let Some(p) = custom_path {
        PathBuf::from(p)
    } else {
        get_default_config_path()
    };

    Config::from_file(path.to_str().unwrap())
        .map_err(|e| format!("Failed to load config: {}", e))
}

#[tauri::command]
fn save_config(config: Config, custom_path: Option<String>) -> Result<(), String> {
    let path = if let Some(p) = custom_path {
        PathBuf::from(p)
    } else {
        get_default_config_path()
    };

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    config.save_to_file(&path)
        .map_err(|e| format!("Failed to save config: {}", e))
}

#[tauri::command]
fn get_config_path() -> String {
    get_default_config_path()
        .to_string_lossy()
        .to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_version,
            greet,
            load_config,
            save_config,
            get_config_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
