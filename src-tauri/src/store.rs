use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const CURRENT_VERSION: u32 = 1;
const MAX_ITEMS: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoreFile {
    version: u32,
    items: Vec<Item>,
}

pub struct Store {
    path: PathBuf,
    items: Vec<Item>,
}

impl Store {
    pub fn load(path: PathBuf) -> Self {
        let items = read_items(&path);
        Self { path, items }
    }

    pub fn list(&self) -> &[Item] {
        &self.items
    }

    pub fn insert(&mut self, item: Item) {
        self.items.insert(0, item);
        if self.items.len() > MAX_ITEMS {
            self.items.truncate(MAX_ITEMS);
        }
        self.save();
    }

    pub fn update(&mut self, id: &str, text: String, tags: Vec<String>) -> bool {
        if let Some(item) = self.items.iter_mut().find(|item| item.id == id) {
            item.text = text;
            item.tags = tags;
            self.save();
            true
        } else {
            false
        }
    }

    pub fn delete(&mut self, id: &str) -> bool {
        let before = self.items.len();
        self.items.retain(|item| item.id != id);
        if self.items.len() != before {
            self.save();
            true
        } else {
            false
        }
    }

    pub fn get(&self, id: &str) -> Option<&Item> {
        self.items.iter().find(|item| item.id == id)
    }

    fn save(&self) {
        let _ = write_items(&self.path, &self.items);
    }
}

pub fn new_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}")
}

fn read_items(path: &Path) -> Vec<Item> {
    let Ok(data) = fs::read_to_string(path) else {
        return Vec::new();
    };
    match serde_json::from_str::<StoreFile>(&data) {
        Ok(file) => file.items,
        Err(_) => Vec::new(),
    }
}

fn write_items(path: &Path, items: &[Item]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = StoreFile {
        version: CURRENT_VERSION,
        items: items.to_vec(),
    };
    fs::write(path, serde_json::to_string_pretty(&file)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "hataclip-{}-{}-{name}.json",
            std::process::id(),
            new_id()
        ))
    }

    #[test]
    fn broken_json_yields_empty() {
        let path = temp_path("broken");
        fs::write(&path, "{not json").unwrap();
        let store = Store::load(path.clone());
        assert!(store.list().is_empty());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn missing_file_yields_empty() {
        let path = temp_path("missing");
        let _ = fs::remove_file(&path);
        let store = Store::load(path);
        assert!(store.list().is_empty());
    }

    #[test]
    fn round_trip_insert_update_delete() {
        let path = temp_path("round");
        let _ = fs::remove_file(&path);
        let mut store = Store::load(path.clone());
        store.insert(Item {
            id: "a".into(),
            text: "hello".into(),
            tags: vec!["t".into()],
        });
        assert_eq!(store.list().len(), 1);
        assert!(store.update("a", "hello2".into(), vec!["u".into()]));
        assert_eq!(store.list()[0].text, "hello2");
        assert!(store.delete("a"));
        assert!(store.list().is_empty());

        let reloaded = Store::load(path.clone());
        assert!(reloaded.list().is_empty());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn extra_fields_are_ignored() {
        let path = temp_path("extra");
        fs::write(
            &path,
            r#"{"version":1,"items":[{"id":"1","text":"x","tags":[],"kind":"future"}]}"#,
        )
        .unwrap();
        let store = Store::load(path.clone());
        assert_eq!(store.list()[0].text, "x");
        let _ = fs::remove_file(path);
    }
}
