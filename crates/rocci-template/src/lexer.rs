use crate::span::Span;

pub struct Cursor<'a> {
    pub src: &'a str,
    pub pos: usize,
    pub paren: usize,
    pub bracket: usize,
    pub brace: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            pos: 0,
            paren: 0,
            bracket: 0,
            brace: 0,
        }
    }

    pub fn at(src: &'a str, pos: usize) -> Self {
        Self {
            src,
            pos,
            paren: 0,
            bracket: 0,
            brace: 0,
        }
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.src.len()
    }

    pub fn rest(&self) -> &'a str {
        &self.src[self.pos..]
    }

    pub fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    pub fn peek_at(&self, offset: usize) -> Option<char> {
        self.src[self.pos.saturating_add(offset)..].chars().next()
    }

    pub fn starts_with(&self, s: &str) -> bool {
        self.rest().starts_with(s)
    }

    pub fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    pub fn eat(&mut self, ch: char) -> bool {
        if self.peek() == Some(ch) {
            self.bump();
            true
        } else {
            false
        }
    }

    pub fn eat_str(&mut self, s: &str) -> bool {
        if self.starts_with(s) {
            self.pos += s.len();
            true
        } else {
            false
        }
    }

    pub fn is_top_level(&self) -> bool {
        self.paren == 0 && self.bracket == 0 && self.brace == 0
    }

    pub fn skip_spaces_tabs(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t')) {
            self.bump();
        }
    }

    pub fn skip_formatting_ws(&mut self) {
        loop {
            match self.peek() {
                Some('\n') | Some('\r') => {
                    if self.eat('\r') {
                        self.eat('\n');
                    } else {
                        self.bump();
                    }
                    self.skip_spaces_tabs();
                }
                _ => break,
            }
        }
    }

    pub fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.bump();
        }
    }

    pub fn skip_comment(&mut self) {
        if self.eat('#') {
            while let Some(ch) = self.peek() {
                if ch == '\n' {
                    break;
                }
                self.bump();
            }
        }
    }

    pub fn skip_trivia(&mut self) {
        loop {
            self.skip_whitespace();
            if self.peek() == Some('#') {
                self.skip_comment();
                continue;
            }
            break;
        }
    }

    pub fn skip_html_comment(&mut self) -> bool {
        if !self.eat_str("<!--") {
            return false;
        }
        while !self.is_eof() {
            if self.eat_str("-->") {
                return true;
            }
            self.bump();
        }
        true
    }

    pub fn skip_string(&mut self) {
        if self.starts_with("\"\"\"") {
            self.pos += 3;
            while !self.is_eof() {
                if self.starts_with("\"\"\"") {
                    self.pos += 3;
                    return;
                }
                if self.starts_with("${") {
                    self.pos += 2;
                    self.skip_balanced_braces_inner();
                    continue;
                }
                if self.peek() == Some('\\') {
                    self.bump();
                    self.bump();
                    continue;
                }
                self.bump();
            }
            return;
        }

        if !self.eat('"') {
            return;
        }
        while let Some(ch) = self.peek() {
            if ch == '\\' {
                self.bump();
                self.bump();
                continue;
            }
            if ch == '"' {
                self.bump();
                return;
            }
            if ch == '$'
                && self
                    .rest()
                    .get(1..)
                    .is_some_and(|rest| rest.starts_with('{'))
            {
                self.bump();
                self.bump();
                self.skip_balanced_braces_inner();
                continue;
            }
            self.bump();
        }
    }

    fn skip_balanced_braces_inner(&mut self) {
        let mut depth = 1;
        while !self.is_eof() && depth > 0 {
            match self.peek() {
                Some('"') => self.skip_string(),
                Some('#') => self.skip_comment(),
                Some('{') => {
                    self.bump();
                    depth += 1;
                }
                Some('}') => {
                    self.bump();
                    depth -= 1;
                }
                _ => {
                    self.bump();
                }
            }
        }
    }

    pub fn skip_number(&mut self) {
        if self.starts_with("0x") || self.starts_with("0X") {
            self.pos += 2;
            while self.peek().is_some_and(|ch| ch.is_ascii_hexdigit()) {
                self.bump();
            }
            return;
        }
        while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            self.bump();
        }
        if self.peek() == Some('.') && self.peek_at(1).is_some_and(|ch| ch.is_ascii_digit()) {
            self.bump();
            while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                self.bump();
            }
        }
    }

    pub fn scan_ident(&mut self) -> Option<Span> {
        let start = self.pos;
        let first = self.peek()?;
        if !is_ident_start(first) {
            return None;
        }
        self.bump();
        while self.peek().is_some_and(is_ident_continue) {
            self.bump();
        }
        Some(Span::new(start, self.pos))
    }

    pub fn scan_tag_name(&mut self) -> Option<Span> {
        let start = self.pos;
        let first = self.peek()?;
        if !first.is_ascii_alphabetic() {
            return None;
        }
        self.bump();
        while self
            .peek()
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
        {
            self.bump();
        }
        Some(Span::new(start, self.pos))
    }

    pub fn scan_attr_name(&mut self) -> Option<Span> {
        let start = self.pos;
        let first = self.peek()?;
        if !first.is_ascii_alphabetic() && first != '_' && first != ':' {
            return None;
        }
        self.bump();
        while self
            .peek()
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '.'))
        {
            self.bump();
        }
        Some(Span::new(start, self.pos))
    }

    pub fn ident_text(&self, span: Span) -> &'a str {
        span.of(self.src)
    }

    pub fn skip_roc_token(&mut self) {
        self.skip_trivia();
        match self.peek() {
            None => {}
            Some('"') => self.skip_string(),
            Some(ch) if is_ident_start(ch) => {
                self.scan_ident();
            }
            Some(ch) if ch.is_ascii_digit() => self.skip_number(),
            Some('(') => {
                self.bump();
                self.paren += 1;
            }
            Some(')') => {
                self.bump();
                self.paren = self.paren.saturating_sub(1);
            }
            Some('[') => {
                self.bump();
                self.bracket += 1;
            }
            Some(']') => {
                self.bump();
                self.bracket = self.bracket.saturating_sub(1);
            }
            Some('{') => {
                self.bump();
                self.brace += 1;
            }
            Some('}') => {
                self.bump();
                self.brace = self.brace.saturating_sub(1);
            }
            Some(_) => {
                self.bump();
            }
        }
    }

    pub fn skip_balanced_braces(&mut self) {
        if self.peek() != Some('{') {
            return;
        }
        let mut depth = 0;
        while !self.is_eof() {
            match self.peek() {
                Some('"') => self.skip_string(),
                Some('#') => self.skip_comment(),
                Some('{') => {
                    self.bump();
                    depth += 1;
                }
                Some('}') => {
                    self.bump();
                    depth -= 1;
                    if depth == 0 {
                        return;
                    }
                }
                _ => {
                    self.bump();
                }
            }
        }
    }
}

pub fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

pub fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

pub fn trim_span(src: &str, span: Span) -> Span {
    let slice = span.of(src);
    let leading = slice.len() - slice.trim_start().len();
    let trailing = slice.len() - slice.trim_end().len();
    let start = span.start as usize + leading;
    let end = (span.end as usize).saturating_sub(trailing);
    if start >= end {
        Span::new(start, start)
    } else {
        Span::new(start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_skip_string_handles_interpolations_and_unclosed() {
        let src = r#""hello ${name} world" and more"#;
        let mut cur = Cursor::at(src, 0);
        cur.skip_string();
        assert_eq!(cur.pos, r#""hello ${name} world""#.len());

        let unclosed = r#""unclosed string with ${nested"#;
        let mut cur2 = Cursor::at(unclosed, 0);
        cur2.skip_string();
        assert!(cur2.is_eof());
    }

    #[test]
    fn cursor_skip_balanced_braces_handles_unclosed_without_hanging() {
        let unclosed = "{ nested { unclosed";
        let mut cur = Cursor::at(unclosed, 0);
        cur.skip_balanced_braces();
        assert!(cur.is_eof());
    }
}
