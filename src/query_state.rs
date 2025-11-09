#[derive(Debug, Default, Clone)]
pub struct Query {
    pub limit: usize,
    pub offset: usize,
    pub where_clause: String,
    pub group_by: String,
    pub order_by: String,
}

impl Query {
    pub fn new() -> Self {
        return Self {
            limit: 100,
            offset: 0,
            where_clause: String::new(),
            group_by: String::new(),
            order_by: String::new(),
        };
    }
}
