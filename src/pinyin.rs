use pinyin::ToPinyin;

/// A matchable unit extracted from a character in a bookmark name.
enum PyUnit {
    /// Chinese character with its full pinyin (e.g., "gong").
    /// The initial is the first byte.
    Han(&'static str),
    /// ASCII alphanumeric character, lowercased.
    Ascii(u8),
}

/// Extract matchable pinyin units from a name, skipping punctuation and spaces.
fn extract_units(name: &str) -> Vec<PyUnit> {
    name.chars()
        .filter_map(|ch| {
            if let Some(py) = ch.to_pinyin() {
                Some(PyUnit::Han(py.plain()))
            } else if ch.is_ascii_alphanumeric() {
                Some(PyUnit::Ascii(ch.to_ascii_lowercase() as u8))
            } else {
                None
            }
        })
        .collect()
}

/// Check if a keyword matches the name via pinyin.
///
/// Supports flexible matching where each Chinese character can match by its
/// full pinyin or just its initial letter, and the match can start from any
/// character position (substring semantics).
///
/// Examples for name "工作填报":
/// - "gongzuotianbao" (full pinyin)
/// - "zuotian" (partial full pinyin)
/// - "gztb" (all initials)
/// - "zt" (partial initials)
/// - "gztianbao" (mixed initials + full pinyin)
pub fn pinyin_match(name: &str, keyword: &str) -> bool {
    if keyword.is_empty() || !keyword.is_ascii() {
        return false;
    }

    let units = extract_units(name);
    // Quick bail: no Chinese characters means regular matching is sufficient
    if !units.iter().any(|u| matches!(u, PyUnit::Han(_))) {
        return false;
    }

    let kw = keyword.to_ascii_lowercase();
    let kw_bytes = kw.as_bytes();

    // Try matching starting from each unit position (substring behavior)
    (0..units.len()).any(|start| match_from(&units[start..], kw_bytes))
}

/// Recursively try to consume the entire keyword against consecutive units.
fn match_from(units: &[PyUnit], kw: &[u8]) -> bool {
    if kw.is_empty() {
        return true;
    }
    if units.is_empty() {
        return false;
    }

    match &units[0] {
        PyUnit::Han(full) => {
            let fb = full.as_bytes();
            // Try full pinyin first
            if kw.len() >= fb.len() && kw[..fb.len()] == *fb && match_from(&units[1..], &kw[fb.len()..])
            {
                return true;
            }
            // Try initial only
            kw[0] == fb[0] && match_from(&units[1..], &kw[1..])
        }
        PyUnit::Ascii(b) => kw[0] == *b && match_from(&units[1..], &kw[1..]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Core examples from the requirement ──────────────────────────

    #[test]
    fn full_pinyin() {
        assert!(pinyin_match("工作填报", "gongzuotianbao"));
    }

    #[test]
    fn partial_full_pinyin() {
        assert!(pinyin_match("工作填报", "zuotian"));
    }

    #[test]
    fn full_initials() {
        assert!(pinyin_match("工作填报", "gztb"));
    }

    #[test]
    fn partial_initials() {
        assert!(pinyin_match("工作填报", "zt"));
    }

    // ── Mixed mode (initials + full pinyin) ─────────────────────────

    #[test]
    fn mixed_initials_then_full() {
        // g(工) z(作) tian(填) bao(报)
        assert!(pinyin_match("工作填报", "gztianbao"));
    }

    #[test]
    fn mixed_full_then_initials() {
        // gong(工) zuo(作) t(填) b(报)
        assert!(pinyin_match("工作填报", "gongzuotb"));
    }

    // ── Chinese + English mixed names ───────────────────────────────

    #[test]
    fn mixed_name_chinese_english() {
        // V2Ray配置 → units: v,2,r,a,y,pei,zhi
        assert!(pinyin_match("V2Ray配置", "v2raypz"));
        assert!(pinyin_match("V2Ray配置", "v2raypeizhi"));
        assert!(pinyin_match("V2Ray配置", "pz"));
        assert!(pinyin_match("V2Ray配置", "peizhi"));
    }

    #[test]
    fn english_prefix_chinese_suffix() {
        assert!(pinyin_match("Google搜索", "googless"));
        assert!(pinyin_match("Google搜索", "sousuo"));
        assert!(pinyin_match("Google搜索", "ss"));
    }

    // ── Edge cases ──────────────────────────────────────────────────

    #[test]
    fn empty_keyword_returns_false() {
        assert!(!pinyin_match("工作填报", ""));
    }

    #[test]
    fn pure_ascii_name_returns_false() {
        // No Chinese chars → pinyin matching is not needed
        assert!(!pinyin_match("Google Docs", "google"));
    }

    #[test]
    fn non_ascii_keyword_returns_false() {
        assert!(!pinyin_match("工作填报", "工作"));
    }

    #[test]
    fn single_char_initial() {
        assert!(pinyin_match("工作填报", "g"));
    }

    #[test]
    fn single_char_full() {
        assert!(pinyin_match("工作填报", "gong"));
    }

    #[test]
    fn no_match() {
        assert!(!pinyin_match("工作填报", "xyz"));
    }

    #[test]
    fn keyword_longer_than_all_pinyin() {
        assert!(!pinyin_match("工作", "gongzuotianbao"));
    }

    #[test]
    fn case_insensitive() {
        assert!(pinyin_match("工作填报", "GZTB"));
        assert!(pinyin_match("工作填报", "GongZuo"));
    }

    #[test]
    fn name_with_spaces_and_punctuation() {
        // Spaces and punctuation are skipped in unit extraction
        assert!(pinyin_match("工作 - 填报", "gztb"));
        assert!(pinyin_match("工作（填报）", "tianbao"));
    }

    #[test]
    fn tail_match() {
        assert!(pinyin_match("工作填报", "bao"));
        assert!(pinyin_match("工作填报", "b"));
        assert!(pinyin_match("工作填报", "tb"));
    }

    // ── Real-world bookmark names ───────────────────────────────────

    #[test]
    fn real_world_examples() {
        assert!(pinyin_match("百度一下", "baidu"));
        assert!(pinyin_match("百度一下", "bdyx"));
        assert!(pinyin_match("淘宝网", "taobao"));
        assert!(pinyin_match("淘宝网", "tbw"));
        assert!(pinyin_match("知乎 - 有问题就会有答案", "zhihu"));
        assert!(pinyin_match("知乎 - 有问题就会有答案", "zh"));
    }
}
