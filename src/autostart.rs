//! Login launch is controlled by the operating system, not by a saved profile.
use anyhow::{Result, bail};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Disabled,
    Enabled,
    RequiresApproval,
    Unavailable,
}

#[cfg(target_os = "macos")]
pub fn status() -> Result<Status> {
    use objc2_service_management::{SMAppService, SMAppServiceStatus};
    // SMAppService is available on macOS 13 and later. The class lookup is
    // guarded so the application still starts on older systems.
    if objc2::runtime::AnyClass::get(c"SMAppService").is_none() {
        return Ok(Status::Unavailable);
    }
    let service = unsafe { SMAppService::mainAppService() };
    let current = unsafe { service.status() };
    Ok(if current == SMAppServiceStatus::Enabled {
        Status::Enabled
    } else if current == SMAppServiceStatus::RequiresApproval {
        Status::RequiresApproval
    } else if current == SMAppServiceStatus::NotRegistered
        || current == SMAppServiceStatus::NotFound
    {
        Status::Disabled
    } else {
        Status::Unavailable
    })
}

#[cfg(target_os = "macos")]
pub fn set_enabled(enabled: bool) -> Result<Status> {
    use objc2_service_management::SMAppService;
    if status()? == Status::Unavailable {
        bail!("Login items unavailable");
    }
    let service = unsafe { SMAppService::mainAppService() };
    if enabled {
        unsafe { service.registerAndReturnError() }.map_err(|e| anyhow::anyhow!("{e}"))?;
    } else {
        unsafe { service.unregisterAndReturnError() }.map_err(|e| anyhow::anyhow!("{e}"))?;
    }
    status()
}

#[cfg(target_os = "windows")]
const KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
#[cfg(target_os = "windows")]
const VALUE: &str = "Weikav Studio";
#[cfg(target_os = "windows")]
const APPROVED_KEY: &str =
    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";

#[cfg(target_os = "windows")]
fn reg() -> std::process::Command {
    use std::os::windows::process::CommandExt;
    let mut command = std::process::Command::new("reg.exe");
    command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    command
}

#[cfg(target_os = "windows")]
pub fn status() -> Result<Status> {
    let result = reg().args(["query", KEY, "/v", VALUE]).output()?;
    if !result.status.success() {
        return Ok(Status::Disabled);
    }
    // Windows Settings can independently disable an otherwise registered Run entry.
    let approval = reg().args(["query", APPROVED_KEY, "/v", VALUE]).output()?;
    if approval.status.success() {
        let output = String::from_utf8_lossy(&approval.stdout);
        if let Some((_, data)) = output.split_once("REG_BINARY") {
            let data = data.trim_start();
            if data.starts_with("03") || data.starts_with("07") {
                return Ok(Status::RequiresApproval);
            }
        }
    }
    Ok(Status::Enabled)
}

#[cfg(target_os = "windows")]
pub fn set_enabled(enabled: bool) -> Result<Status> {
    let result = if enabled {
        let executable = std::env::current_exe()?;
        let command = format!("\"{}\"", executable.display());
        if command.encode_utf16().count() > 260 {
            bail!("Startup command is too long");
        }
        reg()
            .args([
                "add", KEY, "/v", VALUE, "/t", "REG_SZ", "/d", &command, "/f",
            ])
            .output()?
    } else {
        reg().args(["delete", KEY, "/v", VALUE, "/f"]).output()?
    };
    if !result.status.success() {
        bail!("Could not update startup setting");
    }
    status()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn status() -> Result<Status> {
    Ok(Status::Unavailable)
}
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn set_enabled(_: bool) -> Result<Status> {
    bail!("Unsupported platform")
}
