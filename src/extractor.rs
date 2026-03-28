use crate::types::*;
use std::borrow::Cow;
use std::env;
use std::fs::File;
use std::path::Path;

trait Collector<'a> {
    fn collect(&mut self, path: &Path, matches: &mut Vec<BookmarkEntry<'a>>);
}

struct ChromiumCollector<'a> {
    source: Cow<'a, str>,
}

impl<'a> Collector<'a> for ChromiumCollector<'a> {
    fn collect(&mut self, path: &Path, matches: &mut Vec<BookmarkEntry<'a>>) {
        let file = File::open(path);
        let file = match file {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "Failed to open bookmark file at {} with error: {}",
                    path.display(),
                    e
                );
                return;
            }
        };

        let raw_bookmarks = serde_json::from_reader::<_, ChromiumBookmarks>(file);
        let raw_bookmarks = match raw_bookmarks {
            Ok(b) => b,
            Err(e) => {
                eprintln!(
                    "Failed to parse bookmark file at {} with error: {}",
                    path.display(),
                    e
                );
                return;
            }
        };

        self.extract_all(&raw_bookmarks, matches);
        eprintln!("Extracted {} entries from {}", matches.len(), self.source);
    }
}

impl<'a> ChromiumCollector<'a> {
    fn new(source: impl Into<Cow<'a, str>>) -> Self {
        Self {
            source: source.into(),
        }
    }

    fn extract_all(&mut self, bookmarks: &ChromiumBookmarks, matches: &mut Vec<BookmarkEntry<'a>>) {
        let mut roots = Vec::from([
            &bookmarks.roots.bookmark_bar,
            &bookmarks.roots.other,
            &bookmarks.roots.synced,
        ]);

        if let Some(ref wsv2) = bookmarks.roots.workspaces_v2 {
            roots.push(wsv2);
        }
        for root in roots {
            self.extract(root, String::new(), matches);
        }
    }

    fn extract(
        &mut self,
        entry: &ChromiumEntry,
        path: String,
        matches: &mut Vec<BookmarkEntry<'a>>,
    ) {
        if entry.r#type == ChromiumEntryItemType::Url {
            if let Some(ref url) = entry.url {
                matches.push(BookmarkEntry::new(
                    entry.name.clone(),
                    url.clone(),
                    path.clone(),
                    self.source.to_string(),
                ));
            }
        }

        if let Some(ref children) = entry.children {
            for child in children {
                let new_path = path.clone() + entry.name.clone().as_str() + PATH_SPLIT;
                self.extract(child, new_path, matches);
            }
        }
    }
}

struct SafariCollector<'a> {
    source: Cow<'a, str>,
}

impl<'a> Collector<'a> for SafariCollector<'a> {
    fn collect(&mut self, path: &Path, matches: &mut Vec<BookmarkEntry<'a>>) {
        self.extract_all(path, matches);
        eprintln!("Extracted {} entries from {}", matches.len(), self.source);
    }
}

impl<'a> SafariCollector<'a> {
    fn new() -> SafariCollector<'a> {
        SafariCollector {
            source: Cow::Borrowed("Safari"),
        }
    }

    fn extract_all(&mut self, path: &Path, matches: &mut Vec<BookmarkEntry<'a>>) {
        let raw_bookmarks: Result<SafariBookmarks, plist::Error> = plist::from_file(path);

        let raw_bookmarks = match raw_bookmarks {
            Ok(bookmarks) => bookmarks,
            Err(e) => {
                eprintln!("Failed to parse safari bookmarks: {}\n", e);
                return;
            }
        };

        if let Some(children) = raw_bookmarks.children {
            for entry in children {
                self.extract(entry, String::new(), matches);
            }
        }
    }

    fn extract(&mut self, entry: SafariEntry, path: String, matches: &mut Vec<BookmarkEntry<'a>>) {
        match entry.web_bookmark_type {
            SafariEntryType::WebBookmarkTypeLeaf => {
                if let Some(url) = entry.url_string {
                    let name = if let Some(title) = entry.title {
                        title
                    } else if let Some(uri_dict) = entry.uri_dictionary {
                        uri_dict.title
                    } else {
                        url.clone()
                    };
                    matches.push(BookmarkEntry::new(name, url, path, self.source.to_string()));
                }
            }
            SafariEntryType::WebBookmarkTypeList => {
                if let Some(children) = entry.children {
                    for child in children {
                        let new_path = path.clone()
                            + entry
                                .title
                                .as_deref()
                                .unwrap_or("Untitled")
                                .replace("com.apple.ReadingList", "ReadingList")
                                .as_str()
                            + PATH_SPLIT;
                        self.extract(child, new_path, matches);
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn extract_bookmarks<'a>() -> Vec<BookmarkEntry<'a>> {
    let Ok(home) = env::var("HOME") else {
        eprintln!("Failed to get HOME environment variable");
        return Vec::new();
    };

    let mut bookmarks: Vec<BookmarkEntry> = Vec::new();

    for (name, path) in BOOKMARK_PROVIDERS {
        let bookmark_file = Path::new(&home).join(path);
        let mut collector: Box<dyn Collector<'a>> = if *name == "Safari" {
            Box::new(SafariCollector::new())
        } else {
            Box::new(ChromiumCollector::new(Cow::Borrowed(*name)))
        };
        collector.collect(&bookmark_file, &mut bookmarks);
    }

    bookmarks
}
