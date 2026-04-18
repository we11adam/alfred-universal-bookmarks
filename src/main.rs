mod cache;
mod deleter;
mod extractor;
mod pinyin;
mod types;
mod updater;
use crate::types::*;
use alfred::{Item, ItemBuilder, Modifier, json};
use std::{collections::HashSet, env, io};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprint!("Insufficient arguments provided. Usage: ub <action> [args...]");
        return;
    }

    eprintln!("Args: {:?}", &args);

    let action = &args[1];
    match action.as_str() {
        "search" => {
            let keyword = args.get(2).map(|s| s.as_str()).unwrap_or("");
            search(keyword);
        }
        "update" => {
            updater::run();
        }
        "delete" => {
            let arg = args.get(2).map(|s| s.as_str()).unwrap_or("");
            deleter::delete(arg);
        }
        "version" => {
            println!("{} ({})", env!("CARGO_PKG_VERSION"), env!("GIT_COMMIT"),);
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
    let mut pinyin_matches: Vec<Item> = Vec::new();
    let mut url_matches: Vec<Item> = Vec::new();

    for bookmark in &bookmarks {
        let name_hit = bookmark.name.to_lowercase().contains(&keyword_lower);
        let url_hit = bookmark.url.to_lowercase().contains(&keyword_lower);

        if name_hit {
            name_matches.push(build_item(bookmark));
        } else if pinyin::pinyin_match(&bookmark.name, &keyword_lower) {
            pinyin_matches.push(build_item(bookmark));
        } else if url_hit {
            url_matches.push(build_item(bookmark));
        }
    }

    name_matches.append(&mut pinyin_matches);
    name_matches.append(&mut url_matches);
    deduplicate(&mut name_matches);
    eprintln!(
        "Found {:?} matches for keyword: {}",
        name_matches.len(),
        keyword
    );
    let _ = json::write_items(io::stdout(), &name_matches);
}

fn build_item(bookmark: &'_ BookmarkEntry) -> Item<'_> {
    let subtitle =
        bookmark.source.clone() + PATH_SPLIT + bookmark.path.as_str() + bookmark.url.as_str();
    let delete_arg = format!("{}\t{}", bookmark.source, bookmark.url);
    ItemBuilder::new(bookmark.name.as_str())
        .subtitle(subtitle)
        .arg(bookmark.url.as_str())
        .uid(bookmark.url.as_str())
        .icon_path("./icon.png")
        .subtitle_mod(
            Modifier::Command,
            format!("Delete from {}", bookmark.source),
        )
        .arg_mod(Modifier::Command, delete_arg)
        .into_item()
}

fn deduplicate(items: &mut Vec<Item>) {
    let mut seen = HashSet::new();
    items.retain(|item| seen.insert(item.arg.clone()));
}
