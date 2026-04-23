use crate::config::Colors;
use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub surface: Color,
    pub border: Color,
    pub border_focused: Color,
    pub text: Color,
    pub text_dim: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub active: Color,
    pub inactive: Color,
    pub failed: Color,
    pub filter_bar: Color,
    pub header: Color,
}

impl Theme {
    pub fn from_colors(colors: &Colors) -> Self {
        Self {
            background: resolve(&colors.background, Color::Reset),
            surface: resolve(&colors.surface, Color::Black),
            border: resolve(&colors.border, Color::Blue),
            border_focused: resolve(&colors.border_focused, Color::Green),
            text: resolve(&colors.text, Color::White),
            text_dim: resolve(&colors.text_dim, Color::Gray),
            selection_bg: resolve(&colors.selection_bg, Color::DarkGray),
            selection_fg: resolve(&colors.selection_fg, Color::White),
            active: resolve(&colors.active, Color::Green),
            inactive: resolve(&colors.inactive, Color::Gray),
            failed: resolve(&colors.failed, Color::Red),
            filter_bar: resolve(&colors.filter_bar, Color::Yellow),
            header: resolve(&colors.header, Color::Blue),
        }
    }
}

fn resolve(hex: &Option<String>, fallback: Color) -> Color {
    hex.as_deref()
        .and_then(parse_hex)
        .map(|(r, g, b)| Color::Rgb(r, g, b))
        .unwrap_or(fallback)
}

fn parse_hex(s: &str) -> Option<(u8, u8, u8)> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some((r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_hex() {
        assert_eq!(parse_hex("#00ff88"), Some((0, 255, 136)));
        assert_eq!(parse_hex("8844ff"), Some((136, 68, 255)));
    }

    #[test]
    fn rejects_invalid_hex() {
        assert_eq!(parse_hex("#xyz"), None);
        assert_eq!(parse_hex("#0000"), None);
    }

    #[test]
    fn fallback_when_none() {
        let colors = Colors::default();
        let theme = Theme::from_colors(&colors);
        assert_eq!(theme.active, Color::Green);
    }
}
