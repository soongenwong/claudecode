//! Tiny, dependency-light syntax tokenizer for the diff panel.
//!
//! The CLI already pulls in `syntect` for Markdown rendering, but mapping
//! syntect's terminal escapes onto `ratatui` spans (and blending them with the
//! per-line diff background) is fiddly and brittle across themes. For the
//! dashboard we only need *good enough* coloring of the languages we actually
//! stream, so we use a small hand-rolled tokenizer. It is deterministic, never
//! panics on partial input (diff lines are revealed character-by-character), and
//! produces `ratatui` spans directly.

use ratatui::style::{Color, Style};
use ratatui::text::Span;

/// A coarse token class. Mapped to a color by [`token_color`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Token {
    Keyword,
    Type,
    Str,
    Number,
    Comment,
    Punct,
    Ident,
}

fn token_color(token: Token) -> Color {
    match token {
        Token::Keyword => Color::Magenta,
        Token::Type => Color::Cyan,
        Token::Str => Color::Green,
        Token::Number => Color::LightYellow,
        Token::Comment => Color::DarkGray,
        Token::Punct => Color::Gray,
        Token::Ident => Color::White,
    }
}

const KEYWORDS: &[&str] = &[
    "fn", "let", "mut", "pub", "use", "mod", "struct", "enum", "impl", "trait", "for", "while",
    "loop", "if", "else", "match", "return", "self", "Self", "async", "await", "move", "ref",
    "const", "static", "where", "as", "in", "break", "continue", "type", "dyn", "crate", "super",
    "def", "class", "import", "from", "function", "var", "true", "false", "null", "None", "and",
    "or", "not",
];

fn is_type_name(word: &str) -> bool {
    let mut chars = word.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_uppercase()) && word.len() > 1
}

/// Split a source line into colored [`Span`]s, blending each token color with
/// the supplied per-line `base` style (which carries the diff background and any
/// dim/bold modifiers). Returns owned spans because diff lines are mutated as
/// they stream in.
pub fn highlight_line(line: &str, base: Style) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut chars = line.char_indices().peekable();

    let push = |spans: &mut Vec<Span<'static>>, text: String, token: Token| {
        if text.is_empty() {
            return;
        }
        spans.push(Span::styled(text, base.fg(token_color(token))));
    };

    while let Some(&(start, ch)) = chars.peek() {
        // Line comments: `//` (Rust/JS) or `#` (Python/shell).
        if ch == '#' || (ch == '/' && line[start..].starts_with("//")) {
            let rest: String = line[start..].to_string();
            push(&mut spans, rest, Token::Comment);
            break;
        }

        // String literals (single or double quoted).
        if ch == '"' || ch == '\'' {
            let quote = ch;
            let mut text = String::new();
            text.push(ch);
            chars.next();
            while let Some(&(_, c)) = chars.peek() {
                text.push(c);
                chars.next();
                if c == quote {
                    break;
                }
            }
            push(&mut spans, text, Token::Str);
            continue;
        }

        // Numbers.
        if ch.is_ascii_digit() {
            let mut text = String::new();
            while let Some(&(_, c)) = chars.peek() {
                if c.is_ascii_alphanumeric() || c == '.' || c == '_' {
                    text.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            push(&mut spans, text, Token::Number);
            continue;
        }

        // Identifiers / keywords.
        if ch.is_alphabetic() || ch == '_' {
            let mut text = String::new();
            while let Some(&(_, c)) = chars.peek() {
                if c.is_alphanumeric() || c == '_' {
                    text.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            let token = if KEYWORDS.contains(&text.as_str()) {
                Token::Keyword
            } else if is_type_name(&text) {
                Token::Type
            } else {
                Token::Ident
            };
            push(&mut spans, text, token);
            continue;
        }

        // Everything else (whitespace, punctuation) folds into one punct span
        // until the next "interesting" character, so spaces keep the base style.
        // `/` ends the run so the comment branch above can re-evaluate `//`.
        let mut text = String::new();
        while let Some(&(_, c)) = chars.peek() {
            if c.is_alphanumeric() || matches!(c, '_' | '"' | '\'' | '#' | '/') {
                break;
            }
            text.push(c);
            chars.next();
        }
        if text.is_empty() {
            // Lone `/` that isn't a comment: emit it as punctuation and advance.
            if let Some((_, c)) = chars.next() {
                push(&mut spans, c.to_string(), Token::Punct);
            }
            continue;
        }
        if text.chars().all(char::is_whitespace) {
            spans.push(Span::styled(text, base));
        } else {
            push(&mut spans, text, Token::Punct);
        }
    }

    if spans.is_empty() {
        spans.push(Span::styled(String::new(), base));
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(line: &str) -> String {
        highlight_line(line, Style::default())
            .iter()
            .map(|s| s.content.as_ref())
            .collect()
    }

    #[test]
    fn round_trips_content_exactly() {
        for line in [
            "fn is_expired(&self, now: SystemTime) -> bool {",
            "let claims = self.decode_claims()?; // ok",
            "    return Err(AuthError::Expired);",
            "x / y / z",
            "",
        ] {
            assert_eq!(text(line), line, "content must be preserved verbatim");
        }
    }

    #[test]
    fn keyword_gets_keyword_color() {
        let spans = highlight_line("fn main", Style::default());
        let kw = spans.iter().find(|s| s.content == "fn").expect("fn span");
        assert_eq!(kw.style.fg, Some(token_color(Token::Keyword)));
    }

    #[test]
    fn comment_swallows_rest_of_line() {
        let spans = highlight_line("a // b c", Style::default());
        let comment = spans.last().unwrap();
        assert_eq!(comment.content, "// b c");
        assert_eq!(comment.style.fg, Some(token_color(Token::Comment)));
    }

    #[test]
    fn never_panics_on_partial_input() {
        // Diff lines are revealed character-by-character, so partial tokens
        // (unterminated strings, lone slashes) must be handled gracefully.
        for partial in ["let s = \"unterm", "value /", "fn ", "0x"] {
            let _ = highlight_line(partial, Style::default());
        }
    }
}
