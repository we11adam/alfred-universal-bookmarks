use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct BookmarkEntry<'a> {
    pub name: Cow<'a, str>,
    pub url: Cow<'a, str>,
}

impl<'a> BookmarkEntry<'a> {
    pub fn new<S: Into<Cow<'a, str>>>(name: S, url: S) -> BookmarkEntry<'a> {
        Self {
            name: name.into(),
            url: url.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChromiumLikeBookmarks {
    pub checksum: String,
    pub roots: Roots,
    pub sync_metadata: Option<String>,
    pub version: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Roots {
    pub bookmark_bar: ChromiumLikeEntry,
    pub other: ChromiumLikeEntry,
    pub synced: ChromiumLikeEntry,
    #[serde(default)] // skip when not available
    pub workspaces_v2: Option<ChromiumLikeEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChromiumLikeEntry {
    pub children: Option<Vec<ChromiumLikeEntry>>,
    pub date_added: String,
    pub date_last_used: String,
    pub date_modified: Option<String>,
    pub guid: String,
    pub id: String,
    pub name: String,
    pub source: String,
    pub url: Option<String>,
    pub r#type: ChromiumLikeEntryItemType,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChromiumLikeEntryItemType {
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
    (
        "brave",
        "Library/Application Support/BraveSoftware/Brave-Browser/Default/Bookmarks",
    ),
    (
        "brave_beta",
        "Library/Application Support/BraveSoftware/Brave-Browser-Beta/Default/Bookmarks",
    ),
    (
        "chrome",
        "Library/Application Support/Google/Chrome/Default/Bookmarks",
    ),
    (
        "chromium",
        "Library/Application Support/Chromium/Default/Bookmarks",
    ),
    (
        "opera",
        "Library/Application Support/com.operasoftware.Opera/Bookmarks",
    ),
    (
        "sidekick",
        "Library/Application Support/Sidekick/Default/Bookmarks",
    ),
    (
        "vivaldi",
        "Library/Application Support/Vivaldi/Default/Bookmarks",
    ),
    (
        "edge",
        "Library/Application Support/Microsoft Edge/Default/Bookmarks",
    ),
    (
        "arc",
        "Library/Application Support/Arc/User Data/Default/Bookmarks",
    ),
    (
        "dia",
        "Library/Application Support/Dia/User Data/Default/Bookmarks",
    ),
    (
        "thorium",
        "Library/Application Support/Thorium/Default/Bookmarks",
    ),
    (
        "comet",
        "Library/Application Support/Comet/Default/Bookmarks",
    ),
    (
        "helium",
        "Library/Application Support/net.imput.helium/Default/Bookmarks",
    ),
    ("safari", "Library/Safari/Bookmarks.plist"),
];
