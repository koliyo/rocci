use std::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PositionEncoding {
    Utf8,
    #[default]
    Utf16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start: start as u32,
            end: end as u32,
        }
    }

    pub fn point(offset: usize) -> Self {
        Self::new(offset, offset)
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    pub fn len(self) -> usize {
        self.end.saturating_sub(self.start) as usize
    }

    pub fn as_range(self) -> std::ops::Range<usize> {
        self.start as usize..self.end as usize
    }

    pub fn of(self, src: &str) -> &str {
        let start = floor_char_boundary(src, (self.start as usize).min(src.len()));
        let end = floor_char_boundary(src, (self.end as usize).min(src.len()));
        if start >= end { "" } else { &src[start..end] }
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn contains(self, offset: u32) -> bool {
        self.start <= offset && offset <= self.end
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SourceFile<'a> {
    pub name: &'a str,
    pub src: &'a str,
}

impl<'a> SourceFile<'a> {
    pub fn new(name: &'a str, src: &'a str) -> Self {
        Self { name, src }
    }

    pub fn slice(self, span: Span) -> &'a str {
        span.of(self.src)
    }

    pub fn line_col(self, offset: u32) -> (u32, u32) {
        let mut line = 1;
        let mut col = 1;
        for (i, ch) in self.src.char_indices() {
            if i as u32 >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    pub fn position(self, offset: u32, encoding: PositionEncoding) -> (u32, u32) {
        let offset = floor_char_boundary(self.src, (offset as usize).min(self.src.len()));
        let mut line = 0u32;
        let mut line_start = 0usize;
        for (i, ch) in self.src.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                line_start = i + 1;
            }
        }
        (line, count_units(&self.src[line_start..offset], encoding))
    }

    pub fn offset_at(self, line: u32, character: u32, encoding: PositionEncoding) -> u32 {
        let Some(line_start) = line_start_offset(self.src, line) else {
            return self.src.len() as u32;
        };
        let rest = &self.src[line_start..];
        let line_end = rest
            .find('\n')
            .map(|i| line_start + i)
            .unwrap_or(self.src.len());
        let line_text = &self.src[line_start..line_end];
        (line_start + units_to_bytes(line_text, character, encoding)) as u32
    }

    pub fn range(self, span: Span, encoding: PositionEncoding) -> ((u32, u32), (u32, u32)) {
        (
            self.position(span.start, encoding),
            self.position(span.end, encoding),
        )
    }
}

fn floor_char_boundary(src: &str, mut index: usize) -> usize {
    index = index.min(src.len());
    while index > 0 && !src.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn line_start_offset(src: &str, line: u32) -> Option<usize> {
    if line == 0 {
        return Some(0);
    }
    let mut current = 0u32;
    for (i, ch) in src.char_indices() {
        if ch == '\n' {
            current += 1;
            if current == line {
                return Some(i + 1);
            }
        }
    }
    None
}

fn count_units(text: &str, encoding: PositionEncoding) -> u32 {
    match encoding {
        PositionEncoding::Utf8 => text.len() as u32,
        PositionEncoding::Utf16 => text.chars().map(utf16_len).sum(),
    }
}

fn units_to_bytes(text: &str, character: u32, encoding: PositionEncoding) -> usize {
    match encoding {
        PositionEncoding::Utf8 => floor_char_boundary(text, (character as usize).min(text.len())),
        PositionEncoding::Utf16 => {
            let mut units = 0u32;
            for (i, ch) in text.char_indices() {
                let width = utf16_len(ch);
                if units + width > character {
                    return i;
                }
                units += width;
                if units == character {
                    return i + ch.len_utf8();
                }
            }
            text.len()
        }
    }
}

fn utf16_len(ch: char) -> u32 {
    if (ch as u32) > 0xFFFF { 2 } else { 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_and_utf16_differ_on_non_bmp() {
        let src = "a😀b";
        let file = SourceFile::new("t.rocci", src);
        let offset_b = src.find('b').unwrap() as u32;
        assert_eq!(file.position(offset_b, PositionEncoding::Utf8), (0, 5));
        assert_eq!(file.position(offset_b, PositionEncoding::Utf16), (0, 3));
        assert_eq!(file.offset_at(0, 5, PositionEncoding::Utf8), offset_b);
        assert_eq!(file.offset_at(0, 3, PositionEncoding::Utf16), offset_b);
    }

    #[test]
    fn multiline_positions_are_zero_based() {
        let src = "hello\nworld";
        let file = SourceFile::new("t.rocci", src);
        let offset_w = src.find('w').unwrap() as u32;
        assert_eq!(file.position(offset_w, PositionEncoding::Utf8), (1, 0));
        assert_eq!(file.offset_at(1, 0, PositionEncoding::Utf8), offset_w);
        assert_eq!(file.line_col(offset_w), (2, 1));
    }
}
