use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use tarinsta::{InstallOptions, Installer};

use crate::registry::RegistryEntry;
use crate::util;

pub struct InstalledTar {
    pub entry: RegistryEntry,
}

pub fn install_tar(tar_path: &Path) -> Result<InstalledTar, String> {
    let installer = Installer::new("en").map_err(|e| e.to_string())?;

    let (gz, tmp_dir) = ensure_tar_gz(tar_path)?;
    let result = install_gz(&installer, &gz);
    if let Some(dir) = tmp_dir {
        let _ = std::fs::remove_dir_all(dir);
    }
    result
}

fn install_gz(installer: &Installer, gz: &Path) -> Result<InstalledTar, String> {
    let package = installer.inspect(gz).map_err(|e| e.to_string())?;

    let mut opts = default_opts();
    let install_path = opts.install_path(&package.app_name);

    if install_path.exists() && !opts.overwrite {
        println!(
            "[univ] {} is already installed at {}. Overwrite? [y/N]",
            package.app_name,
            install_path.display()
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        if !input.trim().eq_ignore_ascii_case("y") {
            return Err("installation cancelled".into());
        }
        opts.overwrite = true;
    }

    if package.binaries.len() > 1 {
        println!("[univ] Multiple binaries found in the archive:");
        for (i, b) in package.binaries.iter().enumerate() {
            println!("  {}: {}", i + 1, b.display());
        }
        print!("[univ] Choose binary [1-{}]: ", package.binaries.len());
        std::io::stdout().flush().ok();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        opts.bin_index = input
            .trim()
            .parse::<usize>()
            .map(|n| n.saturating_sub(1))
            .unwrap_or(0);
    }

    let installed = installer.install(&package, &opts).map_err(|e| e.to_string())?;

    println!("[univ] Installed {} at {}", installed.app_name, installed.install_path.display());
    println!("[univ] Binary linked at {}", installed.bin_link.display());
    if let Some(df) = &installed.desktop_file {
        println!("[univ] Desktop entry created at {}", df.display());
    }

    Ok(InstalledTar {
        entry: RegistryEntry {
            name: installed.app_name,
            source: "tar".into(),
            install_path: Some(installed.install_path.to_string_lossy().into_owned()),
            bin_link: Some(installed.bin_link.to_string_lossy().into_owned()),
            desktop_file: installed.desktop_file.map(|p| p.to_string_lossy().into_owned()),
        },
    })
}

fn default_opts() -> InstallOptions {
    let mut opts = InstallOptions::default();
    if !util::is_root() {
        opts.install_root = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("univ/apps");
    }
    opts
}

fn ensure_tar_gz(path: &Path) -> Result<(PathBuf, Option<PathBuf>), String> {
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        return Ok((path.to_path_buf(), None));
    }

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp = std::env::temp_dir().join(format!("univ_tar_{}_{}", std::process::id(), nanos));
    let extract = tmp.join("x");
    std::fs::create_dir_all(&extract)
        .map_err(|e| format!("cannot create {}: {e}", extract.display()))?;

    let status = std::process::Command::new("tar")
        .arg("-xf")
        .arg(path)
        .arg("-C")
        .arg(&extract)
        .status()
        .map_err(|e| format!("failed to run tar: {e}"))?;
    if !status.success() {
        let _ = std::fs::remove_dir_all(&tmp);
        return Err(format!("cannot extract {}", path.display()));
    }

    let gz = tmp.join(format!("{}.gz", path.file_stem().and_then(|s| s.to_str()).unwrap_or("package.tar")));
    let status = std::process::Command::new("tar")
        .arg("-czf")
        .arg(&gz)
        .arg("-C")
        .arg(&extract)
        .arg(".")
        .status()
        .map_err(|e| format!("failed to run tar: {e}"))?;
    if !status.success() {
        let _ = std::fs::remove_dir_all(&tmp);
        return Err("cannot repack archive as tar.gz".into());
    }

    let _ = std::fs::remove_dir_all(&extract);
    Ok((gz, Some(tmp)))
}
