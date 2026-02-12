use ratatui::layout::Rect;

pub fn quote_ident(name: &str) -> String {
    // Double-quote the identifier and escape embedded quotes
    let mut s = String::with_capacity(name.len() + 2);
    s.push('"');
    for ch in name.chars() {
        if ch == '"' {
            s.push('"');
        }
        s.push(ch);
    }
    s.push('"');

    return s;
}

#[allow(dead_code)]
pub fn centered_rect_pct(w_pct: u16, h_pct: u16, area: Rect) -> Rect {
    let w = area.width.saturating_mul(w_pct).saturating_div(100);
    let h = area.height.saturating_mul(h_pct).saturating_div(100);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;

    return Rect { x, y, width: w, height: h };
}

#[cfg(test)]
mod tests {
    use super::*;

    // Security-critical: SQL injection prevention
    #[test]
    fn test_quote_ident_basic() {
        assert_eq!(quote_ident("users"), r#""users""#);
        assert_eq!(quote_ident("my_table"), r#""my_table""#);
    }

    #[test]
    fn test_quote_ident_with_quotes() {
        // Critical: embedded quotes must be escaped by doubling them
        assert_eq!(quote_ident(r#"evil"table"#), r#""evil""table""#);
        assert_eq!(quote_ident(r#"test"name"here"#), r#""test""name""here""#);
    }

    #[test]
    fn test_quote_ident_special_characters() {
        assert_eq!(quote_ident("table-name"), r#""table-name""#);
        assert_eq!(quote_ident("table name"), r#""table name""#);
        assert_eq!(quote_ident("table.name"), r#""table.name""#);
    }

    #[test]
    fn test_centered_rect_pct() {
        let area = Rect { x: 0, y: 0, width: 100, height: 100 };

        // 50% width, 50% height should be centered
        let result = centered_rect_pct(50, 50, area);
        assert_eq!(result.width, 50);
        assert_eq!(result.height, 50);
        assert_eq!(result.x, 25); // (100-50)/2
        assert_eq!(result.y, 25);
    }

    #[test]
    fn test_centered_rect_pct_with_offset() {
        let area = Rect { x: 20, y: 30, width: 200, height: 100 };

        let result = centered_rect_pct(50, 50, area);
        assert_eq!(result.width, 100); // 50% of 200
        assert_eq!(result.height, 50); // 50% of 100
        assert_eq!(result.x, 70); // 20 + (200-100)/2
        assert_eq!(result.y, 55); // 30 + (100-50)/2
    }
}
