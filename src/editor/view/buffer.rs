#[derive(Debug, Clone)]
pub struct Buffer {
    pub lines: Vec<String>,
}

impl Default for Buffer {
    fn default() -> Self {
        return Self {
            lines: vec!["Hellow world!".to_string(), "gooybye".to_string()],
        };
    }
}
