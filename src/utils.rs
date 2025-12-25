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

pub fn centered_rect_pct(w_pct: u16, h_pct: u16, area: Rect) -> Rect {
    let w = area.width.saturating_mul(w_pct).saturating_div(100);
    let h = area.height.saturating_mul(h_pct).saturating_div(100);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}