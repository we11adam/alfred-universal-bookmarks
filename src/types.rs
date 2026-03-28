use serde::{Deserialize, Serialize};
use std::borrow::Cow;

pub const PATH_SPLIT: &str = " » ";

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct BookmarkEntry<'a> {
    pub name: Cow<'a, str>,
    pub url: Cow<'a, str>,
    pub path: Cow<'a, str>,
    pub source: Cow<'a, str>,
}

impl<'a> BookmarkEntry<'a> {
    pub fn new<S: Into<Cow<'a, str>>>(name: S, url: S, path: S, source: S) -> BookmarkEntry<'a> {
        Self {
            name: name.into(),
            url: url.into(),
            path: path.into(),
            source: source.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChromiumBookmarks {
    pub checksum: String,
    pub roots: Roots,
    pub sync_metadata: Option<String>,
    pub version: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Roots {
    pub bookmark_bar: ChromiumEntry,
    pub other: ChromiumEntry,
    pub synced: ChromiumEntry,
    #[serde(default)] // skip when not available
    pub workspaces_v2: Option<ChromiumEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChromiumEntry {
    pub children: Option<Vec<ChromiumEntry>>,
    pub date_added: String,
    pub date_last_used: String,
    pub date_modified: Option<String>,
    pub guid: String,
    pub id: String,
    pub name: String,
    pub source: String,
    pub url: Option<String>,
    pub r#type: ChromiumEntryItemType,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChromiumEntryItemType {
    Folder,
    Url,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SafariBookmarks {
    #[serde(rename = "Children")]
    pub children: Option<Vec<SafariEntry>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SafariEntry {
    #[serde(rename = "Title")]
    pub title: Option<String>,
    #[serde(rename = "URLString")]
    pub url_string: Option<String>,
    #[serde(rename = "Children")]
    pub children: Option<Vec<SafariEntry>>,
    #[serde(rename = "WebBookmarkType")]
    pub web_bookmark_type: SafariEntryType,
    #[serde(rename = "URIDictionary")]
    pub uri_dictionary: Option<SafariURIDictionary>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum SafariEntryType {
    WebBookmarkTypeLeaf,
    WebBookmarkTypeList,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SafariURIDictionary {
    pub title: String,
}

pub const BOOKMARK_PROVIDERS: &[(&str, &str)] = &[
    ("Safari", "Library/Safari/Bookmarks.plist"),
    (
        "Brave",
        "Library/Application Support/BraveSoftware/Brave-Browser/Default/Bookmarks",
    ),
    (
        "Brave Beta",
        "Library/Application Support/BraveSoftware/Brave-Browser-Beta/Default/Bookmarks",
    ),
    (
        "Google Chrome",
        "Library/Application Support/Google/Chrome/Default/Bookmarks",
    ),
    (
        "Chromium",
        "Library/Application Support/Chromium/Default/Bookmarks",
    ),
    (
        "Opera",
        "Library/Application Support/com.operasoftware.Opera/Bookmarks",
    ),
    (
        "Sidekick",
        "Library/Application Support/Sidekick/Default/Bookmarks",
    ),
    (
        "Vivaldi",
        "Library/Application Support/Vivaldi/Default/Bookmarks",
    ),
    (
        "Microsoft Edge",
        "Library/Application Support/Microsoft Edge/Default/Bookmarks",
    ),
    (
        "Arc",
        "Library/Application Support/Arc/User Data/Default/Bookmarks",
    ),
    (
        "Dia",
        "Library/Application Support/Dia/User Data/Default/Bookmarks",
    ),
    (
        "Thorium",
        "Library/Application Support/Thorium/Default/Bookmarks",
    ),
    (
        "Comet",
        "Library/Application Support/Comet/Default/Bookmarks",
    ),
    (
        "Helium",
        "Library/Application Support/net.imput.helium/Default/Bookmarks",
    ),
];
