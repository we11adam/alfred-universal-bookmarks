//! Auto-update mechanism for Universal Bookmarks Alfred Workflow.
//!
//! Inspired by OneUpdater. Checks GitHub Releases for a newer version and, if found,
//! downloads the `.alfredworkflow` package and opens it to trigger Alfred's import dialog.
//!
//! # Environment variables (set via Alfred workflow variables)
//! - `UPDATER_DEBUG` – set to `"1"` to write verbose logs to `<cache_dir>/updater.log`
//! - `alfred_workflow_cache` – Alfred injects this automatically at runtime

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

// ── Configuration ────────────────────────────────────────────────────────────
const REPO: &str = "we11adam/alfred-universal-bookmarks";
const FREQUENCY_DAYS: u64 = 7;
const LAST_CHECK_FILE: &str = ".last_update_check";
const LOG_FILE: &str = "updater.log";
const MAX_LOG_BYTES: u64 = 512 * 1024; // 512 KB
const TMP_WORKFLOW_PATH: &str = "/tmp/UniversalBookmarks-latest.alfredworkflow";

struct LatestRelease {
    json: String,
    version: String,
}

struct WorkflowAsset {
    arch: String,
    download_url: String,
}

enum UpdateError {
    Network,
    ParseResponse,
    NoAsset,
    Download(String),
    OpenInstaller,
}

/// Run the full update-check workflow. This is designed to be called from a
/// background process (`./ub update &`) so it must never panic or block the
/// main search flow.
pub fn run() {
    let debug = env::var("UPDATER_DEBUG").unwrap_or_default() == "1";
    let cache_dir = resolve_cache_dir();

    let mut logger = Logger::new(debug, &cache_dir);
    logger.log("── updater started ──────────────────────────────");

    // If running from a git checkout (development environment), skip auto-update.
    // This avoids the development workflow attempting to download and install
    // release assets while developing locally.
    if is_development_checkout() {
        logger.log(
            "Detected .git in current directory; running in development mode, skipping auto-update",
        );
        return;
    }

    if let Err(e) = fs::create_dir_all(&cache_dir) {
        logger.log(&format!("ERROR: failed to create cache dir: {e}"));
        return;
    }

    // ── Frequency gate ───────────────────────────────────────────────────────
    let check_file = cache_dir.join(LAST_CHECK_FILE);
    let now = current_unix_timestamp();

    if let Ok(contents) = fs::read_to_string(&check_file) {
        if let Ok(last_check) = contents.trim().parse::<u64>() {
            let days_elapsed = (now - last_check) / 86400;
            logger.log(&format!(
                "last check: {last_check}, now: {now}, days elapsed: {days_elapsed}, frequency: {FREQUENCY_DAYS}"
            ));
            if days_elapsed < FREQUENCY_DAYS {
                logger.log(&format!(
                    "skipping: next check in {} day(s)",
                    FREQUENCY_DAYS - days_elapsed
                ));
                return;
            }
        }
    } else {
        logger.log("no previous check recorded, proceeding");
    }

    // Record this check
    let _ = fs::write(&check_file, now.to_string());

    // ── Get local version ────────────────────────────────────────────────────
    let local_version = env!("CARGO_PKG_VERSION");
    logger.log(&format!("local version: {local_version}"));

    // ── Fetch latest release from GitHub ──────────────────────────────────────
    let api_url = latest_release_api_url();
    logger.log(&format!("fetching: {api_url}"));

    let release = match fetch_latest_release() {
        Ok(release) => release,
        Err(UpdateError::Network) => {
            logger.log("ERROR: failed to fetch release info from GitHub");
            return;
        }
        Err(UpdateError::ParseResponse) => {
            logger.log("ERROR: cannot parse tag_name from response");
            return;
        }
        Err(_) => return,
    };
    logger.log(&format!("remote version: {}", release.version));

    // ── Compare versions ─────────────────────────────────────────────────────
    if local_version == release.version {
        logger.log(&format!("up to date ({local_version})"));
        return;
    }

    if !is_newer(&release.version, local_version) {
        logger.log(&format!(
            "local ({local_version}) is newer than remote ({}), skipping",
            release.version
        ));
        return;
    }

    logger.log(&format!(
        "update available: {local_version} → {}",
        release.version
    ));

    // ── Detect binary architecture ───────────────────────────────────────────
    let asset = match select_workflow_asset(&release.json) {
        Ok(asset) => asset,
        Err(UpdateError::NoAsset) => {
            logger.log("ERROR: no .alfredworkflow asset in release");
            return;
        }
        Err(_) => return,
    };
    logger.log(&format!("binary architecture: {}", asset.arch));
    logger.log(&format!("download URL: {}", asset.download_url));

    // ── Download ─────────────────────────────────────────────────────────────
    logger.log(&format!("downloading to {TMP_WORKFLOW_PATH}..."));

    let tmp_path = match download_workflow(&asset.download_url) {
        Ok(path) => path,
        Err(UpdateError::Download(e)) => {
            logger.log(&format!("ERROR: download failed: {e}"));
            return;
        }
        Err(_) => return,
    };

    // ── Install ──────────────────────────────────────────────────────────────
    logger.log(&format!("opening {tmp_path} for Alfred import"));
    let _ = open_workflow(tmp_path);

    logger.log("── updater finished ─────────────────────────────");
}

