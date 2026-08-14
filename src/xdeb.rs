use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::util;

const XDEB_SCRIPT: &str = include_str!("../xdeb/xdeb");

pub fn install_deb_on_void(deb: &Path) -> Result<(), String> {
    let xdeb = find_xdeb()?;
    let pkroot = xdeb_pkroot();
    std::fs::create_dir_all(&pkroot).map_err(|e| format!("cannot create {}: {e}", pkroot.display()))?;

    println!("[univ] Converting .deb to xbps with xdeb...");
    let status = std::process::Command::new(&xdeb)
        .env("XDEB_PKGROOT", &pkroot)
        .arg("-Sedf")
        .arg(deb)
        .status()
        .map_err(|e| format!("failed to run xdeb: {e}"))?;
    if !status.success() {
        return Err("xdeb conversion failed".into());
    }

    let name = deb_name(deb);
    let binpkgs = pkroot.join("binpkgs");
    if !binpkgs.exists() {
        return Err(format!("xdeb did not produce binpkgs at {}", binpkgs.display()));
    }

    println!("[univ] Installing {name} via xbps-install...");
    let binpkgs_str = binpkgs.to_str().ok_or("invalid binpkgs path")?;
    util::run_privileged_status(&["xbps-install", "-R", binpkgs_str, &name])
}

fn find_xdeb() -> Result<PathBuf, String> {
    if let Ok(p) = std::env::var("UNIV_XDEB") {
        let p = PathBuf::from(p);
        if p.exists() {
            return Ok(p);
        }
    }

    let dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("univ/bin");
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;

    let path = dir.join("xdeb");
    if !path.exists() {
        std::fs::write(&path, XDEB_SCRIPT)
            .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        let mut perms = std::fs::metadata(&path)
            .map_err(|e| format!("cannot stat {}: {e}", path.display()))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms)
            .map_err(|e| format!("cannot chmod {}: {e}", path.display()))?;
    }
    Ok(path)
}

fn xdeb_pkroot() -> PathBuf {
    if let Ok(p) = std::env::var("XDEB_PKGROOT") {
        return PathBuf::from(p);
    }
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("univ/xdeb")
}

fn deb_name(deb: &Path) -> String {
    deb.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .split('_')
        .next()
        .unwrap_or("")
        .to_string()
}
