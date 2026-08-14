use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::util;

const ALIEN_FILES: [(&str, &str); 9] = [
    ("alien.pl", include_str!("../alien/alien.pl")),
    ("Alien/Package.pm", include_str!("../alien/Alien/Package.pm")),
    ("Alien/Package/Deb.pm", include_str!("../alien/Alien/Package/Deb.pm")),
    ("Alien/Package/Rpm.pm", include_str!("../alien/Alien/Package/Rpm.pm")),
    ("Alien/Package/Tgz.pm", include_str!("../alien/Alien/Package/Tgz.pm")),
    ("Alien/Package/Slp.pm", include_str!("../alien/Alien/Package/Slp.pm")),
    ("Alien/Package/Pkg.pm", include_str!("../alien/Alien/Package/Pkg.pm")),
    ("Alien/Package/Lsb.pm", include_str!("../alien/Alien/Package/Lsb.pm")),
    ("Alien/Package/Dir.pm", include_str!("../alien/Alien/Package/Dir.pm")),
];

pub struct Converted {
    pub file: PathBuf,
    pub work_dir: PathBuf,
}

pub fn convert(input: &Path, dest: &str) -> Result<Converted, String> {
    if !util::command_exists("perl") {
        return Err(
            "converting packages with alien requires Perl, but perl is not installed.\n\
             Install perl (e.g. `dnf install perl` or `apt install perl`) and try again."
                .into(),
        );
    }

    let alien_dir = materialize()?;

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let work = std::env::temp_dir().join(format!("univ_alien_{}_{}", std::process::id(), nanos));
    std::fs::create_dir_all(&work)
        .map_err(|e| format!("cannot create {}: {e}", work.display()))?;

    println!("[univ] Converting {} to {dest} with alien...", input.display());
    let status = Command::new("perl")
        .arg("-I")
        .arg(&alien_dir)
        .arg(alien_dir.join("alien.pl"))
        .arg(format!("--to-{dest}"))
        .arg(input)
        .current_dir(&work)
        .status()
        .map_err(|e| format!("failed to run perl: {e}"))?;
    if !status.success() {
        let _ = std::fs::remove_dir_all(&work);
        return Err("alien conversion failed".into());
    }

    let file = std::fs::read_dir(&work)
        .ok()
        .and_then(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .find(|p| p.extension().and_then(|s| s.to_str()) == Some(dest))
        })
        .ok_or_else(|| format!("alien did not produce a .{dest} file"))?;

    Ok(Converted { file, work_dir: work })
}

fn materialize() -> Result<PathBuf, String> {
    let dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("univ/alien");

    for (rel, content) in ALIEN_FILES {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
        }
        std::fs::write(&path, content)
            .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    }

    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn build_deb() -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("univ_alien_fixture_{}_{}", std::process::id(), nanos));
        std::fs::create_dir_all(dir.join("pkg/usr/bin")).unwrap();
        std::fs::write(dir.join("pkg/usr/bin/hi"), "#!/bin/sh\necho hi\n").unwrap();
        std::fs::set_permissions(dir.join("pkg/usr/bin/hi"), std::fs::Permissions::from_mode(0o755))
        .unwrap();

        let pkg = dir.join("pkg");
        let run = |cwd: &Path, args: &[&str]| {
            let s = Command::new(args[0]).args(&args[1..]).current_dir(cwd).status().unwrap();
            assert!(s.success());
        };

        run(&pkg, &["tar", "-czf", "../data.tar.gz", "usr"]);
        std::fs::write(
            dir.join("control"),
            "Package: hi\nVersion: 1.0\nArchitecture: all\nMaintainer: t <t@t>\nDescription: test\n",
        )
        .unwrap();
        run(&dir, &["tar", "-czf", "control.tar.gz", "control"]);
        std::fs::write(dir.join("debian-binary"), "2.0\n").unwrap();
        run(&dir, &["ar", "r", "hi_1.0_all.deb", "debian-binary", "control.tar.gz", "data.tar.gz"]);

        let deb = dir.join("hi_1.0_all.deb");
        let _ = std::fs::remove_file(dir.join("control.tar.gz"));
        let _ = std::fs::remove_file(dir.join("data.tar.gz"));
        let _ = std::fs::remove_dir_all(&pkg);
        deb
    }

    #[test]
    fn converts_deb_with_embedded_alien() {
        if !util::command_exists("perl") {
            eprintln!("skipping: perl not installed");
            return;
        }
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let xdg = std::env::temp_dir().join(format!("univ_alien_xdg_{}_{}", std::process::id(), nanos));
        // SAFETY: test runs single-threaded; no other threads observe the env change.
        unsafe { std::env::set_var("XDG_DATA_HOME", &xdg) };

        let deb = build_deb();
        let converted = convert(&deb, "tgz").expect("alien convert failed");
        assert!(converted.file.exists());
        assert_eq!(converted.file.extension().and_then(|s| s.to_str()), Some("tgz"));

        let _ = std::fs::remove_dir_all(&converted.work_dir);
        let _ = std::fs::remove_dir_all(&xdg);
        let _ = std::fs::remove_file(&deb);
    }
}