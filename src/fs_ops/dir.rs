use anyhow::Result;
use std::{fs::DirEntry, path::Path};

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
    let mut tmp: Vec<FsEntry> = dir_s
        .map(|f| match f {
            Ok(de) => FsEntry::from(de),
            Err(e) => FsEntry::err_entry(format!("{:?}", e)),
        })
        .collect();
    tmp.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(tmp)
}
