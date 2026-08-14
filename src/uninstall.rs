use std::path::Path;

use crate::distro::Distro;
use crate::native;
use crate::registry::Registry;

pub fn uninstall(distro: Distro, name: &str) -> Result<(), String> {
    let mut registry = Registry::load();

    if let Some(entry) = registry.find(name) {
        println!("[univ] Removing {name} (installed from {})", entry.source);

        if let Some(p) = &entry.install_path {
            let path = Path::new(p);
            if path.is_dir() {
                println!("[univ] Removing {}", path.display());
                std::fs::remove_dir_all(path)
                    .map_err(|e| format!("cannot remove {}: {e}", path.display()))?;
            } else if path.is_file() {
                println!("[univ] Removing {}", path.display());
                std::fs::remove_file(path)
                    .map_err(|e| format!("cannot remove {}: {e}", path.display()))?;
            }
        }
        if let Some(b) = &entry.bin_link {
            let link = Path::new(b);
            if std::fs::symlink_metadata(link).is_ok() {
                println!("[univ] Removing symlink {}", link.display());
                std::fs::remove_file(link)
                    .map_err(|e| format!("cannot remove {}: {e}", link.display()))?;
            }
        }
        if let Some(d) = &entry.desktop_file {
            let df = Path::new(d);
            if df.exists() {
                println!("[univ] Removing desktop entry {}", df.display());
                std::fs::remove_file(df)
                    .map_err(|e| format!("cannot remove {}: {e}", df.display()))?;
            }
        }

        registry.remove(name);
        println!("[univ] {name} removed");
        return Ok(());
    }

    if native::is_installed(distro, name) {
        println!("[univ] Removing {name} with the native package manager");
        native::native_remove(distro, name)?;
        println!("[univ] {name} removed");
        return Ok(());
    }

    Err(format!(
        "{name} is not installed (not tracked by univ and not found in {})",
        distro.name()
    ))
}
