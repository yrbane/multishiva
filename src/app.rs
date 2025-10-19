/// GUI Application module
///
/// Launches the Tauri-based GUI application for MultiShiva.
///
/// ## Development Mode
///
/// In development, this will spawn `cargo tauri dev` to run the GUI with hot-reload.
///
/// ## Production Mode
///
/// In production builds, users should use the standalone Tauri application built
/// with `cargo tauri build`, which creates platform-specific installers.
///
/// ## Usage
///
/// ```bash
/// # Development
/// cargo run -- --gui
/// # Or directly:
/// cargo tauri dev
///
/// # Production
/// cargo tauri build
/// ```
use anyhow::Result;
use std::process::Command;

/// Launch the Tauri GUI application
pub fn launch_gui() -> Result<()> {
    tracing::info!("Starting MultiShiva GUI v{}", env!("CARGO_PKG_VERSION"));

    // Check if we're in development mode (cargo is available)
    if is_dev_environment() {
        launch_dev_gui()
    } else {
        launch_prod_gui()
    }
}

/// Check if running in development environment
fn is_dev_environment() -> bool {
    // Check if we're running from cargo (not an installed binary)
    std::env::current_exe()
        .ok()
        .and_then(|path| path.to_str().map(|s| s.contains("target")))
        .unwrap_or(false)
}

/// Launch GUI in development mode using cargo tauri dev
fn launch_dev_gui() -> Result<()> {
    tracing::info!("Launching GUI in development mode...");
    tracing::info!("Running: cargo tauri dev");

    let status = Command::new("cargo").arg("tauri").arg("dev").status();

    match status {
        Ok(exit_status) => {
            if exit_status.success() {
                Ok(())
            } else {
                anyhow::bail!("Tauri dev server exited with error")
            }
        }
        Err(e) => {
            tracing::error!("Failed to launch Tauri GUI: {}", e);
            anyhow::bail!(
                "Failed to launch GUI. Make sure you have Tauri CLI installed:\n\
                 cargo install tauri-cli\n\n\
                 Or run directly:\n\
                 cargo tauri dev"
            )
        }
    }
}

/// Launch GUI in production mode
fn launch_prod_gui() -> Result<()> {
    tracing::error!("Production GUI launch not yet implemented");
    anyhow::bail!(
        "MultiShiva GUI must be built as a standalone application.\n\n\
         To build the production GUI:\n\
         cargo tauri build\n\n\
         This will create platform-specific installers in:\n\
         src-tauri/target/release/bundle/\n\n\
         For development, use:\n\
         cargo tauri dev"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dev_environment_detection() {
        // This test will pass in development (when running via cargo)
        // We just ensure the function doesn't panic
        let _ = is_dev_environment();
    }
}
