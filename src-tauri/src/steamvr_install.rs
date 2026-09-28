use serde_json::Value;
use std::{env, fs, path::{Path, PathBuf}, process::Command};

fn vrpathreg() -> Result<PathBuf, String> {
    let local = env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is not set")?;
    let paths = PathBuf::from(local).join("openvr").join("openvrpaths.vrpath");
    let json: Value = serde_json::from_slice(&fs::read(&paths).map_err(|e| format!("{}: {e}", paths.display()))?)
        .map_err(|e| e.to_string())?;
    let runtime = json.get("runtime").and_then(Value::as_array)
        .and_then(|paths| paths.iter().filter_map(Value::as_str).map(PathBuf::from).find(|path| path.join("bin/win64/vrpathreg.exe").is_file()))
        .ok_or("SteamVR runtime was not found in openvrpaths.vrpath")?;
    Ok(runtime.join("bin/win64/vrpathreg.exe"))
}

fn driver_dir() -> Result<PathBuf, String> {
    let exe = env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe.parent().ok_or("Application directory is missing")?.join("driver_qptp");
    if !dir.join("driver.vrdrivermanifest").is_file() ||
        !dir.join("bin/win64/driver_qptp.dll").is_file() ||
        !dir.join("resources/input/qptp_profile.json").is_file() ||
        !dir.join("resources/input/qptp_battery_profile.json").is_file() {
        return Err(format!("Incomplete SteamVR driver package: {}", dir.display()));
    }
    Ok(dir)
}

fn run(vrpathreg: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new(vrpathreg).args(args).output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!("vrpathreg {:?}: {}", args, String::from_utf8_lossy(&output.stderr)));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

pub fn register() -> Result<(), String> {
    let tool = vrpathreg()?;
    let driver = driver_dir()?;
    let path = driver.to_str().ok_or("Driver path is not Unicode")?;
    // Only one qptp package should be registered, including upgrades from a dev checkout.
    run(&tool, &["removedriverswithname", "qptp"])?;
    run(&tool, &["adddriver", path])?;
    Ok(())
}

pub fn ensure_registered() -> Result<(), String> {
    let tool = vrpathreg()?;
    let driver = driver_dir()?;
    let output = Command::new(&tool).args(["finddriver", "qptp"]).output().map_err(|e| e.to_string())?;
    if output.status.success() {
        let current = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if Path::new(&current) == driver { return Ok(()); }
    }
    register()
}

pub fn unregister() -> Result<(), String> {
    let tool = vrpathreg()?;
    let driver = driver_dir()?;
    let path = driver.to_str().ok_or("Driver path is not Unicode")?;
    run(&tool, &["removedriver", path])?;
    Ok(())
}
