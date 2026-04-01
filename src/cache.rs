use crate::types::BookmarkEntry;
use rkyv::rancor::Error;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// Returns bookmarks for `provider_name`, using a `.cache/<name>.rkyv` file when the
/// bookmark source file has not been modified since the cache was last written.
///
/// Cache validity: the `.rkyv` file's own mtime is compared against the bookmark
/// source file's mtime – no separate sidecar file needed.
///
/// # Arguments
/// * `provider_name` – human-readable browser name, used as the cache file stem
/// * `bookmark_path` – path to the browser's raw bookmark file (used for mtime check)
/// * `fetch`         – closure that reads & parses the raw bookmark file from scratch
pub fn get_or_update(
    provider_name: &str,
    bookmark_path: &Path,
    fetch: impl FnOnce() -> Vec<BookmarkEntry>,
) -> Vec<BookmarkEntry> {
    // Ensure the cache directory exists next to the binary's working directory.
    let cache_dir = Path::new(".cache");
    if !cache_dir.exists()
        && let Err(e) = fs::create_dir_all(cache_dir)
    {
        eprintln!("Failed to create cache dir: {}", e);
    }

    let cache_file = cache_dir.join(format!("{}.rkyv", provider_name.replace(' ', "_")));

    // ── 1. Read both mtimes ──────────────────────────────────────────────────
    let mtime = |p: &Path| -> Option<SystemTime> {
        fs::metadata(p).and_then(|m| m.modified()).ok()
    };

    // ── 2. Cache hit when the .rkyv file is at least as new as the source ───
    if let (Some(src_mtime), Some(cache_mtime)) = (mtime(bookmark_path), mtime(&cache_file))
        && cache_mtime >= src_mtime
    {
        match fs::read(&cache_file).map(|b| rkyv::from_bytes::<Vec<BookmarkEntry>, Error>(&b)) {
            Ok(Ok(entries)) => {
                eprintln!("[cache] HIT  {} ({} bookmarks)", provider_name, entries.len());
                return entries;
            }
            Ok(Err(e)) => eprintln!("[cache] Deserialise failed for {}: {}", provider_name, e),
            Err(e) => eprintln!("[cache] Read failed for {}: {}", provider_name, e),
        }
        // fall through to re-fetch
    }

    // ── 3. Cache miss – fetch fresh data and write cache ────────────────────
    let entries = fetch();

    if entries.is_empty() {
        return entries;
    }

    match rkyv::to_bytes::<Error>(&entries) {
        Ok(bytes) => match fs::write(&cache_file, &bytes) {
            Ok(_) => eprintln!(
                "[cache] MISS {} – wrote {} bookmarks",
                provider_name,
                entries.len()
            ),
            Err(e) => eprintln!("[cache] Write failed for {}: {}", provider_name, e),
        },
        Err(e) => eprintln!("[cache] Serialise failed for {}: {}", provider_name, e),
    }

    entries
}
