

pub fn path_split<'a>(arg: &'a str) -> Vec<&'a str> {
    arg.split('/')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn path_relocate<'a>(arg: &'a Vec<String>) -> Vec<&'a str> {
    arg.iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}
