use std::fs;
use std::path::PathBuf;

use toml::Value;

#[derive(Debug, Clone)]
pub struct RegistryEntry {
    pub name: String,
    pub source: String,
    pub install_path: Option<String>,
    pub bin_link: Option<String>,
    pub desktop_file: Option<String>,
}

#[derive(Debug)]
pub struct Registry {
    pub path: PathBuf,
    pub apps: Vec<RegistryEntry>,
}

impl Registry {
    pub fn load() -> Self {
        Self::load_from(default_path())
    }

    pub fn load_from(path: PathBuf) -> Self {
        let mut apps = Vec::new();
        if let Ok(content) = fs::read_to_string(&path)
            && let Ok(value) = toml::from_str::<Value>(&content)
            && let Some(array) = value.get("apps").and_then(|v| v.as_array())
        {
            for item in array {
                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if name.is_empty() {
                    continue;
                }
                apps.push(RegistryEntry {
                    name,
                    source: item.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    install_path: item.get("install_path").and_then(|v| v.as_str()).map(String::from),
                    bin_link: item.get("bin_link").and_then(|v| v.as_str()).map(String::from),
                    desktop_file: item.get("desktop_file").and_then(|v| v.as_str()).map(String::from),
                });
            }
        }
        Self { path, apps }
    }

    pub fn find(&self, name: &str) -> Option<&RegistryEntry> {
        self.apps.iter().find(|a| a.name == name)
    }

    pub fn add(&mut self, entry: RegistryEntry) {
        self.apps.retain(|a| a.name != entry.name);
        self.apps.push(entry);
        self.save();
    }

    pub fn remove(&mut self, name: &str) -> Option<RegistryEntry> {
        let idx = self.apps.iter().position(|a| a.name == name)?;
        let entry = self.apps.remove(idx);
        self.save();
        Some(entry)
    }

    pub fn save(&self) {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let mut out = String::from("# univ registry\n");
        for a in &self.apps {
            out.push_str(&format!("\n[[apps]]\nname = {:?}\nsource = {:?}\n", a.name, a.source));
            if let Some(p) = &a.install_path {
                out.push_str(&format!("install_path = {:?}\n", p));
            }
            if let Some(b) = &a.bin_link {
                out.push_str(&format!("bin_link = {:?}\n", b));
            }
            if let Some(d) = &a.desktop_file {
                out.push_str(&format!("desktop_file = {:?}\n", d));
            }
        }
        fs::write(&self.path, out).ok();
    }
}

fn default_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("univ/registry.toml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn registry_roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "univ_reg_{}_{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("registry.toml");

        let mut reg = Registry { path: path.clone(), apps: vec![] };
        reg.add(RegistryEntry {
            name: "myapp".into(),
            source: "tar".into(),
            install_path: Some("/opt/myapp".into()),
            bin_link: Some("/usr/local/bin/myapp".into()),
            desktop_file: Some("/usr/local/share/applications/myapp.desktop".into()),
        });

        let loaded = Registry::load_from(path);
        assert_eq!(loaded.apps.len(), 1);
        assert_eq!(loaded.apps[0].name, "myapp");
        assert_eq!(loaded.apps[0].install_path.as_deref(), Some("/opt/myapp"));

        let removed = Registry::load_from(loaded.path.clone()).remove("myapp");
        assert!(removed.is_some());
        assert!(Registry::load_from(loaded.path).apps.is_empty());

        fs::remove_dir_all(&dir).unwrap();
    }
}