/// Run an immediate update check requested explicitly by the user (`:update` command).
/// Bypasses the frequency gate and always performs a network check.
/// Resets the frequency gate timestamp on success so the background auto-update
/// won't fire again within FREQUENCY_DAYS.
/// Returns a short human-readable result string suitable for an Alfred notification.
pub fn run_once() -> String {
    if is_development_checkout() {
        return "Development mode – update skipped".to_string();
    }

    let cache_dir = resolve_cache_dir();
    let _ = fs::create_dir_all(&cache_dir);

    let local_version = env!("CARGO_PKG_VERSION");

    let release = match fetch_latest_release() {
        Ok(release) => release,
        Err(UpdateError::Network) => return "Update check failed – network error".to_string(),
        Err(UpdateError::ParseResponse) => {
            return "Update check failed – could not parse response".to_string();
        }
        Err(_) => return "Update check failed".to_string(),
    };

    // Reset the frequency gate so background auto-update won't fire for another cycle.
    let now = current_unix_timestamp();
    let _ = fs::write(cache_dir.join(LAST_CHECK_FILE), now.to_string());

    if local_version == release.version || !is_newer(&release.version, local_version) {
        return format!("v{local_version} is the latest version");
    }

    let asset = match select_workflow_asset(&release.json) {
        Ok(asset) => asset,
        Err(UpdateError::NoAsset) => {
            return format!("v{} is available but no asset was found", release.version);
        }
        Err(_) => return "Update check failed".to_string(),
    };

    let tmp_path = match download_workflow(&asset.download_url) {
        Ok(path) => path,
        Err(UpdateError::Download(e)) => return format!("Update download failed: {e}"),
        Err(_) => return "Update download failed".to_string(),
    };

    match open_workflow(tmp_path) {
        Ok(()) => format!(
            "Installing v{} – Alfred will prompt to import",
            release.version
        ),
        Err(_) => format!(
            "Downloaded v{} but failed to open installer",
            release.version
        ),
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn is_development_checkout() -> bool {
    env::current_dir()
        .map(|cwd| cwd.join(".git").exists())
        .unwrap_or(false)
}

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn latest_release_api_url() -> String {
    format!("https://api.github.com/repos/{REPO}/releases/latest")
}

fn fetch_latest_release() -> Result<LatestRelease, UpdateError> {
    let release_json = fetch_url(&latest_release_api_url()).ok_or(UpdateError::Network)?;
    let version = extract_json_string(&release_json, "tag_name")
        .map(|tag| tag.trim_start_matches('v').to_string())
        .ok_or(UpdateError::ParseResponse)?;

    Ok(LatestRelease {
        json: release_json,
        version,
    })
}

fn select_workflow_asset(release_json: &str) -> Result<WorkflowAsset, UpdateError> {
    let arch = detect_arch();
    let download_url = find_workflow_asset(release_json, &arch).ok_or(UpdateError::NoAsset)?;

    Ok(WorkflowAsset { arch, download_url })
}

fn download_workflow(download_url: &str) -> Result<&'static str, UpdateError> {
    download_file(download_url, TMP_WORKFLOW_PATH).map_err(UpdateError::Download)?;
    Ok(TMP_WORKFLOW_PATH)
}

fn open_workflow(path: &str) -> Result<(), UpdateError> {
    match Command::new("open").arg(path).status() {
        Ok(status) if status.success() => Ok(()),
        _ => Err(UpdateError::OpenInstaller),
    }
}

/// Resolve the cache directory. Alfred sets `alfred_workflow_cache` at runtime;
/// fall back to the conventional path using the bundle id.
fn resolve_cache_dir() -> PathBuf {
    if let Ok(dir) = env::var("alfred_workflow_cache") {
        return PathBuf::from(dir);
    }

    // Fallback: read bundle id from info.plist next to the binary
    let bundle_id = read_bundle_id().unwrap_or_else(|| "com.welladam.universalbookmarks".into());
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".into());

    PathBuf::from(home)
        .join("Library/Caches/com.runningwithcrayons.Alfred/Workflow Data")
        .join(bundle_id)
}

/// Best-effort read of bundleid from info.plist in the working directory.
fn read_bundle_id() -> Option<String> {
    let output = Command::new("/usr/libexec/PlistBuddy")
        .args(["-c", "Print :bundleid", "info.plist"])
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Simple semver comparison: returns true if `remote` is strictly newer than `local`.
fn is_newer(remote: &str, local: &str) -> bool {
    let parse = |v: &str| -> Vec<u64> {
        v.split('.')
            .map(|s| s.parse::<u64>().unwrap_or(0))
            .collect()
    };
    let r = parse(remote);
    let l = parse(local);
    r > l
}

/// Minimal JSON string extraction without a JSON library dependency.
/// Finds `"key": "value"` and returns value.
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let pos = json.find(&pattern)?;
    let after_key = &json[pos + pattern.len()..];
    // Skip colon and whitespace
    let after_colon = after_key.trim_start().strip_prefix(':')?;
    let after_ws = after_colon.trim_start();
    // Value should start with a quote
    let after_quote = after_ws.strip_prefix('"')?;
    let end = after_quote.find('"')?;
    Some(after_quote[..end].to_string())
}

/// Detect the architecture of the currently running binary using `lipo -archs`.
/// Returns "arm64", "amd64", "universal", or "unknown".
fn detect_arch() -> String {
    let exe = match env::current_exe() {
        Ok(p) => p,
        Err(_) => return "unknown".into(),
    };

    let output = Command::new("/usr/bin/lipo")
        .args(["-archs", &exe.to_string_lossy()])
        .output();

    let archs = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => return "unknown".into(),
    };

    // lipo -archs returns space-separated list, e.g. "x86_64 arm64"
    if archs.contains("x86_64") && archs.contains("arm64") {
        "universal".into()
    } else if archs.contains("arm64") {
        "arm64".into()
    } else if archs.contains("x86_64") {
        "amd64".into()
    } else {
        "unknown".into()
    }
}

