use crate::types::*;
use std::env;
use std::fs::File;
use std::path::Path;

pub fn extract_bookmarks<'a>() -> Vec<BookmarkEntry<'a>> {
    let Ok(home) = env::var("HOME") else {
        eprint!("Failed to get HOME environment variable");
        return Vec::new();
    };

    BOOKMARK_PROVIDERS
        .iter()
        .filter(|entry| entry.0 != "safari")
        .flat_map(|(_, path)| {
            let bookmark_file = Path::new(&home).join(path);
            extract_from_chromiumlike(&bookmark_file)
        })
        .collect()
}


// handles chromiumlike bookmarks
pub fn extract_from_chromiumlike<'a, P: AsRef<Path>>(path: P) -> Vec<BookmarkEntry<'a>> {
    let Ok(file) = File::open(path) else {
        return Vec::new();
    };

    let Ok(raw_bookmarks) = serde_json::from_reader::<_, ChromiumLikeBookmarks>(file) else {
        return Vec::new();
    };

    extract_all_urls(&raw_bookmarks)
}


// collect all URls
fn extract_all_urls<'a>(bookmarks: &ChromiumLikeBookmarks) -> Vec<BookmarkEntry<'a>> {
    let mut results = Vec::new();
    let mut roots = Vec::from([
        &bookmarks.roots.bookmark_bar,
        &bookmarks.roots.other,
        &bookmarks.roots.synced,
    ]);

    if let Some(ref wsv2) = bookmarks.roots.workspaces_v2 {
        roots.push(wsv2);
    }
    for root in roots {
        collect_urls(root, &mut results);
    }
    results
}


// recursively collect URls, mutating the results vector as we go
fn collect_urls(payload: &ChromiumLikeEntry, results: &mut Vec<BookmarkEntry>) {
    if payload.r#type == ChromiumLikeEntryItemType::Url {
        if let Some(ref url) = payload.url {
            results.push(BookmarkEntry::new(payload.name.clone(), url.clone()));
        }
    }

    if let Some(ref children) = payload.children {
        for child in children {
            collect_urls(child, results);
        }
    }
}