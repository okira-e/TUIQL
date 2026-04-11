pub struct HelpViewModel {
    /// Index into the selectable help items.
    pub cursor: usize,
}

impl Default for HelpViewModel {
    fn default() -> Self {
        Self { cursor: 0 }
    }
}