/// Find the best `.alfredworkflow` download URL from release JSON.
/// Matches the binary architecture: arm64 → arm64 asset, amd64 → amd64 asset,
/// universal → universal asset. Falls back to any `.alfredworkflow` if no match.
fn find_workflow_asset(json: &str, arch: &str) -> Option<String> {
    let mut preferred: Option<String> = None;
    let mut fallback: Option<String> = None;

    let search = "\"browser_download_url\"";
    let mut start = 0;
    while let Some(pos) = json[start..].find(search) {
        let abs_pos = start + pos;
        if let Some(url) = extract_json_string(&json[abs_pos..], "browser_download_url")
            && url.ends_with(".alfredworkflow")
        {
            // Match architecture keyword in the asset filename
            if preferred.is_none() && url.contains(arch) {
                preferred = Some(url.clone());
            }
            if fallback.is_none() {
                fallback = Some(url);
            }
        }
        start = abs_pos + search.len();
    }

    preferred.or(fallback)
}

/// Helper to read proxy settings from environment and apply them to the given Command.
fn apply_proxy_envs(cmd: &mut Command) {
    let vars = [
        "https_proxy",
        "HTTPS_PROXY",
        "http_proxy",
        "HTTP_PROXY",
        "all_proxy",
        "ALL_PROXY",
    ];
    for var in vars {
        if let Ok(val) = env::var(var) {
            let val = val.trim();
            if !val.is_empty() {
                cmd.env(var, val);
            }
        }
    }
}

/// Fetch a URL using /usr/bin/curl (available on all macOS).
fn fetch_url(url: &str) -> Option<String> {
    let mut cmd = Command::new("/usr/bin/curl");
    cmd.args(["-sL", "--max-time", "10"]);

    apply_proxy_envs(&mut cmd);

    let output = cmd.arg(url).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        None
    }
}

/// Download a file using /usr/bin/curl.
fn download_file(url: &str, dest: &str) -> Result<(), String> {
    let mut cmd = Command::new("/usr/bin/curl");
    cmd.args(["-sL", "--max-time", "60"]);

    apply_proxy_envs(&mut cmd);

    let status = cmd
        .args([url, "-o", dest])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        // Verify file is non-empty
        let meta = fs::metadata(dest).map_err(|e| e.to_string())?;
        if meta.len() == 0 {
            return Err("downloaded file is empty".into());
        }
        Ok(())
    } else {
        Err(format!("curl exited with {status}"))
    }
}

// ── Logger ───────────────────────────────────────────────────────────────────

struct Logger {
    enabled: bool,
    path: PathBuf,
}

impl Logger {
    fn new(enabled: bool, cache_dir: &Path) -> Self {
        let path = cache_dir.join(LOG_FILE);

        if enabled {
            // Truncate log if it exceeds MAX_LOG_BYTES
            if let Ok(meta) = fs::metadata(&path)
                && meta.len() > MAX_LOG_BYTES
                && let Ok(data) = fs::read(&path)
            {
                let half = data.len() / 2;
                let _ = fs::write(&path, &data[half..]);
            }
        }

        Self { enabled, path }
    }

    fn log(&mut self, msg: &str) {
        if !self.enabled {
            return;
        }
        let timestamp = {
            // Use date command for human-readable local time (no chrono dependency)
            Command::new("date")
                .arg("+%Y-%m-%d %H:%M:%S")
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default()
        };
        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = writeln!(file, "[{timestamp}] {msg}");
        }
        // Also write to stderr for manual testing
        eprintln!("[{timestamp}] {msg}");
    }
}
