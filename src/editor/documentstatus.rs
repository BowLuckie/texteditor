#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct DocumentStatus {
    pub file_name: String,
    pub line_count: usize,
    pub current_line: usize,
    pub is_modified: bool,
}

impl DocumentStatus {
    pub fn modified_indicator_to_string(&self) -> String {
        return if self.is_modified {
            String::from("(modified)")
        } else {
            String::new()
        };
    }

    pub fn line_count_to_string(&self) -> String {
        return format!("{} lines", self.line_count);
    }

    pub fn position_indicator_to_string(&self) -> String {
        return format!(
            "{}/{}",
            self.current_line.saturating_add(1),
            self.line_count
        );
    }
}
