pub fn strikethrough(s: &str) -> String {
    s.chars().map(|c| format!("{}\u{0336}", c)).collect()
}
