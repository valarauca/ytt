

pub fn path_split<'a>(arg: &'a str) -> Vec<&'a str> {
    arg.split('/')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}
