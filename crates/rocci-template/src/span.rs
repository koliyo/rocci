use std::fmt;

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
        let start = (self.start as usize).min(src.len());
        let end = (self.end as usize).min(src.len());
        &src[start..end]
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
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
}
