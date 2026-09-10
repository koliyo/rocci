use crate::PageRef;
use crate::scan::{fence_open, is_fence_close, skip_0_3_spaces};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WikiLinkContext {
    pub prefix: String,
    pub heading_prefix: Option<String>,
    pub label_prefix: Option<String>,
}

pub(crate) fn wiki_link_context(src: &str, offset: usize) -> Option<WikiLinkContext> {
    let offset = offset.min(src.len());
    if in_code(src, offset) {
        return None;
    }
    let open = find_open_wiki(src, offset)?;
    let inner_start = open + 2;
    if offset < inner_start {
        return None;
    }
    if src[inner_start..offset].contains('\n') {
        return None;
    }
    if let Some(close) = src[inner_start..].find("]]") {
        let close_at = inner_start + close;
        if offset >= close_at {
            return None;
        }
    }
    Some(parse_wiki_typed(&src[inner_start..offset]))
}

fn parse_wiki_typed(typed: &str) -> WikiLinkContext {
    if let Some(bar) = typed.find('|') {
        let (prefix, heading_prefix) = split_heading(&typed[..bar]);
        return WikiLinkContext {
            prefix,
            heading_prefix,
            label_prefix: Some(typed[bar + 1..].to_string()),
        };
    }
    let (prefix, heading_prefix) = split_heading(typed);
    WikiLinkContext {
        prefix,
        heading_prefix,
        label_prefix: None,
    }
}

fn split_heading(target: &str) -> (String, Option<String>) {
    match target.find('#') {
        Some(hash) => (
            target[..hash].to_string(),
            Some(target[hash + 1..].to_string()),
        ),
        None => (target.to_string(), None),
    }
}

fn find_open_wiki(src: &str, offset: usize) -> Option<usize> {
    let mut search_end = offset;
    while search_end > 0 {
        let Some(idx) = src[..search_end].rfind("[[") else {
            return None;
        };
        let before = idx;
        if in_code(src, idx) {
            search_end = idx;
            continue;
        }
        if src[idx + 2..offset].contains("]]") {
            search_end = idx;
            continue;
        }
        return Some(before);
    }
    None
}

fn in_code(src: &str, offset: usize) -> bool {
    let mut pos = 0;
    let mut fence: Option<(u8, usize)> = None;
    while pos < src.len() {
        let before = pos;
        let line_start = pos;
        let nl = src[pos..].find('\n').map(|i| pos + i);
        let line_end = nl.unwrap_or(src.len());
        let next = nl.map(|i| i + 1).unwrap_or(src.len());
        let line = &src[line_start..line_end];

        if let Some((ch, n)) = fence {
            if offset >= line_start && offset < next {
                return true;
            }
            if is_fence_close(line, ch, n) {
                fence = None;
            }
            pos = next;
            debug_assert!(pos > before);
            continue;
        }

        let stripped = skip_0_3_spaces(line);
        if let Some(open) = fence_open(stripped) {
            if offset >= line_start && offset < next {
                return true;
            }
            fence = Some(open);
            pos = next;
            debug_assert!(pos > before);
            continue;
        }

        if offset >= line_start
            && offset <= line_end
            && inline_code_contains(line, offset - line_start)
        {
            return true;
        }

        pos = next;
        if pos == before {
            pos += 1;
        }
    }
    false
}

fn inline_code_contains(line: &str, rel: usize) -> bool {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let before = i;
        if bytes[i] != b'`' {
            i += 1;
            continue;
        }
        let n = bytes[i..].iter().take_while(|b| **b == b'`').count();
        let content_start = i + n;
        if let Some(close) = find_closing_backticks(&bytes[content_start..], n) {
            let span_end = content_start + close + n;
            if rel >= i && rel < span_end {
                return true;
            }
            i = span_end;
        } else {
            i += n;
        }
        if i <= before {
            i = before + 1;
        }
    }
    false
}

