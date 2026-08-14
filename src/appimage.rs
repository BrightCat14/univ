use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::registry::RegistryEntry;
use crate::util;

pub fn install_appimage(path: &Path) -> Result<RegistryEntry, String> {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("appimage")
        .to_string();

    let (apps_dir, bin_dir) = if util::is_root() {
        (PathBuf::from("/opt"), PathBuf::from("/usr/local/bin"))
    } else {
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        (base.join("univ/apps"), home.join(".local/bin"))
    };

    fs_create_dir_all(&apps_dir)?;
    fs_create_dir_all(&bin_dir)?;

    let dest = apps_dir.join(format!("{name}.AppImage"));
    if dest.exists() {
        std::fs::remove_file(&dest).map_err(|e| format!("cannot replace {}: {e}", dest.display()))?;
    }
    std::fs::copy(path, &dest).map_err(|e| format!("cannot copy {}: {e}", path.display()))?;

    let mut perms = std::fs::metadata(&dest)
        .map_err(|e| format!("cannot stat {}: {e}", dest.display()))?
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&dest, perms)
        .map_err(|e| format!("cannot chmod {}: {e}", dest.display()))?;

    let link = bin_dir.join(&name);
    if link.exists() {
        std::fs::remove_file(&link).ok();
    }
    std::os::unix::fs::symlink(&dest, &link)
        .map_err(|e| format!("cannot link {}: {e}", link.display()))?;

    Ok(RegistryEntry {
        name,
        source: "appimage".into(),
        install_path: Some(dest.to_string_lossy().into_owned()),
        bin_link: Some(link.to_string_lossy().into_owned()),
        desktop_file: None,
    })
}

fn fs_create_dir_all(p: &Path) -> Result<(), String> {
    std::fs::create_dir_all(p).map_err(|e| format!("cannot create {}: {e}", p.display()))
}
