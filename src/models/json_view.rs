use serde_json::Value;

#[derive(Default)]
pub struct JsonViewModel {
    pub data: Option<Value>,
    pub scroll_y: u16,
}
