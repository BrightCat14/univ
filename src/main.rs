mod alien;
mod appimage;
mod distro;
mod native;
mod pkgtype;
mod registry;
mod tar_install;
mod uninstall;
mod util;
mod xdeb;

use std::path::Path;
use std::process;

use distro::Distro;
use pkgtype::PkgType;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_help(&args[0]);
        process::exit(1);
    }

    let distro = distro::detect();

    match args[1].as_str() {
        "install" => cmd_install(distro, &args[2..]),
        "uninstall" | "remove" => cmd_uninstall(distro, &args[2..]),
        "list" => cmd_list(),
        "help" | "--help" | "-h" => print_help(&args[0]),
        other => {
            eprintln!("[univ] unknown command: {other}");
            print_help(&args[0]);
            process::exit(1);
        }
    }
}

fn cmd_install(distro: Distro, files: &[String]) {
    if files.is_empty() {
        eprintln!("[univ] usage: univ install <package-file> [more files...]");
        process::exit(1);
    }
    for f in files {
        if let Err(e) = install_one(distro, Path::new(f)) {
            eprintln!("[univ] error: {e}");
            process::exit(1);
        }
    }
}

fn install_one(distro: Distro, path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("file not found: {}", path.display()));
    }

    let pkg = pkgtype::detect(path);
    println!("[univ] {} on {}", pkg.label(), distro.name());

    match pkg {
        PkgType::Deb => match distro {
            Distro::Void => xdeb::install_deb_on_void(path),
            Distro::Debian | Distro::Ubuntu => native::install_deb(path),
            Distro::Fedora | Distro::Rhel => {
                let converted = alien::convert(path, "rpm")?;
                let result = native::install_rpm(&converted.file);
                let _ = std::fs::remove_dir_all(&converted.work_dir);
                result
            }
            other => Err(format!(
                ".deb packages are not supported on {}; convert the package manually",
                other.name()
            )),
        },
        PkgType::Rpm => match distro {
            Distro::Fedora | Distro::Rhel => native::install_rpm(path),
            Distro::Debian | Distro::Ubuntu => {
                let converted = alien::convert(path, "deb")?;
                let result = native::install_deb(&converted.file);
                let _ = std::fs::remove_dir_all(&converted.work_dir);
                result
            }
            other => Err(format!(
                ".rpm packages are not supported on {}; convert the package manually",
                other.name()
            )),
        },
        PkgType::TarGz | PkgType::TarXz | PkgType::TarBz2 | PkgType::TarZst | PkgType::Tar => {
            let installed = tar_install::install_tar(path)?;
            let mut reg = registry::Registry::load();
            reg.add(installed.entry);
            Ok(())
        }
        PkgType::AppImage => {
            let entry = appimage::install_appimage(path)?;
            let mut reg = registry::Registry::load();
            reg.add(entry.clone());
            println!("[univ] Installed {}", entry.name);
            Ok(())
        }
        PkgType::Flatpak => {
            native::install_flatpak(path)?;
            let mut reg = registry::Registry::load();
            reg.add(registry::RegistryEntry {
                name: path.file_stem().and_then(|s| s.to_str()).unwrap_or("flatpak").to_string(),
                source: "flatpak".into(),
                install_path: None,
                bin_link: None,
                desktop_file: None,
            });
            Ok(())
        }
        PkgType::Snap => {
            native::install_snap(path)?;
            let mut reg = registry::Registry::load();
            reg.add(registry::RegistryEntry {
                name: path.file_stem().and_then(|s| s.to_str()).unwrap_or("snap").to_string(),
                source: "snap".into(),
                install_path: None,
                bin_link: None,
                desktop_file: None,
            });
            Ok(())
        }
        PkgType::Unknown => Err(format!("cannot determine package type for {}", path.display())),
    }
}

fn cmd_uninstall(distro: Distro, names: &[String]) {
    if names.is_empty() {
        eprintln!("[univ] usage: univ uninstall <package-name> [more names...]");
        process::exit(1);
    }
    for n in names {
        if let Err(e) = uninstall::uninstall(distro, n) {
            eprintln!("[univ] error: {e}");
            process::exit(1);
        }
    }
}

fn cmd_list() {
    let reg = registry::Registry::load();
    if reg.apps.is_empty() {
        println!("[univ] no packages tracked by univ");
        return;
    }
    for a in &reg.apps {
        println!(
            "{:<20} source={:<10} path={}",
            a.name,
            a.source,
            a.install_path.as_deref().unwrap_or("-")
        );
    }
}

fn print_help(prog: &str) {
    println!(
        "\
univ - universal package manager

Usage:
  {prog} install <file>...     Install packages (deb, rpm, tar.gz, AppImage, flatpak, snap)
  {prog} uninstall <name>...   Remove packages
  {prog} list                  List packages installed through univ
  {prog} help                  Show this help

Environment:
  UNIV_XDEB     Override path to the xdeb script (by default xdeb is embedded in the binary)
  XDEB_PKGROOT  Working directory for xdeb (default: $XDG_DATA_HOME/univ/xdeb)"
    );
}
