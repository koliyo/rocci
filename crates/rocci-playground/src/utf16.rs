//! UTF-8 byte offset to UTF-16 code unit offset conversion helpers.

/// Converts a UTF-8 byte offset in `src` to a UTF-16 code unit offset.
///
/// If `byte_offset` exceeds `src.len()`, it is clamped to `src.len()`.
/// If `byte_offset` falls within a multi-byte UTF-8 sequence, it is rounded down
/// to the nearest char boundary before computing the UTF-16 offset.
pub fn byte_to_utf16_offset(src: &str, byte_offset: usize) -> usize {
    let clamped_byte = byte_offset.min(src.len());
    let valid_byte = if src.is_char_boundary(clamped_byte) {
        clamped_byte
    } else {
        let mut b = clamped_byte;
        while b > 0 && !src.is_char_boundary(b) {
            b -= 1;
        }
        b
    };

    let slice = &src[..valid_byte];
    let mut utf16_len = 0;
    for ch in slice.chars() {
        utf16_len += ch.len_utf16();
    }
    utf16_len
}

/// Converts a byte range `[start_byte, end_byte]` in `src` to a UTF-16 code unit range `(from, to)`.
pub fn byte_range_to_utf16(src: &str, start_byte: usize, end_byte: usize) -> (usize, usize) {
    let from = byte_to_utf16_offset(src, start_byte);
    let to = byte_to_utf16_offset(src, end_byte.max(start_byte));
    (from, to)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii() {
        let s = "Hello, world!";
        assert_eq!(byte_to_utf16_offset(s, 0), 0);
        assert_eq!(byte_to_utf16_offset(s, 5), 5);
        assert_eq!(byte_to_utf16_offset(s, 13), 13);
        assert_eq!(byte_to_utf16_offset(s, 100), 13);
    }

    #[test]
    fn test_bmp_multibyte() {
        // 'ä' is 2 bytes in UTF-8, 1 code unit in UTF-16
        // '€' is 3 bytes in UTF-8, 1 code unit in UTF-16
        let s = "ä€b";
        assert_eq!(s.len(), 6); // 2 + 3 + 1
        assert_eq!(byte_to_utf16_offset(s, 0), 0);
        assert_eq!(byte_to_utf16_offset(s, 2), 1); // after 'ä'
        assert_eq!(byte_to_utf16_offset(s, 5), 2); // after '€'
        assert_eq!(byte_to_utf16_offset(s, 6), 3); // after 'b'
    }

    #[test]
    fn test_non_bmp_surrogate_pair() {
        // '🎉' (U+1F389) is 4 bytes in UTF-8, 2 code units in UTF-16 (surrogate pair)
        let s = "Party 🎉 rock";
        assert_eq!(s.len(), 15); // 6 + 4 + 5 = 15 bytes
        assert_eq!(byte_to_utf16_offset(s, 0), 0);
        assert_eq!(byte_to_utf16_offset(s, 6), 6); // before '🎉'
        assert_eq!(byte_to_utf16_offset(s, 10), 8); // after '🎉' -> 6 + 2 = 8
        assert_eq!(byte_to_utf16_offset(s, 15), 13); // end of string -> 8 + 5 = 13

        let (from, to) = byte_range_to_utf16(s, 6, 10);
        assert_eq!(from, 6);
        assert_eq!(to, 8);
    }
}
