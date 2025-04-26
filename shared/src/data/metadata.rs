use std::fs::DirEntry;
use std::os::unix::fs::PermissionsExt;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use crate::data::metadata::MetadataKind::{Dir, File, Other, Symlink};

#[derive(Serialize, Deserialize, Clone)]
pub enum MetadataKind {
    File,
    Dir,
    Symlink,
    Other,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EntryMetadata {
    pub name: String,
    pub size: u64,
    pub kind: MetadataKind,
    pub permissions: u32,
}

impl Default for EntryMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl EntryMetadata {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            size: 0,
            kind: Other,
            permissions: 0,
        }
    }

    pub fn with_name(name: String) -> Self {
        Self {
            name,
            ..Self::default()
        }
    }
    pub fn to_pretty_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
    }
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &[u8]) -> Result<Self> {
        serde_json::from_str(String::from_utf8_lossy(json).as_ref())
    }
}

// impl Debug for EntryMetadata {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "EntryMetadata {{ path: {}, size: {}, kind: {}, permissions: {} }}", self.path, self.size, self.kind, self.permissions)
//     }
// }

impl From<DirEntry> for EntryMetadata {
    fn from(item: DirEntry) -> Self {
        let mut entry = Self::with_name(item.file_name().to_string_lossy().to_string());
        if let Ok(meta) = item.metadata() {
            if meta.is_dir() {
                entry.kind = Dir;
            } else if meta.is_file() {
                entry.kind = File;
            } else if meta.is_symlink() {
                entry.kind = Symlink;
            } else {
                entry.kind = Other;
            }
            entry.size = meta.len();
            entry.permissions = meta.permissions().mode();
        }
        entry
    }
}
