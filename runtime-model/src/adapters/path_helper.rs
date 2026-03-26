
use std::path::{Path,PathBuf};

pub trait GetTreePath {
    fn get_tree_path<'a>(&'a self) -> Result<Vec<&'a str>,anyhow::Error>;
}
impl GetTreePath for str {
    fn get_tree_path<'a>(&'a self) -> Result<Vec<&'a str>,anyhow::Error> {
        split_and_validate_path(self)
    }
}
impl<'b> GetTreePath for &'b str {
    fn get_tree_path<'a>(&'a self) -> Result<Vec<&'a str>,anyhow::Error> {
        split_and_validate_path(self)
    }
}
impl<'b> GetTreePath for &'b &str {
    fn get_tree_path<'a>(&'a self) -> Result<Vec<&'a str>,anyhow::Error> {
        split_and_validate_path(self)
    }
}
impl GetTreePath for String {
    fn get_tree_path<'a>(&'a self) -> Result<Vec<&'a str>,anyhow::Error> {
        split_and_validate_path(self.as_str())
    }
}
impl<'b> GetTreePath for &'b String {
    fn get_tree_path<'a>(&'a self) -> Result<Vec<&'a str>,anyhow::Error> {
        split_and_validate_path(self.as_str())
    }
}
impl<'b> GetTreePath for &'b &String {
    fn get_tree_path<'a>(&'a self) -> Result<Vec<&'a str>,anyhow::Error> {
        split_and_validate_path(self.as_str())
    }
}

fn split_and_validate_path<'a>(arg: &'a str) -> Result<Vec<&'a str>,anyhow::Error> {
    if arg.is_empty() {
        anyhow::bail!("path: '{}' is illegal, must not be an empty string", arg)
    }
    if !arg.starts_with('/') {
        anyhow::bail!("path: '{}' is illegal, must start with '/'", arg)
    }
    if !arg.ends_with('/') {
        anyhow::bail!("path: '{}' is illegal, cannot end with '/'", arg)
    }
    arg.split('/')
        .map(|segment| -> Result<&'a str, anyhow::Error> {
            if segment.is_empty() {
                anyhow::bail!("path: '{}' is illegal, segments may not be empty", arg);
            }
            if segment == "." || segment == ".." {
                anyhow::bail!("path: '{}' is illegal, segements may not contain '.' or '..'", arg);
            }
            Ok(segment)
        })
        .collect()
}
