/// System permissions verification module
///
/// Checks if MultiShiva has the necessary permissions to capture and inject
/// input events on different operating systems.
use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum PermissionStatus {
    /// All required permissions are granted
    Granted,
    /// Some permissions are missing
    Denied { missing: Vec<String> },
    /// Unable to determine permission status
    Unknown,
}

impl PermissionStatus {
    pub fn is_granted(&self) -> bool {
        matches!(self, PermissionStatus::Granted)
    }

    pub fn missing_permissions(&self) -> Vec<String> {
        match self {
            PermissionStatus::Denied { missing } => missing.clone(),
            _ => vec![],
        }
    }
}

/// Check system permissions for input capture and injection
pub fn check_permissions() -> Result<PermissionStatus> {
    #[cfg(target_os = "macos")]
    {
        check_macos_permissions()
    }

    #[cfg(target_os = "linux")]
    {
        check_linux_permissions()
    }

    #[cfg(target_os = "windows")]
    {
        check_windows_permissions()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Ok(PermissionStatus::Unknown)
    }
}

/// Get help message for fixing permission issues on current OS
pub fn get_permission_help() -> String {
    #[cfg(target_os = "macos")]
    {
        get_macos_help()
    }

    #[cfg(target_os = "linux")]
    {
        get_linux_help()
    }

    #[cfg(target_os = "windows")]
    {
        get_windows_help()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        "Unknown operating system. MultiShiva supports Linux, macOS, and Windows.".to_string()
    }
}

#[cfg(target_os = "macos")]
fn check_macos_permissions() -> Result<PermissionStatus> {
    use std::process::Command;

    // Check if we can access Accessibility API
    // Note: This is a simplified check. Full check would use macOS APIs via FFI
    let output = Command::new("ioreg")
        .arg("-c")
        .arg("IOHIDSystem")
        .output()
        .context("Failed to check macOS permissions")?;

    if output.status.success() {
        // Try to detect if Accessibility is enabled
        // In a real implementation, we'd use macOS Security framework via FFI
        Ok(PermissionStatus::Granted)
    } else {
        Ok(PermissionStatus::Denied {
            missing: vec!["Accessibility".to_string()],
        })
    }
}

#[cfg(target_os = "macos")]
fn get_macos_help() -> String {
    r#"macOS Permissions Required
==========================

MultiShiva needs Accessibility permissions to capture and inject input events.

How to grant permissions:
1. Open System Settings (or System Preferences)
2. Go to Privacy & Security → Accessibility
3. Add MultiShiva to the list of allowed applications
4. Enable the checkbox next to MultiShiva

Alternative command line:
sudo sqlite3 /Library/Application\ Support/com.apple.TCC/TCC.db \
  "INSERT or REPLACE INTO access VALUES('kTCCServiceAccessibility','com.yourapp.multishiva',0,1,1,NULL,NULL,NULL,'UNUSED',NULL,0,NULL);"

Note: You may need to restart MultiShiva after granting permissions.
"#.to_string()
}

#[cfg(target_os = "linux")]
fn check_linux_permissions() -> Result<PermissionStatus> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let mut missing = Vec::new();

    // Check if /dev/uinput exists and is accessible
    if let Ok(metadata) = fs::metadata("/dev/uinput") {
        let perms = metadata.permissions();
        let mode = perms.mode();

        // Check if readable and writable
        if mode & 0o600 != 0o600 {
            // Check if user is in input group
            if !is_user_in_group("input")? {
                missing.push("input group membership or /dev/uinput access".to_string());
            }
        }
    } else {
        missing.push("uinput kernel module".to_string());
    }

    // Check if X11 or Wayland is available
    if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
        missing.push("X11 or Wayland display server".to_string());
    }

    if missing.is_empty() {
        Ok(PermissionStatus::Granted)
    } else {
        Ok(PermissionStatus::Denied { missing })
    }
}

