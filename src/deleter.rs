use crate::types::*;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn delete(arg: &str) {
    let Some((source, url)) = arg.split_once('\t') else {
        eprintln!("Invalid delete argument: {}", arg);
        return;
    };

    eprintln!("Deleting bookmark: source={}, url={}", source, url);

    let Ok(home) = env::var("HOME") else {
        eprintln!("Failed to get HOME");
        println!("Error: Cannot determine HOME directory");
        return;
    };

    let Some(bookmark_path) = find_bookmark_path(source, &home) else {
        eprintln!("Unknown source: {}", source);
        println!("Error: Unknown browser '{}'", source);
        return;
    };

    if !bookmark_path.exists() {
        eprintln!("Bookmark file not found: {}", bookmark_path.display());
        println!("Error: Bookmark file not found for {}", source);
        return;
    }

    let result = if source == "Safari" {
        delete_safari_bookmark(&bookmark_path, url)
    } else {
        delete_chromium_bookmark(&bookmark_path, url)
    };

    match result {
        Ok(()) => {
            invalidate_cache(source);
            println!("Deleted from {}", source);
        }
        Err(e) => {
            eprintln!("Failed to delete bookmark: {}", e);
            println!("Error: {}", e);
        }
    }
}

fn find_bookmark_path(source: &str, home: &str) -> Option<PathBuf> {
    let env_var_name = format!("{}_BOOKMARKS_PATH", source.to_uppercase().replace(' ', "_"));
    if let Ok(custom_path) = env::var(&env_var_name) {
        let parsed = if custom_path.starts_with('/') {
            PathBuf::from(custom_path)
        } else {
            Path::new(home).join(custom_path)
        };
        return Some(parsed);
    }

    for (name, path) in BOOKMARK_PROVIDERS {
        if *name == source {
            return Some(Path::new(home).join(path));
        }
    }
    None
}

fn invalidate_cache(source: &str) {
    let cache_file = Path::new(".cache").join(format!("{}.rkyv", source.replace(' ', "_")));
    if cache_file.exists() {
        if let Err(e) = fs::remove_file(&cache_file) {
            eprintln!("Failed to remove cache file: {}", e);
        } else {
            eprintln!("Cache invalidated for {}", source);
        }
    }
}

fn delete_chromium_bookmark(bookmark_path: &Path, url: &str) -> Result<(), String> {
    let content = fs::read_to_string(bookmark_path)
        .map_err(|e| format!("Failed to read bookmark file: {}", e))?;
    let mut bookmarks: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse bookmarks: {}", e))?;

    let roots = bookmarks
        .get_mut("roots")
        .ok_or("Missing 'roots' in bookmarks")?;

    let mut found = false;
    for key in &["bookmark_bar", "other", "synced", "workspaces_v2"] {
        if let Some(root) = roots.get_mut(*key)
            && remove_chromium_entry(root, url)
        {
            found = true;
            break;
        }
    }

    if !found {
        return Err("Bookmark not found".into());
    }

    let json_str = serde_json::to_string_pretty(&bookmarks)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    fs::write(bookmark_path, json_str)
        .map_err(|e| format!("Failed to write bookmark file: {}", e))?;

    Ok(())
}

fn remove_chromium_entry(entry: &mut serde_json::Value, url: &str) -> bool {
    if let Some(children) = entry.get_mut("children")
        && let Some(children_arr) = children.as_array_mut()
    {
        let orig_len = children_arr.len();
        children_arr.retain(|child| child.get("url").and_then(|u| u.as_str()) != Some(url));
        if children_arr.len() < orig_len {
            return true;
        }
        for child in children_arr.iter_mut() {
            if remove_chromium_entry(child, url) {
                return true;
            }
        }
    }
    false
}

fn delete_safari_bookmark(bookmark_path: &Path, url: &str) -> Result<(), String> {
    let value: plist::Value = plist::from_file(bookmark_path)
        .map_err(|e| format!("Failed to read Safari bookmarks: {}", e))?;
    let mut dict = value
        .into_dictionary()
        .ok_or("Safari bookmarks is not a dictionary")?;

    let children = dict
        .get_mut("Children")
        .ok_or("No Children key in bookmarks")?;
    let children_arr = children.as_array_mut().ok_or("Children is not an array")?;

    if !remove_safari_entry(children_arr, url) {
        return Err("Bookmark not found".into());
    }

    plist::to_file_binary(bookmark_path, &plist::Value::Dictionary(dict))
        .map_err(|e| format!("Failed to write Safari bookmarks: {}", e))?;

    Ok(())
}

fn remove_safari_entry(children: &mut Vec<plist::Value>, url: &str) -> bool {
    let orig_len = children.len();
    children.retain(|child| {
        if let Some(dict) = child.as_dictionary()
            && let Some(url_val) = dict.get("URLString")
            && url_val.as_string() == Some(url)
        {
            return false;
        }
        true
    });

    if children.len() < orig_len {
        return true;
    }

    for child in children.iter_mut() {
        if let Some(dict) = child.as_dictionary_mut()
            && let Some(children_val) = dict.get_mut("Children")
            && let Some(inner_arr) = children_val.as_array_mut()
            && remove_safari_entry(inner_arr, url)
        {
            return true;
        }
    }

    false
}
