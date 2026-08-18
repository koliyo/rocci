/// Escape HTML special characters (&, <, >, ", ').
pub fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape() {
        assert_eq!(
            escape("<script>alert(\"xss\") & 'foo'</script>"),
            "&lt;script&gt;alert(&quot;xss&quot;) &amp; &#39;foo&#39;&lt;/script&gt;"
        );
    }
}
