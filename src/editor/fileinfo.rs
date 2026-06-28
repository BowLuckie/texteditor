use std::{
    fmt::{self, Display},
    path::PathBuf,
};

#[derive(Debug, Default, Clone)]
pub struct FileInfo {
    pub path: Option<PathBuf>,
}

impl FileInfo {
    pub fn from(file_name: &str) -> Self {
        return Self {
            path: Some(PathBuf::from(file_name)),
        };
    }
}

impl Display for FileInfo {
    #![allow(clippy::implicit_return)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self
            .path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("[No Name]");
        return f.write_str(name);
    }
}
