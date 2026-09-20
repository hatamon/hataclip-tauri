use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const CURRENT_VERSION: u32 = 1;
const MAX_ITEMS: usize = 500;
const MAX_CONTEXTS: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub pinned: bool,
    /// 貼り付け先アプリの目印。新しい順。
    #[serde(default)]
    pub contexts: Vec<String>,
}

impl Item {
    pub fn new(text: String, tags: Vec<String>) -> Self {
        Self {
            id: new_id(),
            text,
            tags,
            pinned: false,
            contexts: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct StoreFile {
    version: u32,
    items: Vec<Item>,
}

pub struct Store {
    path: PathBuf,
    items: Vec<Item>,
    undo: Vec<(usize, Item)>,
}

impl Store {
    pub fn load(path: PathBuf) -> Self {
        let items = read_items(&path);
        Self {
            path,
            items,
            undo: Vec::new(),
        }
    }

    pub fn list(&self) -> &[Item] {
        &self.items
    }

    pub fn get(&self, id: &str) -> Option<&Item> {
        self.items.iter().find(|item| item.id == id)
    }

    pub fn insert(&mut self, item: Item) {
        if let Some(index) = self
            .items
            .iter()
            .position(|existing| existing.text == item.text)
        {
            let existing = self.items.remove(index);
            self.items.insert(0, existing);
            self.save();
            return;
        }
        self.insert_at(0, item);
    }

    /// 指定した行のすぐ上か下に差し込む。同じ本文でもまとめない。
    pub fn insert_relative(&mut self, anchor: Option<&str>, above: bool, item: Item) {
        let at = anchor
            .and_then(|id| self.items.iter().position(|item| item.id == id))
            .map(|index| if above { index } else { index + 1 })
            .unwrap_or(0);
        self.insert_at(at, item);
    }

    fn insert_at(&mut self, index: usize, item: Item) {
        let index = index.min(self.items.len());
        self.items.insert(index, item);
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

    pub fn delete_many(&mut self, ids: &[String]) -> bool {
        let mut removed = Vec::new();
        let mut index = 0;
        self.items.retain(|item| {
            let keep = !ids.iter().any(|id| id == &item.id);
            if !keep {
                removed.push((index, item.clone()));
            }
            index += 1;
            keep
        });
        if removed.is_empty() {
            return false;
        }
        self.undo = removed;
        self.save();
        true
    }

    /// 直前の削除だけ元に戻す。戻す先は削除前の位置。
    pub fn undo_delete(&mut self) -> bool {
        if self.undo.is_empty() {
            return false;
        }
        for (index, item) in std::mem::take(&mut self.undo) {
            let index = index.min(self.items.len());
            self.items.insert(index, item);
        }
        self.save();
        true
    }

    pub fn set_tag(&mut self, ids: &[String], tag: &str, add: bool) -> bool {
        let tag = tag.trim();
        if tag.is_empty() {
            return false;
        }
        let mut changed = false;
        for item in self.items.iter_mut() {
            if !ids.iter().any(|id| id == &item.id) {
                continue;
            }
            let has = item.tags.iter().any(|entry| entry == tag);
            if add && !has {
                item.tags.push(tag.to_string());
                changed = true;
            } else if !add && has {
                item.tags.retain(|entry| entry != tag);
                changed = true;
            }
        }
        if changed {
            self.save();
        }
        changed
    }

    pub fn set_pinned(&mut self, ids: &[String], pinned: bool) -> bool {
        let mut changed = false;
        for item in self.items.iter_mut() {
            if ids.iter().any(|id| id == &item.id) && item.pinned != pinned {
                item.pinned = pinned;
                changed = true;
            }
        }
        if changed {
            self.save();
        }
        changed
    }

    pub fn record_context(&mut self, ids: &[String], key: &str) {
        if key.is_empty() {
            return;
        }
        let mut changed = false;
        for item in self.items.iter_mut() {
            if !ids.iter().any(|id| id == &item.id) {
                continue;
            }
            item.contexts.retain(|entry| entry != key);
            item.contexts.insert(0, key.to_string());
            item.contexts.truncate(MAX_CONTEXTS);
            changed = true;
        }
        if changed {
            self.save();
        }
    }

    fn save(&self) {
        let _ = write_items(&self.path, &self.items);
    }
}

/// 一覧に出す順番。ピン留め、次に同じ貼り付け先で使ったもの、あとは新しい順のまま。
pub fn ordered(items: &[Item], context: Option<&str>) -> Vec<Item> {
    let mut pinned = Vec::new();
    let mut same_context = Vec::new();
    let mut rest = Vec::new();
    for item in items {
        if item.pinned {
            pinned.push(item.clone());
        } else if context.is_some_and(|key| item.contexts.iter().any(|entry| entry == key)) {
            same_context.push(item.clone());
        } else {
            rest.push(item.clone());
        }
    }
    pinned.append(&mut same_context);
    pinned.append(&mut rest);
    pinned
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

    fn item(id: &str, text: &str) -> Item {
        Item {
            id: id.into(),
            text: text.into(),
            tags: Vec::new(),
            pinned: false,
            contexts: Vec::new(),
        }
    }

    fn fresh(name: &str) -> Store {
        let path = temp_path(name);
        let _ = fs::remove_file(&path);
        Store::load(path)
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
            tags: vec!["t".into()],
            ..item("a", "hello")
        });
        assert_eq!(store.list().len(), 1);
        assert!(store.update("a", "hello2".into(), vec!["u".into()]));
        assert_eq!(store.list()[0].text, "hello2");
        assert!(store.delete_many(&["a".to_string()]));
        assert!(store.list().is_empty());

        let reloaded = Store::load(path.clone());
        assert!(reloaded.list().is_empty());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn insert_promotes_same_text_to_front() {
        let mut store = fresh("promote");
        store.insert(Item {
            tags: vec!["old".into()],
            pinned: true,
            ..item("a", "first")
        });
        store.insert(item("b", "second"));
        store.insert(item("c", "first"));
        assert_eq!(store.list().len(), 2);
        assert_eq!(store.list()[0].id, "a");
        assert_eq!(store.list()[0].tags, vec!["old"]);
        assert!(store.list()[0].pinned);
        assert_eq!(store.list()[1].id, "b");
    }

    #[test]
    fn insert_relative_puts_copy_next_to_the_anchor() {
        let mut store = fresh("relative");
        store.insert(item("a", "one"));
        store.insert(item("b", "two"));
        // ["b", "a"]
        store.insert_relative(Some("b"), false, item("c", "two"));
        assert_eq!(
            store
                .list()
                .iter()
                .map(|i| i.id.as_str())
                .collect::<Vec<_>>(),
            vec!["b", "c", "a"]
        );
        store.insert_relative(Some("a"), true, item("d", "two"));
        assert_eq!(
            store
                .list()
                .iter()
                .map(|i| i.id.as_str())
                .collect::<Vec<_>>(),
            vec!["b", "c", "d", "a"]
        );
    }

    #[test]
    fn undo_puts_deleted_rows_back_where_they_were() {
        let mut store = fresh("undo");
        store.insert(item("c", "three"));
        store.insert(item("b", "two"));
        store.insert(item("a", "one"));
        // ["a", "b", "c"]
        assert!(store.delete_many(&["a".to_string(), "c".to_string()]));
        assert_eq!(
            store
                .list()
                .iter()
                .map(|i| i.id.as_str())
                .collect::<Vec<_>>(),
            vec!["b"]
        );
        assert!(store.undo_delete());
        assert_eq!(
            store
                .list()
                .iter()
                .map(|i| i.id.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
        assert!(!store.undo_delete());
    }

    #[test]
    fn tags_are_added_and_removed_without_duplicates() {
        let mut store = fresh("tags");
        store.insert(item("b", "two"));
        store.insert(item("a", "one"));
        let ids = vec!["a".to_string(), "b".to_string()];
        assert!(store.set_tag(&ids, "work", true));
        assert!(!store.set_tag(&ids, "work", true));
        assert_eq!(store.get("a").unwrap().tags, vec!["work"]);
        assert!(store.set_tag(&["a".to_string()], "work", false));
        assert!(store.get("a").unwrap().tags.is_empty());
        assert_eq!(store.get("b").unwrap().tags, vec!["work"]);
        assert!(!store.set_tag(&ids, "  ", true));
    }

    #[test]
    fn contexts_keep_the_newest_first() {
        let mut store = fresh("contexts");
        store.insert(item("a", "one"));
        let ids = vec!["a".to_string()];
        store.record_context(&ids, "code");
        store.record_context(&ids, "chrome|GitHub");
        store.record_context(&ids, "code");
        assert_eq!(
            store.get("a").unwrap().contexts,
            vec!["code", "chrome|GitHub"]
        );
    }

    #[test]
    fn ordering_puts_pins_then_the_same_context_first() {
        let items = vec![
            item("a", "one"),
            Item {
                contexts: vec!["code".into()],
                ..item("b", "two")
            },
            Item {
                pinned: true,
                ..item("c", "three")
            },
        ];
        let ordered = ordered(&items, Some("code"));
        assert_eq!(
            ordered.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
            vec!["c", "b", "a"]
        );
        let without = ordered_ids(&items, None);
        assert_eq!(without, vec!["c", "a", "b"]);
    }

    fn ordered_ids(items: &[Item], context: Option<&str>) -> Vec<String> {
        ordered(items, context)
            .iter()
            .map(|item| item.id.clone())
            .collect()
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
        assert!(!store.list()[0].pinned);
        let _ = fs::remove_file(path);
    }
}
