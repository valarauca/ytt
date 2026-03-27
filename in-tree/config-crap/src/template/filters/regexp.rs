use regex::Regex;
use minijinja::{
    Error,ErrorKind,Value,
};

pub fn regex_match(input: String, arg: String) -> Result<Value,Error> {
    let r = Regex::new(arg.as_str())
        .map_err(|e| Error::new(ErrorKind::NonPrimitive, format!("could not format input: '{}' as regex: '{:?}'", arg.as_str(), e)))?;
    Ok(Value::from(r.is_match(input.as_str())))
}
