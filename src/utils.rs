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