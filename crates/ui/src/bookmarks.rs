//! Bookmarks management.

use std::collections::HashMap;

/// Bookmark manager.
pub struct BookmarkManager {
    /// Root folder.
    root: BookmarkFolder,
    /// Bookmark bar folder.
    bookmark_bar: BookmarkFolder,
    /// Other bookmarks folder.
    other_bookmarks: BookmarkFolder,
    /// ID counter.
    id_counter: u64,
}

impl BookmarkManager {
    pub fn new() -> Self {
        Self {
            root: BookmarkFolder::new(0, "Root"),
            bookmark_bar: BookmarkFolder::new(1, "Bookmark Bar"),
            other_bookmarks: BookmarkFolder::new(2, "Other Bookmarks"),
            id_counter: 3,
        }
    }

    pub fn add_bookmark(&mut self, url: &str, title: &str, folder_id: Option<u64>) -> u64 {
        self.id_counter += 1;
        let id = self.id_counter;
        let bookmark = Bookmark::new(id, url, title);

        let folder = folder_id
            .and_then(|id| self.find_folder_mut(id))
            .unwrap_or(&mut self.bookmark_bar);

        folder.items.push(BookmarkItem::Bookmark(bookmark));
        id
    }

    pub fn add_folder(&mut self, name: &str, parent_id: Option<u64>) -> u64 {
        self.id_counter += 1;
        let id = self.id_counter;
        let folder = BookmarkFolder::new(id, name);

        let parent = parent_id
            .and_then(|id| self.find_folder_mut(id))
            .unwrap_or(&mut self.bookmark_bar);

        parent.items.push(BookmarkItem::Folder(folder));
        id
    }

    pub fn remove(&mut self, id: u64) {
        self.bookmark_bar.remove_item(id);
        self.other_bookmarks.remove_item(id);
    }

    pub fn find_bookmark(&self, id: u64) -> Option<&Bookmark> {
        self.bookmark_bar.find_bookmark(id)
            .or_else(|| self.other_bookmarks.find_bookmark(id))
    }

    pub fn find_folder_mut(&mut self, id: u64) -> Option<&mut BookmarkFolder> {
        if self.bookmark_bar.id == id {
            return Some(&mut self.bookmark_bar);
        }
        if self.other_bookmarks.id == id {
            return Some(&mut self.other_bookmarks);
        }
        self.bookmark_bar.find_folder_mut(id)
            .or_else(|| self.other_bookmarks.find_folder_mut(id))
    }

    pub fn bookmark_bar(&self) -> &BookmarkFolder {
        &self.bookmark_bar
    }

    pub fn other_bookmarks(&self) -> &BookmarkFolder {
        &self.other_bookmarks
    }

    pub fn is_bookmarked(&self, url: &str) -> bool {
        self.bookmark_bar.contains_url(url) || self.other_bookmarks.contains_url(url)
    }
}

impl Default for BookmarkManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A bookmark.
#[derive(Clone, Debug)]
pub struct Bookmark {
    pub id: u64,
    pub url: String,
    pub title: String,
    pub favicon: Option<String>,
    pub created: std::time::SystemTime,
}

impl Bookmark {
    pub fn new(id: u64, url: &str, title: &str) -> Self {
        Self {
            id,
            url: url.to_string(),
            title: title.to_string(),
            favicon: None,
            created: std::time::SystemTime::now(),
        }
    }
}

/// A bookmark folder.
#[derive(Clone, Debug)]
pub struct BookmarkFolder {
    pub id: u64,
    pub name: String,
    pub items: Vec<BookmarkItem>,
}

impl BookmarkFolder {
    pub fn new(id: u64, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            items: Vec::new(),
        }
    }

    fn remove_item(&mut self, id: u64) {
        self.items.retain(|item| {
            match item {
                BookmarkItem::Bookmark(b) => b.id != id,
                BookmarkItem::Folder(f) => f.id != id,
            }
        });
        for item in &mut self.items {
            if let BookmarkItem::Folder(f) = item {
                f.remove_item(id);
            }
        }
    }

    fn find_bookmark(&self, id: u64) -> Option<&Bookmark> {
        for item in &self.items {
            match item {
                BookmarkItem::Bookmark(b) if b.id == id => return Some(b),
                BookmarkItem::Folder(f) => {
                    if let Some(b) = f.find_bookmark(id) {
                        return Some(b);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn find_folder_mut(&mut self, id: u64) -> Option<&mut BookmarkFolder> {
        for item in &mut self.items {
            if let BookmarkItem::Folder(f) = item {
                if f.id == id {
                    return Some(f);
                }
                if let Some(folder) = f.find_folder_mut(id) {
                    return Some(folder);
                }
            }
        }
        None
    }

    fn contains_url(&self, url: &str) -> bool {
        self.items.iter().any(|item| {
            match item {
                BookmarkItem::Bookmark(b) => b.url == url,
                BookmarkItem::Folder(f) => f.contains_url(url),
            }
        })
    }
}

/// Bookmark item (bookmark or folder).
#[derive(Clone, Debug)]
pub enum BookmarkItem {
    Bookmark(Bookmark),
    Folder(BookmarkFolder),
}
