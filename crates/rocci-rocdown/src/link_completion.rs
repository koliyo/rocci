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
        if offset > close_at {
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
        let idx = src[..search_end].rfind("[[")?;
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
    let mut items: Vec<(String, String)> = Vec::new();
    for page in pages {
        for key in page.wiki_candidate_keys() {
            if !starts_with_ignore_ascii(key, prefix) {
                continue;
            }
            let crate::links::WikiMatch::One(hit) =
                crate::links::match_wiki(key, pages, PageRef::wiki_identity)
            else {
                continue;
            };
            if hit.path != page.path {
                continue;
            }
            items.push((key.to_string(), page.route.clone()));
        }
    }
    items.sort_by(|a, b| a.0.cmp(&b.0));
    items.dedup_by(|a, b| a.0 == b.0);
    items
}

fn starts_with_ignore_ascii(value: &str, prefix: &str) -> bool {
    value
        .get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownDestContext {
    pub path_prefix: String,
    pub heading_prefix: Option<String>,
}

pub(crate) fn markdown_dest_context(src: &str, offset: usize) -> Option<MarkdownDestContext> {
    let offset = offset.min(src.len());
    if in_code(src, offset) {
        return None;
    }
    let dest_start = find_open_markdown_dest(src, offset)?;
    if offset < dest_start {
        return None;
    }
    let typed = &src[dest_start..offset];
    if typed.contains('\n')
        || typed.contains('?')
        || typed.chars().any(|ch| ch.is_whitespace())
        || typed.contains(')')
    {
        return None;
    }
    let (path_prefix, heading_prefix) = split_heading(typed);
    if crate::links::has_scheme(&path_prefix) {
        return None;
    }
    Some(MarkdownDestContext {
        path_prefix,
        heading_prefix,
    })
}

fn find_open_markdown_dest(src: &str, offset: usize) -> Option<usize> {
    let mut search_end = offset;
    while search_end > 0 {
        let idx = src[..search_end].rfind("](")?;
        if in_code(src, idx) {
            search_end = idx;
            continue;
        }
        if src[idx + 2..offset].contains(')') {
            search_end = idx;
            continue;
        }
        let Some(label_open) = src[..idx].rfind('[') else {
            search_end = idx;
            continue;
        };
        if label_open > 0 && src.as_bytes()[label_open - 1] == b'!' {
            search_end = idx;
            continue;
        }
        return Some(idx + 2);
    }
    None
}

pub(crate) fn heading_keys(
    ids: impl IntoIterator<Item = impl AsRef<str>>,
    prefix: &str,
) -> Vec<String> {
    let mut items: Vec<String> = ids
        .into_iter()
        .map(|id| id.as_ref().to_string())
        .filter(|id| id.starts_with(prefix) && !crate::links::is_source_line_anchor_id(id))
        .collect();
    items.sort();
    items.dedup();
    items
}

pub(crate) fn markdown_dest_keys(
    pages: &[PageRef],
    current: Option<&PageRef>,
    prefix: &str,
) -> Vec<(String, String)> {
    let mut items = Vec::new();
    for page in pages {
        for route in published_routes(page) {
            if route.starts_with(prefix) {
                items.push((route, page.route.clone()));
            }
        }
        if let Some(from) = current
            && let Some(rel) = relative_doc_path(from, page)
            && rel.starts_with(prefix)
        {
            items.push((rel, page.route.clone()));
        }
    }
    items.sort_by(|a, b| a.0.cmp(&b.0));
    items.dedup_by(|a, b| a.0 == b.0);
    items
}

fn published_routes(page: &PageRef) -> Vec<String> {
    let collection = crate::catalog::is_collection_id(&page.id) || page.stem == "index";
    let canonical = crate::catalog::canonical_route(&page.route, collection);
    let mut routes = vec![canonical.clone()];
    if canonical.starts_with('/') && !canonical.starts_with("/docs") {
        let docs = if canonical == "/" {
            "/docs/".to_string()
        } else {
            crate::catalog::with_trailing_slash(&format!("/docs{canonical}"))
        };
        routes.push(docs);
    } else if canonical.starts_with("/docs") {
        routes.push(crate::catalog::with_trailing_slash(&canonical));
    }
    routes.sort();
    routes.dedup();
    routes
}

fn relative_doc_path(from: &PageRef, to: &PageRef) -> Option<String> {
    if from.path == to.path {
        return None;
    }
    let from_dir = from.path.parent()?;
    let rel = to.path.strip_prefix(from_dir).ok()?;
    let mut text = rel.to_string_lossy().replace('\\', "/");
    if text.is_empty() || text == "." {
        return None;
    }
    if text.contains("..") {
        return None;
    }
    if text.contains('/') && !text.starts_with("./") {
        text = format!("./{text}");
    }
    Some(text)
}

pub(crate) fn page_for_wiki_key<'a>(pages: &'a [PageRef], key: &str) -> Option<&'a PageRef> {
    crate::links::unique_wiki_page(pages, key)
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
        let after_close = src.find("]]").expect("close") + 2;
        assert!(wiki_link_context(src, after_close).is_none());
    }

    #[test]
    fn empty_closed_wiki_completes_inside() {
        let src = "[[]]";
        let found = wiki_link_context(src, 2).expect("empty wiki");
        assert_eq!(found.prefix, "");
    }

    #[test]
    fn wiki_keys_include_unique_title_for_index_pages() {
        let pages = vec![
            PageRef {
                stem: "index".into(),
                file_name: "index.rocdown".into(),
                path: "applications/index.rocdown".into(),
                route: "/applications/".into(),
                explicit_route: false,
                heading_ids: Vec::new(),
                id: "applications/index".into(),
                title: "Applications".into(),
            },
            PageRef {
                stem: "index".into(),
                file_name: "index.rocdown".into(),
                path: "templates/index.rocdown".into(),
                route: "/templates/".into(),
                explicit_route: false,
                heading_ids: Vec::new(),
                id: "templates/index".into(),
                title: "Templates".into(),
            },
        ];
        let keys: Vec<String> = wiki_page_keys(&pages, "A")
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        assert_eq!(keys, vec!["Applications".to_string()]);
        let all: Vec<String> = wiki_page_keys(&pages, "")
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        assert!(all.contains(&"Applications".to_string()), "{all:?}");
        assert!(all.contains(&"Templates".to_string()), "{all:?}");
        assert!(!all.iter().any(|key| key == "index"), "{all:?}");
    }

    #[test]
    fn wiki_keys_include_unique_file_stem() {
        let pages = vec![
            PageRef {
                stem: "index".into(),
                file_name: "index.rocdown".into(),
                path: "applications/index.rocdown".into(),
                route: "/applications/".into(),
                explicit_route: false,
                heading_ids: Vec::new(),
                id: "applications/index".into(),
                title: "Applications".into(),
            },
            PageRef {
                stem: "handlers".into(),
                file_name: "handlers.rocdown".into(),
                path: "applications/handlers.rocdown".into(),
                route: "/applications/handlers".into(),
                explicit_route: false,
                heading_ids: Vec::new(),
                id: "applications/handlers".into(),
                title: "Handlers".into(),
            },
        ];
        let keys: Vec<String> = wiki_page_keys(&pages, "h")
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        assert!(keys.contains(&"handlers".to_string()), "{keys:?}");
        assert!(keys.contains(&"Handlers".to_string()), "{keys:?}");
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

    #[test]
    fn markdown_dest_heading_prefix() {
        let src = "[x](Page#he";
        let offset = src.len();
        let found = markdown_dest_context(src, offset).expect("dest");
        assert_eq!(found.path_prefix, "Page");
        assert_eq!(found.heading_prefix.as_deref(), Some("he"));
    }

    #[test]
    fn markdown_same_page_heading() {
        let src = "[x](#he";
        let found = markdown_dest_context(src, src.len()).expect("dest");
        assert_eq!(found.path_prefix, "");
        assert_eq!(found.heading_prefix.as_deref(), Some("he"));
    }

    #[test]
    fn markdown_image_dest_is_ignored() {
        let src = "![x](Page#he";
        assert_eq!(markdown_dest_context(src, src.len()), None);
    }
}
