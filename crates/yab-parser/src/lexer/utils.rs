use std::{iter::Peekable, str::Chars};

/// Predicate to check if a character is a line terminator, as defined by the
/// Ecmascript standard.
///
/// See: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar#line_terminators
pub fn is_line_terminator(c: char) -> bool {
    c == '\n' || c == '\r' || c == '\u{2028}' || c == '\u{2029}'
}

/// Trait that can be implemented by operators and punctuators to look up how
/// many members of the enum have lexemes that start with a particular prefix.
/// This allows us to cheaply query in the tokenizer whether a given sequence
/// can be parsed as an operator / puntuator / etc.
pub trait HasPrefixLookup {
    fn fields_starting_with(ident: &str) -> usize;
}

pub fn try_parse_from_prefix_lookup<T>(chars: &mut Peekable<Chars>) -> Option<T>
where
    for<'a> T: HasPrefixLookup + TryFrom<&'a str>,
    for<'a> <T as TryFrom<&'a str>>::Error: core::fmt::Debug,
{
    match chars.peek() {
        Some(c) => {
            // While the iterator is still at {c}, create a potential lexeme out
            // of the character without consuming it.
            let mut prefix_lexeme = String::from(*c);
            let prefix_matches = T::fields_starting_with(&prefix_lexeme);

            if prefix_matches > 0 {
                // If we have at least one match, then we are safe to progress
                // and consume the character we just used on the lexeme.
                _ = chars.next();

                // While we can continue getting characters, check if adding the
                // next character would still give us a valid operator.
                'prefix: while let Some(next_char) = chars.peek() {
                    // We might strip this off later, but tentatively push it
                    // onto the lexeme:
                    prefix_lexeme.push(*next_char);
                    let prefix_matches = T::fields_starting_with(&prefix_lexeme);

                    // If we went from > 0 to 0, then we've gone one character
                    // too far, so strip off the character we just added and
                    // return.
                    if prefix_matches == 0 {
                        prefix_lexeme = prefix_lexeme[..prefix_lexeme.len() - 1].to_string();
                        break 'prefix;
                    } else {
                        // Otherwise the character is valid, so we can safely
                        // keep it in the lexeme and consume it for the next
                        // iteration.
                        _ = chars.next();
                        continue 'prefix;
                    }
                }

                // We've broke out of the loop, either because we've run out of
                // characters altogether, or because we've found the longest
                // operator.
                let prefix_ref = prefix_lexeme.as_str();
                let operator_type = T::try_from(prefix_ref).unwrap();
                return Some(operator_type);
            }

            // If prefix matches == 0 from the very first character, don't
            // consume anything because we don't have a valid operator, and
            // return
            None
        }
        // In this case, we don't even have any characters in the iterator, so
        // return.
        _ => None,
    }
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
