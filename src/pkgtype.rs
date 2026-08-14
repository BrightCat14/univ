use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PkgType {
    Deb,
    Rpm,
    TarGz,
    TarXz,
    TarBz2,
    TarZst,
    Tar,
    AppImage,
    Flatpak,
    Snap,
    Unknown,
}

impl PkgType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Deb => "Debian package (.deb)",
            Self::Rpm => "RPM package (.rpm)",
            Self::TarGz => "tar archive (.tar.gz)",
            Self::TarXz => "tar archive (.tar.xz)",
            Self::TarBz2 => "tar archive (.tar.bz2)",
            Self::TarZst => "tar archive (.tar.zst)",
            Self::Tar => "tar archive (.tar)",
            Self::AppImage => "AppImage",
            Self::Flatpak => "Flatpak bundle (.flatpak)",
            Self::Snap => "Snap package (.snap)",
            Self::Unknown => "unknown package type",
        }
    }
}

pub fn detect(path: &Path) -> PkgType {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    if name.ends_with(".deb") {
        return PkgType::Deb;
    }
    if name.ends_with(".rpm") {
        return PkgType::Rpm;
    }
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        return PkgType::TarGz;
    }
    if name.ends_with(".tar.xz") || name.ends_with(".txz") {
        return PkgType::TarXz;
    }
    if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") {
        return PkgType::TarBz2;
    }
    if name.ends_with(".tar.zst") || name.ends_with(".tzst") {
        return PkgType::TarZst;
    }
    if name.ends_with(".tar") {
        return PkgType::Tar;
    }
    if name.ends_with(".appimage") {
        return PkgType::AppImage;
    }
    if name.ends_with(".flatpak") {
        return PkgType::Flatpak;
    }
    if name.ends_with(".snap") {
        return PkgType::Snap;
    }
    PkgType::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_by_extension() {
        assert_eq!(detect(Path::new("app_1.0_amd64.deb")), PkgType::Deb);
        assert_eq!(detect(Path::new("app-1.0.x86_64.rpm")), PkgType::Rpm);
        assert_eq!(detect(Path::new("app-1.0.tar.gz")), PkgType::TarGz);
        assert_eq!(detect(Path::new("app.tgz")), PkgType::TarGz);
        assert_eq!(detect(Path::new("app-1.0.tar.xz")), PkgType::TarXz);
        assert_eq!(detect(Path::new("app.tar.bz2")), PkgType::TarBz2);
        assert_eq!(detect(Path::new("app.tar.zst")), PkgType::TarZst);
        assert_eq!(detect(Path::new("app-1.0.tar")), PkgType::Tar);
        assert_eq!(detect(Path::new("App.AppImage")), PkgType::AppImage);
        assert_eq!(detect(Path::new("app.flatpak")), PkgType::Flatpak);
        assert_eq!(detect(Path::new("app.snap")), PkgType::Snap);
        assert_eq!(detect(Path::new("archive.zip")), PkgType::Unknown);
    }
}