#[cfg(target_os = "linux")]
fn is_user_in_group(group_name: &str) -> Result<bool> {
    use std::process::Command;

    let output = Command::new("groups")
        .output()
        .context("Failed to check user groups")?;

    if !output.status.success() {
        return Ok(false);
    }

    let groups = String::from_utf8_lossy(&output.stdout);
    Ok(groups.split_whitespace().any(|g| g == group_name))
}

#[cfg(target_os = "linux")]
fn get_linux_help() -> String {
    r#"Linux Permissions Required
==========================

MultiShiva needs access to /dev/uinput and input devices.

How to grant permissions:

1. Load uinput kernel module:
   sudo modprobe uinput

   To load automatically at boot:
   echo 'uinput' | sudo tee /etc/modules-load.d/uinput.conf

2. Add your user to the input group:
   sudo usermod -a -G input $USER

   Then log out and log back in for changes to take effect.

3. Install required libraries:
   # Debian/Ubuntu
   sudo apt-get install libx11-dev libxtst-dev libevdev-dev

   # Fedora/RHEL
   sudo dnf install libX11-devel libXtst-devel libevdev-devel

   # Arch
   sudo pacman -S libx11 libxtst libevdev

4. For Wayland support:
   sudo apt-get install libwayland-dev

Alternative: Set uinput permissions directly (not recommended):
   sudo chmod 666 /dev/uinput
"#
    .to_string()
}

#[cfg(target_os = "windows")]
fn check_windows_permissions() -> Result<PermissionStatus> {
    use std::process::Command;

    // Check if running as Administrator (optional but recommended)
    let output = Command::new("net")
        .args(["session"])
        .output()
        .context("Failed to check Windows permissions")?;

    if output.status.success() {
        // If net session succeeds, we're running as admin
        Ok(PermissionStatus::Granted)
    } else {
        // Not running as admin, but input injection might still work
        // Windows doesn't strictly require admin for SendInput API
        Ok(PermissionStatus::Granted)
    }
}

#[cfg(target_os = "windows")]
fn get_windows_help() -> String {
    r#"Windows Permissions
===================

MultiShiva should work without special permissions on Windows.

However, for best results:

1. Run as Administrator (optional):
   Right-click on multishiva.exe → "Run as administrator"

2. If you encounter issues with input injection:
   - Check Windows Defender / Antivirus settings
   - Add MultiShiva to the exclusions list
   - Some security software may block input injection

3. For production use, code signing is recommended:
   - Sign the executable with a valid certificate
   - This prevents Windows SmartScreen warnings
   - Improves trust and prevents false antivirus alerts

4. Windows UAC prompts:
   - If targeting elevated applications, MultiShiva must also be elevated
   - Consider using Task Scheduler for auto-start with elevation

No installation or system configuration is typically required.
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_status_is_granted() {
        let granted = PermissionStatus::Granted;
        assert!(granted.is_granted());

        let denied = PermissionStatus::Denied {
            missing: vec!["test".to_string()],
        };
        assert!(!denied.is_granted());

        let unknown = PermissionStatus::Unknown;
        assert!(!unknown.is_granted());
    }

    #[test]
    fn test_permission_status_missing() {
        let denied = PermissionStatus::Denied {
            missing: vec!["perm1".to_string(), "perm2".to_string()],
        };
        assert_eq!(denied.missing_permissions().len(), 2);

        let granted = PermissionStatus::Granted;
        assert_eq!(granted.missing_permissions().len(), 0);
    }

    #[test]
    fn test_check_permissions() {
        // This test will work on any supported OS
        let result = check_permissions();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_permission_help() {
        let help = get_permission_help();
        assert!(!help.is_empty());

        // Help should contain OS-specific information
        #[cfg(target_os = "macos")]
        assert!(help.contains("Accessibility"));

        #[cfg(target_os = "linux")]
        assert!(help.contains("uinput"));

        #[cfg(target_os = "windows")]
        assert!(help.contains("Windows"));
    }
}
