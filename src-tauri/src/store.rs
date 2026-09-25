use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
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
    /// ピン留め同士の手動順。小さいほど上。
    #[serde(default)]
    pub pin_rank: i32,
    /// `| add` か `:!!sh` で残した式。`g:` でもう一度実行する。
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub formula: String,
}

impl Item {
    pub fn new(text: String, tags: Vec<String>) -> Self {
        Self {
            id: new_id(),
            text,
            tags,
            pinned: false,
            contexts: Vec::new(),
            pin_rank: 0,
            formula: String::new(),
        }
    }

    pub fn locked(&self) -> bool {
        self.tags.iter().any(|tag| tag == "lock")
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct StoreFile {
    version: u32,
    items: Vec<Item>,
}

const MAX_UNDO: usize = 1;

pub struct Store {
    path: PathBuf,
    items: Vec<Item>,
    undo: Vec<Vec<Item>>,
    redo: Vec<Vec<Item>>,
    /// 読み込みまたは保存したときのファイル時刻。これより新しければ外から書かれている。
    disk_ns: Cell<u128>,
    /// 直前に読み書きした id。これ以外でファイルにだけある行は、保存時に残す。
    baseline: RefCell<HashSet<String>>,
}

impl Store {
    pub fn load(path: PathBuf) -> Self {
        let items = read_items(&path);
        let before = items.len();
        let mut store = Self {
            path,
            items,
            undo: Vec::new(),
            redo: Vec::new(),
            disk_ns: Cell::new(0),
            baseline: RefCell::new(HashSet::new()),
        };
        store.note_disk();
        store.note_baseline();
        if store.items.len() != before {
            store.save();
        }
        store
    }

    /// ファイルが新しければメモリをそちらに合わせる。取り消しは捨てる。
    fn absorb(&mut self) {
        self.reload_if_newer();
    }

    /// パイプの `add` などでファイルが新しければ、一覧を開き直したときに取り込む。
    pub fn reload_if_newer(&mut self) -> bool {
        let newer = mtime_ns(&self.path);
        if newer <= self.disk_ns.get() {
            return false;
        }
        let items = read_items(&self.path);
        let before = items.len();
        self.items = items;
        self.undo.clear();
        self.redo.clear();
        self.disk_ns.set(newer);
        self.note_baseline();
        if self.items.len() != before {
            self.save();
        }
        true
    }

    fn note_disk(&self) {
        self.disk_ns.set(mtime_ns(&self.path));
    }

    pub fn list(&self) -> &[Item] {
        &self.items
    }

    pub fn get(&self, id: &str) -> Option<&Item> {
        self.items.iter().find(|item| item.id == id)
    }

    pub fn insert(&mut self, item: Item) {
        self.absorb();
        self.push_undo();
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

    /// ドロップしたパスを先頭へ。1 回のドロップは 1 段の取り消し。
    pub fn drop_paths(&mut self, paths: &[String]) -> bool {
        self.absorb();
        let paths: Vec<String> = paths
            .iter()
            .map(|path| path.trim().to_string())
            .filter(|path| !path.is_empty())
            .collect();
        if paths.is_empty() {
            return false;
        }
        self.push_undo();
        for path in paths.into_iter().rev() {
            if let Some(index) = self.items.iter().position(|item| item.text == path) {
                let existing = self.items.remove(index);
                self.items.insert(0, existing);
            } else {
                self.items.insert(
                    0,
                    Item::new(path, vec!["path".into(), "file".into()]),
                );
                if self.items.len() > MAX_ITEMS {
                    self.items.truncate(MAX_ITEMS);
                }
            }
        }
        self.save();
        true
    }

    /// 指定した行のすぐ上か下に差し込む。同じ本文でもまとめない。
    pub fn insert_relative(&mut self, anchor: Option<&str>, above: bool, mut item: Item) {
        self.absorb();
        self.push_undo();
        if item.pinned {
            if let Some(id) = anchor {
                item.pin_rank = self.pin_rank_beside(id, above);
            }
        }
        let at = anchor
            .and_then(|id| self.items.iter().position(|item| item.id == id))
            .map(|index| if above { index } else { index + 1 })
            .unwrap_or(0);
        self.insert_at(at, item);
    }

    fn pin_rank_beside(&mut self, anchor_id: &str, above: bool) -> i32 {
        let anchor_rank = self.get(anchor_id).map(|item| item.pin_rank).unwrap_or(0);
        if above {
            let prev = self
                .items
                .iter()
                .filter(|item| item.pinned && item.pin_rank < anchor_rank)
                .map(|item| item.pin_rank)
                .max();
            match prev {
                None => anchor_rank - 1,
                Some(prev) if prev + 1 < anchor_rank => prev + 1,
                Some(_) => {
                    for item in &mut self.items {
                        if item.pinned && item.pin_rank < anchor_rank {
                            item.pin_rank -= 1;
                        }
                    }
                    anchor_rank - 1
                }
            }
        } else {
            let next = self
                .items
                .iter()
                .filter(|item| item.pinned && item.pin_rank > anchor_rank)
                .map(|item| item.pin_rank)
                .min();
            match next {
                None => anchor_rank + 1,
                Some(next) if anchor_rank + 1 < next => anchor_rank + 1,
                Some(_) => {
                    for item in &mut self.items {
                        if item.pinned && item.pin_rank > anchor_rank {
                            item.pin_rank += 1;
                        }
                    }
                    anchor_rank + 1
                }
            }
        }
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
        self.absorb();
        if self.items.iter().any(|item| item.id == id) {
            self.push_undo();
            if let Some(item) = self.items.iter_mut().find(|item| item.id == id) {
                item.text = text;
                item.tags = tags;
            }
            self.save();
            true
        } else {
            false
        }
    }

    pub fn delete_many(&mut self, ids: &[String]) -> bool {
        self.absorb();
        let ids: Vec<String> = ids
            .iter()
            .filter(|id| self.get(id).is_some_and(|item| !item.locked()))
            .cloned()
            .collect();
        if ids.is_empty() {
            return false;
        }
        if !self.items.iter().any(|item| ids.iter().any(|id| id == &item.id)) {
            return false;
        }
        self.push_undo();
        self.remove_ids(&ids);
        true
    }

    /// `#once` のように、取り消しスタックに残さない削除。
    pub fn remove_ids(&mut self, ids: &[String]) {
        self.absorb();
        self.items.retain(|item| !ids.iter().any(|id| id == &item.id));
        self.save();
    }

    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.undo.pop() else {
            return false;
        };
        self.redo.push(self.items.clone());
        self.items = previous;
        self.save();
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo.pop() else {
            return false;
        };
        self.undo.push(self.items.clone());
        if self.undo.len() > MAX_UNDO {
            self.undo.remove(0);
        }
        self.items = next;
        self.save();
        true
    }

    pub fn set_tag(&mut self, ids: &[String], tag: &str, add: bool) -> bool {
        self.absorb();
        let tag = tag.trim();
        if tag.is_empty() {
            return false;
        }
        let mut changed = false;
        for item in self.items.iter() {
            if !ids.iter().any(|id| id == &item.id) {
                continue;
            }
            let has = item.tags.iter().any(|entry| entry == tag);
            if (add && !has) || (!add && has) {
                changed = true;
                break;
            }
        }
        if !changed {
            return false;
        }
        self.push_undo();
        for item in self.items.iter_mut() {
            if !ids.iter().any(|id| id == &item.id) {
                continue;
            }
            let has = item.tags.iter().any(|entry| entry == tag);
            if add && !has {
                item.tags.push(tag.to_string());
            } else if !add && has {
                item.tags.retain(|entry| entry != tag);
            }
        }
        self.save();
        true
    }

    /// 空白で分けた語を、取り消し1回で付け外しする。空の語は捨てる。
    pub fn set_tags(&mut self, ids: &[String], tags: &[String], add: bool) -> bool {
        self.absorb();
        let mut words = Vec::new();
        for tag in tags {
            let tag = tag.trim();
            if tag.is_empty() || words.iter().any(|word| word == tag) {
                continue;
            }
            words.push(tag.to_string());
        }
        if words.is_empty() {
            return false;
        }
        let touches = |item: &Item, tag: &str| {
            let has = item.tags.iter().any(|entry| entry == tag);
            (add && !has) || (!add && has)
        };
        let changed = self.items.iter().any(|item| {
            ids.iter().any(|id| id == &item.id) && words.iter().any(|tag| touches(item, tag))
        });
        if !changed {
            return false;
        }
        self.push_undo();
        for item in self.items.iter_mut() {
            if !ids.iter().any(|id| id == &item.id) {
                continue;
            }
            for tag in &words {
                let has = item.tags.iter().any(|entry| entry == tag);
                if add && !has {
                    item.tags.push(tag.clone());
                } else if !add && has {
                    item.tags.retain(|entry| entry != tag);
                }
            }
        }
        self.save();
        true
    }

    pub fn set_app_tag(&mut self, ids: &[String], app: &str) -> bool {
        self.absorb();
        let app = app.trim().to_lowercase();
        if app.is_empty() || ids.is_empty() {
            return false;
        }
        let tag = format!("app:{app}");
        let mut changed = false;
        for item in self.items.iter() {
            if !ids.iter().any(|id| id == &item.id) {
                continue;
            }
            let has = item.tags.iter().any(|entry| entry == &tag);
            let extras = item.tags.iter().any(|entry| {
                crate::text::tag_arg(entry, "app").is_some() && entry != &tag
            });
            if extras || !has {
                changed = true;
                break;
            }
        }
        if !changed {
            return false;
        }
        self.push_undo();
        for item in self.items.iter_mut() {
            if !ids.iter().any(|id| id == &item.id) {
                continue;
            }
            item.tags
                .retain(|entry| crate::text::tag_arg(entry, "app").is_none());
            item.tags.push(tag.clone());
        }
        self.save();
        true
    }

    pub fn set_pinned(&mut self, ids: &[String], pinned: bool) -> bool {
        self.absorb();
        let mut changed = false;
        for item in self.items.iter() {
            if ids.iter().any(|id| id == &item.id) && item.pinned != pinned {
                changed = true;
                break;
            }
        }
        if !changed {
            return false;
        }
        self.push_undo();
        let mut rank = self
            .items
            .iter()
            .filter(|item| item.pinned)
            .map(|item| item.pin_rank)
            .min()
            .unwrap_or(0);
        for item in self.items.iter_mut() {
            if ids.iter().any(|id| id == &item.id) && item.pinned != pinned {
                item.pinned = pinned;
                if pinned {
                    rank -= 1;
                    item.pin_rank = rank;
                } else {
                    item.pin_rank = 0;
                }
            }
        }
        self.save();
        true
    }

    pub fn record_context(&mut self, ids: &[String], key: &str) {
        self.absorb();
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

    pub fn clear_unpinned(&mut self) -> bool {
        self.absorb();
        if self.items.iter().all(|item| item.pinned || item.locked()) {
            return false;
        }
        self.push_undo();
        self.items.retain(|item| item.pinned || item.locked());
        self.save();
        true
    }

    pub fn move_pins(&mut self, ids: &[String], delta: i32) -> bool {
        self.absorb();
        if ids.is_empty() || delta == 0 {
            return false;
        }
        let pins = ids
            .iter()
            .all(|id| self.get(id).map_or(false, |item| item.pinned));
        let rest = ids
            .iter()
            .all(|id| self.get(id).map_or(false, |item| !item.pinned));
        if !pins && !rest {
            return false;
        }
        let mut order: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.pinned == pins)
            .map(|(index, _)| index)
            .collect();
        if pins {
            order.sort_by(|&a, &b| self.items[a].pin_rank.cmp(&self.items[b].pin_rank));
        }
        let selected: Vec<usize> = order
            .iter()
            .enumerate()
            .filter(|(_, &index)| ids.iter().any(|id| id == &self.items[index].id))
            .map(|(pos, _)| pos)
            .collect();
        if selected.is_empty() {
            return false;
        }
        let first = selected[0];
        let last = selected[selected.len() - 1];
        if last - first + 1 != selected.len() {
            return false;
        }
        if delta < 0 && first == 0 {
            return false;
        }
        if delta > 0 && last + 1 >= order.len() {
            return false;
        }
        self.push_undo();
        let block: Vec<usize> = order.drain(first..=last).collect();
        if delta < 0 {
            let at = first - 1;
            for index in block.into_iter().rev() {
                order.insert(at, index);
            }
        } else {
            for (offset, index) in block.into_iter().enumerate() {
                order.insert(first + 1 + offset, index);
            }
        }
        if pins {
            for (rank, &index) in order.iter().enumerate() {
                self.items[index].pin_rank = rank as i32;
            }
        } else {
            let slots: Vec<usize> = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| !item.pinned)
                .map(|(index, _)| index)
                .collect();
            let moved: Vec<Item> = order.iter().map(|&index| self.items[index].clone()).collect();
            for (slot, item) in slots.into_iter().zip(moved) {
                self.items[slot] = item;
            }
        }
        self.save();
        true
    }

    pub fn split_items(&mut self, ids: &[String]) -> bool {
        self.absorb();
        let targets: Vec<Item> = ids
            .iter()
            .filter_map(|id| self.get(id).cloned())
            .filter(|item| item.text.contains('\n'))
            .collect();
        if targets.is_empty() {
            return false;
        }
        self.push_undo();
        for item in targets.into_iter().rev() {
            let Some(index) = self.items.iter().position(|entry| entry.id == item.id) else {
                continue;
            };
            self.items.remove(index);
            let mut at = index;
            for part in item.text.split('\n') {
                let next = Item::new(part.to_string(), item.tags.clone());
                self.items.insert(at, next);
                at += 1;
            }
        }
        if self.items.len() > MAX_ITEMS {
            self.items.truncate(MAX_ITEMS);
        }
        self.save();
        true
    }

    pub fn import_markdown(&mut self, markdown: &str) -> bool {
        self.absorb();
        let parsed = parse_export(markdown);
        if parsed.is_empty() {
            return false;
        }
        self.push_undo();
        for (text, tags, pinned) in parsed.into_iter().rev() {
            if let Some(index) = self.items.iter().position(|item| item.text == text) {
                let mut existing = self.items.remove(index);
                existing.tags = tags;
                existing.pinned = pinned;
                if pinned {
                    existing.pin_rank = existing.pin_rank.min(-1);
                } else {
                    existing.pin_rank = 0;
                }
                self.items.insert(0, existing);
            } else {
                let mut item = Item::new(text, tags);
                item.pinned = pinned;
                if pinned {
                    item.pin_rank = self
                        .items
                        .iter()
                        .filter(|entry| entry.pinned)
                        .map(|entry| entry.pin_rank)
                        .min()
                        .unwrap_or(0)
                        - 1;
                }
                self.insert_at(0, item);
            }
        }
        self.save();
        true
    }

    pub fn merge_items(&mut self, ids: &[String]) -> bool {
        self.absorb();
        if ids.len() < 2 {
            return false;
        }
        let rows: Vec<Item> = ids
            .iter()
            .filter_map(|id| self.get(id).cloned())
            .collect();
        if rows.len() < 2 {
            return false;
        }
        if rows.iter().any(|item| item.locked()) {
            return false;
        }
        self.push_undo();
        let Some(index) = self.items.iter().position(|item| item.id == rows[0].id) else {
            return false;
        };
        let mut tags = Vec::new();
        for item in &rows {
            for tag in &item.tags {
                if !tags.iter().any(|entry| entry == tag) {
                    tags.push(tag.clone());
                }
            }
        }
        let pinned = rows.iter().any(|item| item.pinned);
        let pin_rank = rows
            .iter()
            .filter(|item| item.pinned)
            .map(|item| item.pin_rank)
            .min()
            .unwrap_or(0);
        let text = rows
            .iter()
            .map(|item| item.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let drop: Vec<String> = rows.iter().map(|item| item.id.clone()).collect();
        self.items
            .retain(|item| !drop.iter().any(|id| id == &item.id));
        let mut merged = Item::new(text, tags);
        merged.pinned = pinned;
        merged.pin_rank = pin_rank;
        let at = index.min(self.items.len());
        self.items.insert(at, merged);
        self.save();
        true
    }

    pub fn clone_items(&mut self, ids: &[String]) -> bool {
        self.absorb();
        let rows: Vec<(usize, Item)> = ids
            .iter()
            .filter_map(|id| {
                self.items
                    .iter()
                    .position(|item| item.id == *id)
                    .map(|index| (index, self.items[index].clone()))
            })
            .collect();
        if rows.is_empty() {
            return false;
        }
        self.push_undo();
        for (index, src) in rows.into_iter().rev() {
            let mut copy = Item::new(src.text, src.tags);
            copy.pinned = src.pinned;
            copy.pin_rank = src.pin_rank;
            self.items.insert((index + 1).min(self.items.len()), copy);
        }
        if self.items.len() > MAX_ITEMS {
            self.items.truncate(MAX_ITEMS);
        }
        self.save();
        true
    }

    pub fn substitute(&mut self, ids: &[String], old: &str, new: &str) -> bool {
        self.absorb();
        if old.is_empty() || ids.is_empty() {
            return false;
        }
        let mut changed = false;
        for item in self.items.iter() {
            if ids.iter().any(|id| id == &item.id) && item.text.contains(old) {
                changed = true;
                break;
            }
        }
        if !changed {
            return false;
        }
        self.push_undo();
        for item in self.items.iter_mut() {
            if ids.iter().any(|id| id == &item.id) {
                item.text = item.text.replace(old, new);
            }
        }
        self.save();
        true
    }

    /// 選んだ行の本文を置き換える。タグとピンはそのまま。
    pub fn replace_texts(&mut self, updates: &[(String, String)]) -> bool {
        self.absorb();
        if updates.is_empty() {
            return false;
        }
        let mut changed = false;
        for (id, text) in updates {
            let Some(item) = self.get(id) else {
                return false;
            };
            if item.text != *text {
                changed = true;
            }
        }
        if !changed {
            return false;
        }
        self.push_undo();
        for (id, text) in updates {
            if let Some(item) = self.items.iter_mut().find(|item| item.id == *id) {
                item.text = text.clone();
            }
        }
        self.save();
        true
    }

    /// 本文を置き換え、同じ式を残す。本文も式も同じなら何もしない。
    pub fn rewrite_with_formula(&mut self, updates: &[(String, String)], formula: &str) -> bool {
        self.absorb();
        if updates.is_empty() {
            return false;
        }
        let mut changed = false;
        for (id, text) in updates {
            let Some(item) = self.get(id) else {
                return false;
            };
            if item.text != *text || item.formula != formula {
                changed = true;
            }
        }
        if !changed {
            return false;
        }
        self.push_undo();
        for (id, text) in updates {
            if let Some(item) = self.items.iter_mut().find(|item| item.id == *id) {
                item.text = text.clone();
                item.formula = formula.to_string();
            }
        }
        self.save();
        true
    }

    pub fn set_formula(&mut self, id: &str, formula: String) -> bool {
        self.absorb();
        let Some(item) = self.get(id) else {
            return false;
        };
        if item.formula == formula {
            return false;
        }
        self.push_undo();
        if let Some(item) = self.items.iter_mut().find(|item| item.id == id) {
            item.formula = formula;
        }
        self.save();
        true
    }

    /// `#grab` の1行目と2行目を入れ替える。タグの無い行と2行目の無い行は飛ばす。
    pub fn swap_grab(&mut self, ids: &[String]) -> bool {
        self.absorb();
        let updates: Vec<(String, String)> = ids
            .iter()
            .filter_map(|id| {
                let item = self.get(id)?;
                if !item.tags.iter().any(|tag| tag == "grab") {
                    return None;
                }
                let next = crate::text::swap_grab_lines(&item.text)?;
                (next != item.text).then(|| (id.clone(), next))
            })
            .collect();
        if updates.is_empty() {
            return false;
        }
        self.push_undo();
        for (id, text) in updates {
            if let Some(item) = self.items.iter_mut().find(|item| item.id == id) {
                item.text = text;
            }
        }
        self.save();
        true
    }

    /// 選んだ行を本文の順に並べ、範囲先頭の位置から置き直す。ピンとタグはそのまま。
    pub fn sort_items(&mut self, ids: &[String]) -> bool {
        self.absorb();
        if ids.len() < 2 {
            return false;
        }
        let mut rows: Vec<Item> = ids
            .iter()
            .filter_map(|id| self.get(id).cloned())
            .collect();
        if rows.len() < 2 {
            return false;
        }
        let first_id = &ids[0];
        let Some(first_index) = self.items.iter().position(|item| item.id == *first_id) else {
            return false;
        };
        let at = self.items[..first_index]
            .iter()
            .filter(|item| !ids.iter().any(|id| id == &item.id))
            .count();
        rows.sort_by(|a, b| a.text.cmp(&b.text));
        let mut next = self.items.clone();
        next.retain(|item| !ids.iter().any(|id| id == &item.id));
        for (offset, row) in rows.iter().enumerate() {
            next.insert(at + offset, row.clone());
        }
        if next
            .iter()
            .map(|item| item.id.as_str())
            .eq(self.items.iter().map(|item| item.id.as_str()))
        {
            return false;
        }
        self.push_undo();
        self.items = next;
        self.save();
        true
    }

    pub fn dedup(&mut self) -> bool {
        self.absorb();
        let mut keep: Vec<String> = Vec::new();
        let mut seen = std::collections::HashMap::<String, (bool, usize)>::new();
        for (index, item) in self.items.iter().enumerate() {
            let key = item.text.replace("\r\n", "\n").trim().to_string();
            match seen.get(&key) {
                None => {
                    seen.insert(key, (item.pinned || item.locked(), index));
                    keep.push(item.id.clone());
                }
                Some((pinned, prev)) => {
                    if self.items[*prev].locked() {
                        continue;
                    }
                    if (item.locked() || (item.pinned && !*pinned)) && !self.items[*prev].locked() {
                        keep.retain(|id| id != &self.items[*prev].id);
                        keep.push(item.id.clone());
                        seen.insert(key, (item.pinned || item.locked(), index));
                    }
                }
            }
        }
        if keep.len() == self.items.len() {
            return false;
        }
        self.push_undo();
        self.items
            .retain(|item| keep.iter().any(|id| id == &item.id));
        self.save();
        true
    }

    fn push_undo(&mut self) {
        self.undo.push(self.items.clone());
        if self.undo.len() > MAX_UNDO {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    fn save(&mut self) {
        self.keep_external();
        let _ = write_items(&self.path, &self.items);
        self.note_disk();
        self.note_baseline();
    }

    /// 開いたあとにパイプが足した行を、この保存で消さない。
    fn keep_external(&mut self) {
        let newer = mtime_ns(&self.path);
        if newer <= self.disk_ns.get() {
            return;
        }
        let disk = read_items(&self.path);
        let baseline = self.baseline.borrow();
        let have: HashSet<&str> = self.items.iter().map(|item| item.id.as_str()).collect();
        let external: Vec<Item> = disk
            .into_iter()
            .filter(|item| !have.contains(item.id.as_str()) && !baseline.contains(&item.id))
            .collect();
        drop(baseline);
        for item in external.into_iter().rev() {
            self.items.insert(0, item);
        }
    }

    fn note_baseline(&self) {
        *self.baseline.borrow_mut() = self.items.iter().map(|item| item.id.clone()).collect();
    }
}

fn mtime_ns(path: &Path) -> u128 {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

/// 一覧に出す順番。ピン留め（手動順）、それ以外はストアの順（新しいものが先）。
pub fn ordered(items: &[Item], context: Option<&str>) -> Vec<Item> {
    let mut pinned = Vec::new();
    let mut rest = Vec::new();
    for item in items {
        if !visible_app(item, context) {
            continue;
        }
        if !visible_not_app(item, context) {
            continue;
        }
        if item.pinned {
            pinned.push(item.clone());
        } else {
            rest.push(item.clone());
        }
    }
    pinned.sort_by(|a, b| a.pin_rank.cmp(&b.pin_rank));
    pinned.append(&mut rest);
    pinned
}

/// `#app:` で隠れる行でも、今いじっている行は一覧に残す。付けたタグが見えなくなるのを防ぐ。
pub fn ordered_keeping(items: &[Item], context: Option<&str>, keep: &[String]) -> Vec<Item> {
    let mut list = ordered(items, context);
    for id in keep {
        if list.iter().any(|item| item.id == *id) {
            continue;
        }
        if let Some(item) = items.iter().find(|item| item.id == *id) {
            list.push(item.clone());
        }
    }
    list
}

fn visible_app(item: &Item, context: Option<&str>) -> bool {
    let apps: Vec<_> = tagged_apps(&item.tags, "app");
    if apps.is_empty() {
        return true;
    }
    let Some(current) = context.map(crate::text::context_app) else {
        return false;
    };
    apps.iter().any(|app| app.eq_ignore_ascii_case(current))
}

fn visible_not_app(item: &Item, context: Option<&str>) -> bool {
    let denied: Vec<_> = tagged_apps(&item.tags, "not");
    if denied.is_empty() {
        return true;
    }
    let Some(current) = context.map(crate::text::context_app) else {
        return true;
    };
    !denied.iter().any(|app| app.eq_ignore_ascii_case(current))
}

fn tagged_apps<'a>(tags: &'a [String], name: &str) -> Vec<&'a str> {
    tags.iter()
        .filter_map(|tag| crate::text::tag_arg(tag, name))
        .collect()
}

pub fn new_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}")
}

#[cfg(test)]
fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
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

/// `:export` が出す Markdown を読み戻す。
pub fn parse_export(markdown: &str) -> Vec<(String, Vec<String>, bool)> {
    let mut items = Vec::new();
    let mut lines = markdown.lines().peekable();
    while let Some(line) = lines.next() {
        if !line.starts_with("## ") {
            continue;
        }
        let mut tags = Vec::new();
        let mut pinned = false;
        loop {
            match lines.peek().copied() {
                Some(next) if next.starts_with("tags:") => {
                    tags = next["tags:".len()..]
                        .split(',')
                        .map(|tag| tag.trim().to_string())
                        .filter(|tag| !tag.is_empty())
                        .collect();
                    lines.next();
                }
                Some(next) if next.trim() == "pinned: true" => {
                    pinned = true;
                    lines.next();
                }
                Some(next) if next.trim().is_empty() => {
                    lines.next();
                }
                Some(next) if next.trim() == "```" => {
                    lines.next();
                    let mut body = String::new();
                    for body_line in lines.by_ref() {
                        if body_line.trim() == "```" {
                            break;
                        }
                        if !body.is_empty() {
                            body.push('\n');
                        }
                        body.push_str(body_line);
                    }
                    items.push((body, tags, pinned));
                    break;
                }
                _ => break,
            }
        }
    }
    items
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
            pin_rank: 0,
            formula: String::new(),
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
    fn reload_picks_up_a_newer_file() {
        let mut store = fresh("reload");
        store.insert(item("a", "one"));
        let path = store.path.clone();
        {
            let mut other = Store::load(path.clone());
            other.insert(item("b", "two"));
        }
        let file = fs::File::options().write(true).open(&path).unwrap();
        file.set_modified(SystemTime::now() + std::time::Duration::from_secs(5))
            .unwrap();
        assert!(store.reload_if_newer());
        assert_eq!(store.list()[0].text, "two");
        assert!(!store.reload_if_newer());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn save_keeps_a_row_added_outside() {
        let mut store = fresh("merge");
        store.insert(item("a", "one"));
        let path = store.path.clone();
        {
            let mut other = Store::load(path.clone());
            other.insert(item("b", "two"));
        }
        let file = fs::File::options().write(true).open(&path).unwrap();
        file.set_modified(SystemTime::now() + std::time::Duration::from_secs(5))
            .unwrap();
        assert!(store.delete_many(&["a".to_string()]));
        assert_eq!(store.list().len(), 1);
        assert_eq!(store.list()[0].text, "two");
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
    fn lock_skips_delete_and_clear() {
        let mut store = fresh("lock");
        store.insert(Item {
            tags: vec!["lock".into()],
            ..item("a", "keep")
        });
        store.insert(item("b", "gone"));
        assert!(!store.delete_many(&["a".to_string()]));
        assert!(store.delete_many(&["a".to_string(), "b".to_string()]));
        assert_eq!(store.list().len(), 1);
        assert_eq!(store.list()[0].id, "a");
        store.insert(item("c", "also"));
        assert!(!store.merge_items(&["a".into(), "c".into()]));
        assert!(store.clear_unpinned());
        assert_eq!(store.list().len(), 1);
        assert_eq!(store.list()[0].id, "a");
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
        assert!(store.undo());
        assert_eq!(
            store
                .list()
                .iter()
                .map(|i| i.id.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
        assert!(store.redo());
        assert_eq!(
            store
                .list()
                .iter()
                .map(|i| i.id.as_str())
                .collect::<Vec<_>>(),
            vec!["b"]
        );
        assert!(!store.redo());
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
        assert!(store.set_tag(&["a".to_string()], "work", true));
        assert!(store.set_app_tag(&["a".to_string()], "chrome"));
        assert_eq!(store.get("a").unwrap().tags, vec!["work", "app:chrome"]);
        assert!(store.set_app_tag(&["a".to_string()], "Code"));
        assert_eq!(store.get("a").unwrap().tags, vec!["work", "app:code"]);
        assert!(!store.set_app_tag(&["a".to_string()], "code"));
    }

    #[test]
    fn set_tags_applies_each_word_in_one_undo() {
        let mut store = fresh("tag-words");
        store.insert(item("a", "one"));
        let ids = vec!["a".to_string()];
        assert!(store.set_tags(
            &ids,
            &["a".into(), "b".into(), " ".into(), "a".into(), "c".into()],
            true
        ));
        assert_eq!(store.get("a").unwrap().tags, vec!["a", "b", "c"]);
        assert!(store.set_tag(&ids, "a b c", true));
        assert_eq!(store.get("a").unwrap().tags, vec!["a", "b", "c", "a b c"]);
        assert!(store.set_tags(&ids, &["a".into(), "c".into(), "missing".into()], false));
        assert_eq!(store.get("a").unwrap().tags, vec!["b", "a b c"]);
        assert!(store.undo());
        assert_eq!(store.get("a").unwrap().tags, vec!["a", "b", "c", "a b c"]);
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
    fn ordering_puts_pins_then_store_order() {
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
        assert_eq!(ordered_ids(&items, Some("code")), vec!["c", "a", "b"]);
        assert_eq!(ordered_ids(&items, None), vec!["c", "a", "b"]);
    }

    #[test]
    fn load_keeps_tmp_tagged_rows() {
        let path = temp_path("tmp");
        fs::write(
            &path,
            r#"{"version":1,"items":[{"id":"1","text":"keep","tags":[]},{"id":"2","text":"gone","tags":["tmp"]}]}"#,
        )
        .unwrap();
        let store = Store::load(path.clone());
        assert_eq!(store.list().len(), 2);
        assert_eq!(store.list()[1].id, "2");
        let reloaded = Store::load(path.clone());
        assert_eq!(reloaded.list().len(), 2);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_keeps_locked_tmp_rows() {
        let path = temp_path("lock-tmp");
        fs::write(
            &path,
            r#"{"version":1,"items":[{"id":"1","text":"keep","tags":["tmp","lock"]}]}"#,
        )
        .unwrap();
        let store = Store::load(path.clone());
        assert_eq!(store.list().len(), 1);
        let _ = fs::remove_file(path);
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

    #[test]
    fn pin_rank_orders_pins() {
        let items = vec![
            Item {
                pinned: true,
                pin_rank: 1,
                ..item("a", "one")
            },
            Item {
                pinned: true,
                pin_rank: 0,
                ..item("b", "two")
            },
        ];
        assert_eq!(ordered_ids(&items, None), vec!["b", "a"]);
    }

    #[test]
    fn here_tag_no_longer_hides() {
        let items = vec![
            Item {
                tags: vec!["here".into()],
                contexts: vec!["code".into()],
                ..item("a", "one")
            },
            item("b", "two"),
        ];
        assert_eq!(ordered_ids(&items, Some("other")), vec!["a", "b"]);
        assert_eq!(ordered_ids(&items, None), vec!["a", "b"]);
    }

    #[test]
    fn app_tag_shows_only_that_process() {
        let items = vec![
            Item {
                tags: vec!["app:chrome".into()],
                ..item("a", "one")
            },
            item("b", "two"),
        ];
        assert_eq!(ordered_ids(&items, Some("chrome")), vec!["a", "b"]);
        assert_eq!(ordered_ids(&items, Some("chrome|GitHub")), vec!["a", "b"]);
        assert_eq!(ordered_ids(&items, Some("CHROME")), vec!["a", "b"]);
        assert_eq!(ordered_ids(&items, Some("code")), vec!["b"]);
        assert_eq!(ordered_ids(&items, None), vec!["b"]);
    }

    #[test]
    fn keeping_shows_app_tagged_row_with_its_tags() {
        let items = vec![
            Item {
                tags: vec!["url".into(), "app:chrome".into()],
                ..item("a", "one")
            },
            item("b", "two"),
        ];
        let kept = ordered_keeping(&items, Some("code"), &["a".into()]);
        assert_eq!(
            kept.iter().map(|item| item.id.as_str()).collect::<Vec<_>>(),
            vec!["b", "a"]
        );
        assert_eq!(kept[1].tags, vec!["url", "app:chrome"]);
    }

    #[test]
    fn app_tags_match_any() {
        let items = vec![Item {
            tags: vec!["app:chrome".into(), "app:code".into()],
            ..item("a", "one")
        }];
        assert_eq!(ordered_ids(&items, Some("chrome")), vec!["a"]);
        assert_eq!(ordered_ids(&items, Some("code")), vec!["a"]);
        assert_eq!(ordered_ids(&items, Some("notepad")), Vec::<String>::new());
    }

    #[test]
    fn app_and_not_both_apply() {
        let items = vec![Item {
            tags: vec!["app:chrome".into(), "not:chrome".into()],
            ..item("a", "one")
        }];
        assert_eq!(ordered_ids(&items, Some("chrome")), Vec::<String>::new());
        let items = vec![Item {
            tags: vec!["app:chrome".into(), "not:code".into()],
            ..item("a", "one")
        }];
        assert_eq!(ordered_ids(&items, Some("chrome")), vec!["a"]);
        assert_eq!(ordered_ids(&items, Some("code")), Vec::<String>::new());
    }

    #[test]
    fn move_pins_swaps_neighbors() {
        let mut store = fresh("pins");
        store.insert(Item {
            pinned: true,
            pin_rank: 0,
            ..item("a", "one")
        });
        store.insert(Item {
            pinned: true,
            pin_rank: 1,
            ..item("b", "two")
        });
        // insert puts at front: b then a. pin_rank 1 then 0 so ordered is a, b.
        assert!(store.move_pins(&["a".into()], 1));
        assert_eq!(ordered_ids(store.list(), None), vec!["b", "a"]);
        assert!(!store.move_pins(&["c".into()], 1));
        assert!(!store.move_pins(&["b".into()], -1));
    }

    #[test]
    fn move_unpinned_rows_stays_under_pins() {
        let mut store = fresh("rows");
        store.insert(item("c", "three"));
        store.insert(item("b", "two"));
        store.insert(Item {
            pinned: true,
            pin_rank: 0,
            ..item("p", "pin")
        });
        store.insert(Item {
            tags: vec!["lock".into()],
            ..item("a", "one")
        });
        assert_eq!(ordered_ids(store.list(), None), vec!["p", "a", "b", "c"]);
        assert!(!store.move_pins(&["a".into()], -1));
        assert!(store.move_pins(&["a".into()], 1));
        assert_eq!(ordered_ids(store.list(), None), vec!["p", "b", "a", "c"]);
        assert!(!store.move_pins(&["p".into(), "b".into()], 1));
        assert!(!store.move_pins(&["c".into()], 1));
    }

    #[test]
    fn split_makes_one_row_per_line() {
        let mut store = fresh("split");
        store.insert(Item {
            tags: vec!["work".into()],
            ..item("a", "one\ntwo")
        });
        assert!(store.split_items(&["a".to_string()]));
        assert_eq!(store.list().len(), 2);
        assert_eq!(store.list()[0].text, "one");
        assert_eq!(store.list()[1].text, "two");
        assert_eq!(store.list()[0].tags, vec!["work"]);
        assert!(store.get("a").is_none());
        assert!(!store.split_items(&[store.list()[0].id.clone()]));
    }

    #[test]
    fn import_updates_same_text_and_adds_new() {
        let mut store = fresh("import");
        store.insert(item("a", "hello"));
        let md = "# hataclip\n\n## hello\ntags: work\npinned: true\n\n```\nhello\n```\n\n## other\n\n```\nworld\n```\n";
        assert!(store.import_markdown(md));
        assert_eq!(store.list().len(), 2);
        assert_eq!(store.list()[0].text, "hello");
        assert_eq!(store.list()[0].tags, vec!["work"]);
        assert!(store.list()[0].pinned);
        assert_eq!(store.list()[1].text, "world");
    }

    #[test]
    fn merge_joins_text_and_unions_tags() {
        let mut store = fresh("merge");
        store.insert(Item {
            tags: vec!["a".into()],
            ..item("x", "one")
        });
        store.insert(Item {
            tags: vec!["b".into(), "a".into()],
            pinned: true,
            ..item("y", "two")
        });
        assert!(store.merge_items(&["y".into(), "x".into()]));
        assert_eq!(store.list().len(), 1);
        assert_eq!(store.list()[0].text, "two\none");
        assert_eq!(store.list()[0].tags, vec!["b", "a"]);
        assert!(store.list()[0].pinned);
    }

    #[test]
    fn clone_puts_a_copy_below() {
        let mut store = fresh("clone");
        store.insert(item("a", "one"));
        assert!(store.clone_items(&["a".into()]));
        assert_eq!(store.list().len(), 2);
        assert_eq!(store.list()[0].id, "a");
        assert_eq!(store.list()[1].text, "one");
        assert_ne!(store.list()[1].id, "a");
    }

    #[test]
    fn drop_paths_puts_file_rows_in_front() {
        let mut store = fresh("drop");
        store.insert(item("a", "keep"));
        assert!(store.drop_paths(&["C:\\a.txt".into(), "C:\\b.txt".into()]));
        assert_eq!(store.list()[0].text, "C:\\a.txt");
        assert_eq!(store.list()[1].text, "C:\\b.txt");
        assert_eq!(store.list()[0].tags, vec!["path", "file"]);
        store.undo();
        assert_eq!(store.list().len(), 1);
        assert_eq!(store.list()[0].id, "a");
    }

    #[test]
    fn substitute_replaces_all() {
        let mut store = fresh("sub");
        store.insert(item("a", "foo foo"));
        assert!(store.substitute(&["a".into()], "foo", "bar"));
        assert_eq!(store.get("a").unwrap().text, "bar bar");
        assert!(!store.substitute(&["a".into()], "", "x"));
    }

    #[test]
    fn replace_texts_updates_and_undoes() {
        let mut store = fresh("replace");
        store.insert(item("a", "old"));
        store.insert(item("b", "keep"));
        assert!(store.replace_texts(&[("a".into(), "new".into())]));
        assert_eq!(store.get("a").unwrap().text, "new");
        assert_eq!(store.get("b").unwrap().text, "keep");
        assert!(!store.replace_texts(&[("a".into(), "new".into())]));
        store.undo();
        assert_eq!(store.get("a").unwrap().text, "old");
        assert!(!store.replace_texts(&[]));
        assert!(!store.replace_texts(&[("missing".into(), "x".into())]));
    }

    #[test]
    fn dedup_keeps_pin_then_newer() {
        let mut store = fresh("dedup");
        store.insert(item("a", "  same\n"));
        store.insert(Item {
            pinned: true,
            ..item("b", "same")
        });
        store.insert(item("c", "other"));
        assert!(store.dedup());
        let ids: Vec<_> = store.list().iter().map(|i| i.id.as_str()).collect();
        assert_eq!(ids, vec!["c", "b"]);
    }

    #[test]
    fn sort_orders_selected_from_first_slot() {
        let mut store = fresh("sort");
        store.insert(item("a", "c-text"));
        store.insert(item("b", "a-text"));
        store.insert(item("c", "b-text"));
        assert!(store.sort_items(&["b".into(), "c".into(), "a".into()]));
        let texts: Vec<_> = store.list().iter().map(|item| item.text.as_str()).collect();
        assert_eq!(texts, vec!["a-text", "b-text", "c-text"]);
        assert!(!store.sort_items(&["b".into(), "c".into(), "a".into()]));
        store.undo();
        let texts: Vec<_> = store.list().iter().map(|item| item.text.as_str()).collect();
        assert_eq!(texts, vec!["b-text", "a-text", "c-text"]);
    }

    #[test]
    fn sort_keeps_tags_and_inserts_at_visual_first() {
        let mut store = fresh("sort-slot");
        store.insert(Item {
            tags: vec!["keep".into()],
            ..item("a", "aa")
        });
        store.insert(item("b", "zz"));
        store.insert(item("c", "mm"));
        assert!(store.sort_items(&["b".into(), "a".into()]));
        let ids: Vec<_> = store.list().iter().map(|item| item.id.as_str()).collect();
        assert_eq!(ids, vec!["c", "a", "b"]);
        assert_eq!(store.get("a").unwrap().tags, vec!["keep"]);
    }

    #[test]
    fn not_app_hides_that_process() {
        let items = vec![
            Item {
                tags: vec!["not:chrome".into()],
                ..item("a", "one")
            },
            item("b", "two"),
        ];
        assert_eq!(ordered_ids(&items, Some("chrome")), vec!["b"]);
        assert_eq!(ordered_ids(&items, Some("chrome|GitHub")), vec!["b"]);
        assert_eq!(ordered_ids(&items, Some("code")), vec!["a", "b"]);
        assert_eq!(ordered_ids(&items, None), vec!["a", "b"]);
    }

    #[test]
    fn bare_not_tag_no_longer_hides() {
        let items = vec![
            Item {
                tags: vec!["not".into()],
                contexts: vec!["code".into()],
                ..item("a", "one")
            },
            item("b", "two"),
        ];
        assert_eq!(ordered_ids(&items, Some("code")), vec!["a", "b"]);
    }

    #[test]
    fn load_keeps_expired_ttl() {
        let path = temp_path("ttl");
        let old = now_secs().saturating_sub(4000);
        fs::write(
            &path,
            format!(
                r#"{{"version":1,"items":[{{"id":"1","text":"gone","tags":["ttl:1h"],"created_at":{old}}},{{"id":"2","text":"keep","tags":["ttl:1h"],"created_at":{}}}]}}"#,
                now_secs()
            ),
        )
        .unwrap();
        let store = Store::load(path.clone());
        assert_eq!(store.list().len(), 2);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn formula_stays_when_the_body_changes() {
        let mut store = fresh("formula");
        store.insert(item("a", "user-name"));
        assert!(store.set_formula("a", "sel | kebab".into()));
        assert!(store.replace_texts(&[("a".into(), "http-response".into())]));
        assert_eq!(store.get("a").unwrap().text, "http-response");
        assert_eq!(store.get("a").unwrap().formula, "sel | kebab");
        assert!(store.rewrite_with_formula(
            &[("a".into(), "USER".into())],
            "!! tr a-z A-Z"
        ));
        assert_eq!(store.get("a").unwrap().formula, "!! tr a-z A-Z");
        store.undo();
        assert_eq!(store.get("a").unwrap().formula, "sel | kebab");
        assert_eq!(store.get("a").unwrap().text, "http-response");
    }
}
