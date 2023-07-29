use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use crate::utils::PathExt;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ModulePath {
    segments: Vec<String>,
}

impl ModulePath {
    pub fn new() -> Self {
        Self { segments: vec![] }
    }

    pub fn join<S: AsRef<str>>(&self, segment: S) -> Self {
        let mut new = self.clone();
        new.push(segment.as_ref().to_string());
        new
    }

    pub fn push(&mut self, segment: String) {
        self.segments.push(segment)
    }

    pub fn pop(&mut self) {
        self.segments.pop();
    }

    pub fn parent(&self) -> ModulePath {
        let mut parent = self.clone();
        parent.pop();
        parent
    }

    pub fn element(&self) -> Option<&str> {
        self.segments.last().map(|last| last.as_str())
    }

    pub fn as_path<P: AsRef<Path>>(&self, base_path: P, entry_file: &str) -> PathBuf {
        let relative = self.segments.join("/");
        let mut full_path = base_path.as_ref().join(relative).join_if_exists("mod.rs");

        if !full_path.ends_with("mod.rs") {
            match self.segments.last() {
                Some(last_seg) => {
                    let last_seg = last_seg.replace("r#", "");

                    full_path.pop();
                    full_path.push(format!("{last_seg}.rs"))
                }
                None => full_path.push(entry_file),
            }
        }

        full_path
    }
}

impl From<&str> for ModulePath {
    fn from(value: &str) -> Self {
        let segments = value.split("::").map(|s| s.to_string()).collect();
        let mut path = ModulePath::new();
        path.segments = segments;
        path
    }
}

impl Display for ModulePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.segments.join("::"))
    }
}
