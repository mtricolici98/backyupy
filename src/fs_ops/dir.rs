use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::HashSet,
    fmt::format,
    fs::{self, DirEntry, OpenOptions},
    iter::Once,
    path::Path,
    sync::{Arc, Mutex, OnceLock},
};

#[derive(Debug, PartialEq)]
pub enum FsEntryType {
    File,
    Folder,
    HiddenFile,
    ErrorFile,
}

#[derive(Debug)]
pub struct FsEntry {
    pub name: String,
    pub abs_path: String,
    pub error: Option<String>,
    pub ignored: bool,
    pub e_type: FsEntryType,
}

impl FsEntry {
    fn err_entry(err: String) -> Self {
        FsEntry {
            name: String::from("???"),
            abs_path: String::from("???"),
            error: Some(err),
            ignored: false,
            e_type: FsEntryType::ErrorFile,
        }
    }
    fn from_de_and_config(de: DirEntry, config: &BupyConfig) -> Self {
        let mut instance = Self::from(de);
        instance.ignored = config.ignore_list.contains(&instance.abs_path);
        return instance;
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct BupyConfig {
    pub ignore_list: HashSet<String>,
    pub preselected_drive: Option<String>,
    pub base_path: String,
}

impl BupyConfig {
    pub fn new(base_path: &str) -> Self {
        BupyConfig {
            ignore_list: HashSet::new(),
            preselected_drive: None,
            base_path: String::from(base_path),
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Path::new(&self.base_path);
        let path = path.join(".bupyconfig");
        let config_str = serde_json::to_string_pretty(self)?;

        fs::write(path, config_str).map_err(Error::new)
    }
}

impl From<DirEntry> for FsEntry {
    fn from(de: DirEntry) -> Self {
        let Ok(name) = de.file_name().into_string() else {
            return FsEntry::err_entry("Unparasable name".to_string());
        };

        let Ok(abs_path) = de.path().into_os_string().into_string() else {
            return FsEntry::err_entry("Unparasable path".to_string());
        };

        let Ok(ft) = de.file_type() else {
            return FsEntry::err_entry("Unknow file type".to_string());
        };

        return FsEntry {
            name,
            abs_path,
            error: None,
            ignored: false,
            e_type: if ft.is_dir() {
                FsEntryType::Folder
            } else {
                FsEntryType::File
            },
        };
    }
}

pub fn list_dir_own(path: &Path) -> Result<Vec<FsEntry>> {
    let dir_s = std::fs::read_dir(path)?;
    let config = get_config()
        .lock()
        .map_err(|_| anyhow::anyhow!("Faile to open config"))?;
    let mut tmp: Vec<FsEntry> = dir_s
        .map(|f| match f {
            Ok(de) => FsEntry::from_de_and_config(de, &config),
            Err(e) => FsEntry::err_entry(format!("{:?}", e)),
        })
        .collect();
    tmp.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(tmp)
}

static CONFIG: OnceLock<Arc<Mutex<BupyConfig>>> = OnceLock::<Arc<Mutex<BupyConfig>>>::new();

fn get_config_instance(base_path: &str) -> Arc<Mutex<BupyConfig>> {
    let Ok(config) = fs::read_to_string(Path::new(base_path).join(".bupyconfig")) else {
        return Arc::new(Mutex::new(BupyConfig::new(base_path)));
    };
    let Ok(config) = serde_json::from_str(&config) else {
        return Arc::new(Mutex::new(BupyConfig::new(base_path)));
    };
    return Arc::new(Mutex::new(config));
}

pub fn init_config(base_path: &str) {
    CONFIG.get_or_init(|| get_config_instance(base_path));
}

pub fn get_config() -> &'static Arc<Mutex<BupyConfig>> {
    CONFIG.get().unwrap()
}

pub fn mark_for_ignore(list: &mut Vec<FsEntry>, index: usize) -> Result<()> {
    let el = list.get_mut(index);
    if let Some(entry) = el {
        entry.ignored = !entry.ignored;
        let mut config = get_config()
            .lock()
            .map_err(|err| anyhow::anyhow!("configuration mutex is poisoned: {err}"))?;
        if entry.ignored {
            config.ignore_list.insert(entry.abs_path.clone());
        } else {
            config.ignore_list.remove(&entry.abs_path);
        }
        config.save()
    } else {
        Err(Error::msg("Failed to find listing to ignore"))
    }
}