fn find_closing_backticks(bytes: &[u8], n: usize) -> Option<usize> {
    let mut i = 0;
    while i < bytes.len() {
        let before = i;
        if bytes[i] != b'`' {
            i += 1;
            continue;
        }
        let m = bytes[i..].iter().take_while(|b| **b == b'`').count();
        if m == n {
            return Some(i);
        }
        i += m;
        if i <= before {
            i = before + 1;
        }
    }
    None
}

pub(crate) fn wiki_page_keys(pages: &[PageRef], prefix: &str) -> Vec<(String, String)> {
    let mut counts = std::collections::HashMap::<&str, usize>::new();
    for page in pages {
        *counts.entry(page.wiki_key()).or_insert(0) += 1;
    }
    let mut items: Vec<(String, String)> = pages
        .iter()
        .filter(|page| counts.get(page.wiki_key()).copied().unwrap_or(0) == 1)
        .filter(|page| page.wiki_key().starts_with(prefix))
        .map(|page| (page.wiki_key().to_string(), page.route.clone()))
        .collect();
    items.sort_by(|a, b| a.0.cmp(&b.0));
    items.dedup_by(|a, b| a.0 == b.0);
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(src: &str, at: &str) -> Option<WikiLinkContext> {
        let offset = src
            .find(at)
            .unwrap_or_else(|| panic!("missing {at:?} in {src:?}"))
            + at.len();
        wiki_link_context(src, offset)
    }

    fn prefix(src: &str, at: &str) -> Option<String> {
        ctx(src, at).map(|c| c.prefix)
    }

    #[test]
    fn open_brackets_have_empty_prefix() {
        assert_eq!(prefix("See [[", "[["), Some(String::new()));
    }

    #[test]
    fn incomplete_page_prefix() {
        let found = ctx("See [[pre", "[[pre").expect("wiki");
        assert_eq!(found.prefix, "pre");
        assert_eq!(found.heading_prefix, None);
        assert_eq!(found.label_prefix, None);
    }

    #[test]
    fn heading_prefix_after_hash() {
        let found = ctx("[[Page#", "[[Page#").expect("wiki");
        assert_eq!(found.prefix, "Page");
        assert_eq!(found.heading_prefix.as_deref(), Some(""));
        assert_eq!(found.label_prefix, None);
    }

    #[test]
    fn heading_prefix_partial() {
        let found = ctx("[[Page#he", "[[Page#he").expect("wiki");
        assert_eq!(found.prefix, "Page");
        assert_eq!(found.heading_prefix.as_deref(), Some("he"));
    }

    #[test]
    fn closed_wiki_counts_with_cursor_in_target() {
        let src = "See [[Page]] now";
        let start = src.find("[[").expect("open") + 2;
        let at_pa = start + 2;
        let found = wiki_link_context(src, at_pa).expect("inside closed wiki");
        assert_eq!(found.prefix, "Pa");
        assert!(wiki_link_context(src, src.find("]]").expect("close")).is_none());
    }

    #[test]
    fn colon_kind_bracket_is_not_wiki() {
        assert_eq!(wiki_link_context(":note[", 6), None);
        assert_eq!(prefix(":note[title", ":note["), None);
    }

    #[test]
    fn inline_code_is_not_wiki() {
        assert_eq!(prefix("`[[x`", "`[[x"), None);
        assert_eq!(prefix("before `[[x` after", "`[[x"), None);
    }

    #[test]
    fn fenced_code_is_not_wiki() {
        let src = "```\n[[pre\n```\n";
        let offset = src.find("[[pre").expect("fence wiki") + "[[pre".len();
        assert_eq!(wiki_link_context(src, offset), None);
    }

    #[test]
    fn label_prefix_after_bar() {
        let found = ctx("[[Page|la", "[[Page|la").expect("wiki");
        assert_eq!(found.prefix, "Page");
        assert_eq!(found.label_prefix.as_deref(), Some("la"));
    }
}
