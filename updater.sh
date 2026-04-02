#!/bin/bash

# Auto-update mechanism for Universal Bookmarks Alfred Workflow
# Inspired by OneUpdater (https://github.com/vitorgalvao/alfred-workflows)
#
# This script is triggered in the background every time the user selects
# a bookmark result. It silently checks GitHub for a newer release and,
# if found, downloads and opens the .alfredworkflow to trigger Alfred's
# built-in import/update dialog.
#
# ── Debug ──────────────────────────────────────────────────────
# Set the workflow environment variable `UPDATER_DEBUG` to `1` in
# Alfred Preferences → Workflows → Universal Bookmarks → [x] icon
# (Environment Variables) to enable verbose logging.
#
# Log file location:
#   ~/Library/Caches/com.runningwithcrayons.Alfred/Workflow Data/<bundleid>/updater.log

# ── Configuration ──────────────────────────────────────────────
readonly REPO='we11adam/alfred-universal-bookmarks'
readonly FREQUENCY=7 # days between update checks
# ── End Configuration ──────────────────────────────────────────

readonly WORKFLOW_DIR="$(cd "$(dirname "$0")" && pwd)"
readonly BUNDLE_ID=$(/usr/libexec/PlistBuddy -c 'Print :bundleid' "${WORKFLOW_DIR}/info.plist" 2>/dev/null)
readonly CACHE_DIR="${alfred_workflow_cache:-${HOME}/Library/Caches/com.runningwithcrayons.Alfred/Workflow Data/${BUNDLE_ID}}"
readonly LOG_FILE="${CACHE_DIR}/updater.log"

# ── Logging setup ─────────────────────────────────────────────
# UPDATER_DEBUG is passed in by Alfred as a workflow environment variable.
DEBUG="${UPDATER_DEBUG:-0}"

log() {
    [[ "${DEBUG}" != "1" ]] && return
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" >> "${LOG_FILE}"
}

if [[ "${DEBUG}" == "1" ]]; then
    # Ensure cache dir exists early for logging
    mkdir -p "${CACHE_DIR}" 2>/dev/null
    # Truncate log if it exceeds 512 KB to prevent unbounded growth
    if [[ -f "${LOG_FILE}" ]] && (( $(stat -f%z "${LOG_FILE}" 2>/dev/null || echo 0) > 524288 )); then
        tail -c 262144 "${LOG_FILE}" > "${LOG_FILE}.tmp" && mv "${LOG_FILE}.tmp" "${LOG_FILE}"
    fi
    log "── updater started ──────────────────────────────"
    log "WORKFLOW_DIR=${WORKFLOW_DIR}"
    log "CACHE_DIR=${CACHE_DIR}"
else
    # Silent operation — discard all stdout/stderr
    exec &>/dev/null
fi

readonly LAST_CHECK_FILE="${CACHE_DIR}/.last_update_check"

# Ensure cache directory exists
mkdir -p "${CACHE_DIR}" || { log "ERROR: failed to create cache dir"; exit 1; }

# ── Frequency gate ─────────────────────────────────────────────
if [[ -f "${LAST_CHECK_FILE}" ]]; then
    last_check=$(cat "${LAST_CHECK_FILE}")
    now=$(date +%s)
    days_elapsed=$(( (now - last_check) / 86400 ))
    log "last check: ${last_check}, now: ${now}, days elapsed: ${days_elapsed}, frequency: ${FREQUENCY}"
    if [[ ${days_elapsed} -lt ${FREQUENCY} ]]; then
        log "skipping: next check in $(( FREQUENCY - days_elapsed )) day(s)"
        exit 0
    fi
else
    log "no previous check recorded, proceeding"
fi

# Record this check
date +%s > "${LAST_CHECK_FILE}"

# ── Get local version ─────────────────────────────────────────
local_version=$(/usr/libexec/PlistBuddy -c 'Print :version' "${WORKFLOW_DIR}/info.plist" 2>/dev/null)
log "local version: ${local_version:-<not found>}"
[[ -z "${local_version}" ]] && { log "ERROR: cannot read local version from info.plist"; exit 1; }

# ── Fetch latest release from GitHub ──────────────────────────
log "fetching: https://api.github.com/repos/${REPO}/releases/latest"
release_json=$(curl -sL --max-time 10 \
    "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null)
[[ -z "${release_json}" ]] && { log "ERROR: empty response from GitHub API"; exit 1; }

# Extract tag (strip leading 'v' if present)
remote_version=$(echo "${release_json}" | \
    /usr/bin/python3 -c "
import sys, json
data = json.load(sys.stdin)
print(data.get('tag_name', '').lstrip('v'))
" 2>/dev/null)
log "remote version: ${remote_version:-<not found>}"
[[ -z "${remote_version}" ]] && { log "ERROR: cannot parse remote version"; exit 1; }

# ── Compare versions ──────────────────────────────────────────
if [[ "${local_version}" == "${remote_version}" ]]; then
    log "up to date (${local_version})"
    exit 0
fi

newest=$(printf '%s\n%s' "${local_version}" "${remote_version}" | sort -V | tail -1)
if [[ "${newest}" == "${local_version}" ]]; then
    log "local (${local_version}) is newer than remote (${remote_version}), skipping"
    exit 0
fi

log "update available: ${local_version} → ${remote_version}"

# ── Download update ───────────────────────────────────────────
download_url=$(echo "${release_json}" | \
    /usr/bin/python3 -c "
import sys, json
assets = json.load(sys.stdin).get('assets', [])
universal = [a for a in assets if 'universal' in a['name'] and a['name'].endswith('.alfredworkflow')]
fallback  = [a for a in assets if a['name'].endswith('.alfredworkflow')]
pick = universal[0] if universal else (fallback[0] if fallback else None)
print(pick['browser_download_url'] if pick else '')
" 2>/dev/null)
log "download URL: ${download_url:-<not found>}"
[[ -z "${download_url}" ]] && { log "ERROR: no .alfredworkflow asset in release"; exit 1; }

readonly TMPFILE="/tmp/UniversalBookmarks-latest.alfredworkflow"
log "downloading to ${TMPFILE}..."
curl -sL --max-time 60 "${download_url}" -o "${TMPFILE}" 2>/dev/null || { log "ERROR: download failed"; exit 1; }

# ── Install ───────────────────────────────────────────────────
if [[ -s "${TMPFILE}" ]]; then
    log "opening ${TMPFILE} for Alfred import"
    open "${TMPFILE}"
else
    log "ERROR: downloaded file is empty"
    exit 1
fi

log "── updater finished ─────────────────────────────"
