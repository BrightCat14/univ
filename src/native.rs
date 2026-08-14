use std::path::Path;

use crate::distro::Distro;
use crate::util;

pub fn install_deb(path: &Path) -> Result<(), String> {
    let p = path.to_str().ok_or("invalid path")?;
    util::run_privileged_status(&["dpkg", "-i", p])?;
    let _ = util::run_privileged_status(&["apt", "-f", "install", "-y"]);
    Ok(())
}

pub fn install_rpm(path: &Path) -> Result<(), String> {
    let p = path.to_str().ok_or("invalid path")?;
    util::run_privileged_status(&["dnf", "install", "-y", p])
}

pub fn install_flatpak(path: &Path) -> Result<(), String> {
    let p = path.to_str().ok_or("invalid path")?;
    util::run_status("flatpak", &["install", "--user", "-y", p])
}

pub fn install_snap(path: &Path) -> Result<(), String> {
    let p = path.to_str().ok_or("invalid path")?;
    util::run_privileged_status(&["snap", "install", p, "--dangerous"])
}

pub fn native_remove(distro: Distro, name: &str) -> Result<(), String> {
    match distro {
        Distro::Void => util::run_privileged_status(&["xbps-remove", "-R", name]),
        Distro::Debian | Distro::Ubuntu => util::run_privileged_status(&["apt", "remove", "-y", name]),
        Distro::Arch => util::run_privileged_status(&["pacman", "-Rns", "--noconfirm", name]),
        Distro::Fedora | Distro::Rhel => util::run_privileged_status(&["dnf", "remove", "-y", name]),
        Distro::Alpine => util::run_privileged_status(&["apk", "del", name]),
        Distro::OpenSuse => util::run_privileged_status(&["zypper", "remove", "-y", name]),
        Distro::Gentoo => util::run_privileged_status(&["emerge", "--unmerge", name]),
        Distro::Unknown => Err(format!("cannot remove {name}: unknown distro")),
    }
}

pub fn is_installed(distro: Distro, name: &str) -> bool {
    match distro {
        Distro::Void => {
            util::run_stdout("xbps-query", &[name])
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false)
        }
        Distro::Debian | Distro::Ubuntu => util::run_stdout("dpkg", &["-s", name])
            .map(|s| s.contains("install ok installed"))
            .unwrap_or(false),
        Distro::Arch => util::run_stdout("pacman", &["-Q", name])
            .map(|s| s.trim().contains(name))
            .unwrap_or(false),
        Distro::Fedora | Distro::Rhel => util::run_stdout("rpm", &["-q", name])
            .map(|s| !s.contains("not installed"))
            .unwrap_or(false),
        Distro::Alpine => util::run_stdout("apk", &["info", "-e", name])
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false),
        Distro::OpenSuse => util::run_stdout("zypper", &["se", "--installed-only", name])
            .map(|s| s.contains(name))
            .unwrap_or(false),
        Distro::Gentoo => util::run_stdout("qlist", &["-I", name])
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false),
        Distro::Unknown => false,
    }
}
