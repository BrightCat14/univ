use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Distro {
    Void,
    Debian,
    Ubuntu,
    Arch,
    Fedora,
    Rhel,
    Alpine,
    OpenSuse,
    Gentoo,
    Unknown,
}

impl Distro {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Void => "Void Linux",
            Self::Debian => "Debian",
            Self::Ubuntu => "Ubuntu",
            Self::Arch => "Arch Linux",
            Self::Fedora => "Fedora",
            Self::Rhel => "RHEL/CentOS",
            Self::Alpine => "Alpine",
            Self::OpenSuse => "openSUSE",
            Self::Gentoo => "Gentoo",
            Self::Unknown => "Unknown",
        }
    }
}

pub fn detect() -> Distro {
    match os_release_id().as_str() {
        "void" => Distro::Void,
        "debian" => Distro::Debian,
        "ubuntu" | "linuxmint" | "pop" | "elementary" | "kali" => Distro::Ubuntu,
        "arch" | "manjaro" | "endeavouros" | "garuda" | "cachyos" => Distro::Arch,
        "fedora" => Distro::Fedora,
        "rhel" | "centos" | "rocky" | "almalinux" => Distro::Rhel,
        "alpine" => Distro::Alpine,
        "opensuse" | "opensuse-leap" | "opensuse-tumbleweed" => Distro::OpenSuse,
        "gentoo" => Distro::Gentoo,
        _ => Distro::Unknown,
    }
}

fn os_release_id() -> String {
    let content = fs::read_to_string("/etc/os-release").unwrap_or_default();
    for line in content.lines() {
        if let Some(rest) = line.trim().strip_prefix("ID=") {
            return rest.trim().trim_matches('"').to_string();
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    fn id_from(content: &str) -> String {
        content
            .lines()
            .find_map(|l| l.trim().strip_prefix("ID="))
            .map(|v| v.trim().trim_matches('"').to_string())
            .unwrap_or_default()
    }

    #[test]
    fn parses_os_release_id() {
        assert_eq!(id_from("NAME=\"Void\"\nID=void\n"), "void");
        assert_eq!(id_from("ID=\"ubuntu\"\n"), "ubuntu");
        assert_eq!(id_from("NAME=Debian\n"), "");
    }
}
