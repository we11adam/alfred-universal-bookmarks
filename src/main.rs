mod extractor;
mod types;
use crate::types::BookmarkEntry;
use alfred::{Item, ItemBuilder, json};
use std::{env, io};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprint!("Insufficient arguments provided. Usage: <program> <action> <keyword>");
        return;
    }

    eprintln!("args: {:?}", &args);

    let action = &args[1];
    match action.as_str() {
        "search" => {
            let keyword = &args[2];
            search(keyword);
        }
        _ => {
            eprintln!("Unsupported action: {}", action);
        }
    }
}

fn search(keyword: &str) {
    let bookmarks = extractor::extract_bookmarks();
    let keyword_lower = keyword.to_lowercase();

    let mut name_matches: Vec<Item> = Vec::new();
    let mut url_matches: Vec<Item> = Vec::new();

    for bookmark in &bookmarks {
        let name_hit = bookmark.name.to_lowercase().contains(&keyword_lower);
        let url_hit = bookmark.url.to_lowercase().contains(&keyword_lower);

        if name_hit {
            name_matches.push(build_item(bookmark));
        } else if url_hit {
            url_matches.push(build_item(bookmark));
        }
    }

    name_matches.append(&mut url_matches);
    eprintln!(
        "Found {:?} matches for keyword: {}",
        name_matches.len(),
        keyword
    );
    let _ = json::write_items(io::stdout(), &name_matches);
}

fn build_item<'a>(bookmark: &'a BookmarkEntry) -> Item<'a> {
    ItemBuilder::new(bookmark.name.clone())
        .subtitle(bookmark.url.as_ref())
        .arg(bookmark.url.as_ref())
        .into_item()
}
