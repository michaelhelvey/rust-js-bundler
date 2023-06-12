pub fn is_line_terminator(c: char) -> bool {
    c == '\n' || c == '\r' || c == '\u{2028}' || c == '\u{2029}'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_line_terminator() {
        assert!(is_line_terminator('\n'));
        assert!(is_line_terminator('\r'));
        assert!(is_line_terminator('\u{2028}'));
        assert!(is_line_terminator('\u{2029}'));
        assert!(!is_line_terminator('a'));
    }
}
