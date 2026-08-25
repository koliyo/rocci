use crate::span::Span;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OriginKind {
    OrdinaryRoc,
    ComponentSignature,
    Directive,
    RouteHeader,
    ComponentTag,
    TextExpression,
    AttributeExpression,
    StaticMarkup,
    Scaffolding,
    Css,
    MarkdownStructure,
    MarkdownText,
    MarkdownBoilerplate,
    PageRoc,
    RocBlock,
    RenderRoc,
}

impl OriginKind {
    pub fn maps_roc_semantics(self) -> bool {
        matches!(
            self,
            Self::OrdinaryRoc
                | Self::ComponentSignature
                | Self::Directive
                | Self::RouteHeader
                | Self::TextExpression
                | Self::AttributeExpression
                | Self::PageRoc
                | Self::RocBlock
                | Self::RenderRoc
        )
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::OrdinaryRoc => "ordinary_roc",
            Self::ComponentSignature => "component_signature",
            Self::Directive => "directive",
            Self::RouteHeader => "route_header",
            Self::ComponentTag => "component_tag",
            Self::TextExpression => "text_expression",
            Self::AttributeExpression => "attribute_expression",
            Self::StaticMarkup => "static_markup",
            Self::Scaffolding => "scaffolding",
            Self::Css => "css",
            Self::MarkdownStructure => "markdown_structure",
            Self::MarkdownText => "markdown_text",
            Self::MarkdownBoilerplate => "markdown_boilerplate",
            Self::PageRoc => "page_roc",
            Self::RocBlock => "roc_block",
            Self::RenderRoc => "render_roc",
        }
    }
}

impl fmt::Display for OriginKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    pub generated: Span,
    pub source: Span,
    pub origin: OriginKind,
}

impl Segment {
    pub fn new(generated: Span, source: Span, origin: OriginKind) -> Self {
        Self {
            generated,
            source,
            origin,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MappedOffset {
    pub offset: u32,
    pub origin: OriginKind,
}

pub fn source_to_generated(
    source: &str,
    generated: &str,
    segments: &[Segment],
    source_offset: u32,
) -> Option<MappedOffset> {
    map_offset(source, generated, segments, source_offset, true)
}

pub fn generated_to_source(
    source: &str,
    generated: &str,
    segments: &[Segment],
    generated_offset: u32,
) -> Option<MappedOffset> {
    map_offset(source, generated, segments, generated_offset, false)
}

pub fn map_source_span(
    source: &str,
    generated: &str,
    segments: &[Segment],
    span: Span,
) -> Option<Span> {
    map_span(source, generated, segments, span, true)
}

pub fn map_generated_span(
    source: &str,
    generated: &str,
    segments: &[Segment],
    span: Span,
) -> Option<Span> {
    map_span(source, generated, segments, span, false)
}

pub(crate) fn remap_segments(map: &[Option<u32>], segments: &[Segment]) -> Vec<Segment> {
    segments
        .iter()
        .filter_map(|segment| {
            let generated = remap_span(map, segment.generated)?;
            Some(Segment::new(generated, segment.source, segment.origin))
        })
        .collect()
}

fn map_offset(
    source: &str,
    generated: &str,
    segments: &[Segment],
    offset: u32,
    from_source: bool,
) -> Option<MappedOffset> {
    let mut best: Option<(u32, u32, MappedOffset)> = None;
    for segment in segments {
        if !segment.origin.maps_roc_semantics() {
            continue;
        }
        let containing = if from_source {
            segment.source
        } else {
            segment.generated
        };
        if !containing.contains(offset) {
            continue;
        }
        let source_slice = segment.source.of(source);
        let generated_slice = segment.generated.of(generated);
        let local = if from_source {
            offset.saturating_sub(segment.source.start)
        } else {
            offset.saturating_sub(segment.generated.start)
        };
        let mapped_local = if from_source {
            align_local(source_slice, generated_slice, local)
        } else {
            align_local(generated_slice, source_slice, local)
        };
        let Some(mapped_local) = mapped_local else {
            continue;
        };
        let dest_start = if from_source {
            segment.generated.start
        } else {
            segment.source.start
        };
        let mapped = MappedOffset {
            offset: dest_start + mapped_local,
            origin: segment.origin,
        };
        let key = (segment.source.len() as u32, segment.generated.len() as u32);
        let replace = match best {
            None => true,
            Some((source_len, generated_len, _)) => {
                key.0 < source_len || (key.0 == source_len && key.1 <= generated_len)
            }
        };
        if replace {
            best = Some((key.0, key.1, mapped));
        }
    }
    best.map(|(_, _, mapped)| mapped)
}

fn map_span(
    source: &str,
    generated: &str,
    segments: &[Segment],
    span: Span,
    from_source: bool,
) -> Option<Span> {
    if span.is_empty() {
        let mapped = map_offset(source, generated, segments, span.start, from_source)?;
        return Some(Span {
            start: mapped.offset,
            end: mapped.offset,
        });
    }
    let last = span.end.saturating_sub(1);
    let start = map_offset(source, generated, segments, span.start, from_source)?;
    let end = map_offset(source, generated, segments, last, from_source)?;
    let lo = start.offset.min(end.offset);
    let hi = start.offset.max(end.offset) + 1;
    Some(Span { start: lo, end: hi })
}

fn align_local(from: &str, to: &str, local: u32) -> Option<u32> {
    let local = local as usize;
    if local > from.len() {
        return None;
    }
    if from == to {
        return Some(local as u32);
    }
    if to == from.trim() {
        let lead = from.len() - from.trim_start().len();
        let interior_end = lead + to.len();
        if local < lead || local > interior_end {
            return None;
        }
        return Some((local - lead) as u32);
    }
    if from == to.trim() {
        let lead = to.len() - to.trim_start().len();
        return Some((local + lead) as u32);
    }
    if let Some(idx) = unique_substring(from, to) {
        if local < idx || local > idx + to.len() {
            return None;
        }
        return Some((local - idx) as u32);
    }
    if let Some(idx) = unique_substring(to, from) {
        return Some((local + idx) as u32);
    }
    None
}

fn unique_substring(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() {
        return None;
    }
    let first = haystack.find(needle)?;
    if haystack[first + needle.len()..].contains(needle) {
        return None;
    }
    Some(first)
}

fn remap_span(map: &[Option<u32>], span: Span) -> Option<Span> {
    if span.is_empty() {
        let start = map_point(map, span.start)?;
        return Some(Span { start, end: start });
    }
    let start_i = span.start as usize;
    let end_i = (span.end as usize).min(map.len());
    if start_i >= map.len() {
        return None;
    }
    let mut first = None;
    let mut last = None;
    for mapped in map.iter().take(end_i).skip(start_i).flatten().copied() {
        if first.is_none() {
            first = Some(mapped);
        }
        last = Some(mapped);
    }
    Some(Span {
        start: first?,
        end: last? + 1,
    })
}

fn map_point(map: &[Option<u32>], offset: u32) -> Option<u32> {
    let i = offset as usize;
    if i < map.len() {
        if let Some(mapped) = map[i] {
            return Some(mapped);
        }
    }
    if i > 0 {
        return map.get(i - 1).copied().flatten().map(|mapped| mapped + 1);
    }
    None
}
