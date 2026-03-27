use std::collections::HashMap;

use chrono::{NaiveDateTime,NaiveDate,Datelike,Timelike};
use chrono::format::strftime::{StrftimeItems};
use minijinja::{
    Error, ErrorKind, Value,
};

/// Converts a string timestamp into a table of values
///
/// strftime syntax documentation at -> https://docs.rs/chrono/latest/chrono/format/strftime/index.html
pub fn parse_datetime(input: String, parse: String) -> Result<Value,Error> {
    let _ = StrftimeItems::new_lenient(parse.as_ref())
        .parse()
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, format!("input: '{}' is not a valid strftime string, error: '{}'", parse.as_str(), e)))?;
    let dt = NaiveDateTime::parse_from_str(input.as_str(),parse.as_str())
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, format!("input: '{}' could not be parsed by '{}', error: '{}'", input.as_str(), parse.as_str(), e)))?;
    let mut map = HashMap::new();
    map.insert("year".to_string(), Value::from(dt.year()));
    map.insert("month".to_string(), Value::from(dt.month()));
    map.insert("day".to_string(), Value::from(dt.day()));
    map.insert("hour".to_string(), Value::from(dt.hour()));
    map.insert("minute".to_string(), Value::from(dt.minute()));
    map.insert("second".to_string(), Value::from(dt.second()));
    map.insert("nanos".to_string(), Value::from(dt.nanosecond()));
    Ok(Value::from(map))
}

/// Reformats a timestamp
///
/// strftime syntax documentation at -> https://docs.rs/chrono/latest/chrono/format/strftime/index.html
pub fn reformat_datetime(input: String, parse: String, format: String) -> Result<Value,Error> {

    /*
     * Sanity Checks
     *
     */
    let _ = StrftimeItems::new_lenient(parse.as_str())
        .parse()
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, format!("parse: '{}' is not a valid strftime string, error: '{}'", parse.as_str(), e)))?;
    let _ = StrftimeItems::new_lenient(format.as_str())
        .parse()
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, format!("format: '{}' is not a valid strftime string, error: '{}'", format.as_str(), e)))?;

    /*
     * Operation
     *
     */
    let dt = NaiveDateTime::parse_from_str(input.as_str(),parse.as_str())
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, format!("input: '{}' could not be parsed by '{}', error: '{}'", input.as_str(), parse.as_str(), e)))?;
    let mut s = String::with_capacity(format.len()*2);
    dt.format(format.as_str())
        .write_to(&mut s)
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, format!("format error occured writing output, '{:?}'", e)))?;
    Ok(Value::from(s))
}

/// Reformats a date
///
/// strftime syntax documentation at -> https://docs.rs/chrono/latest/chrono/format/strftime/index.html
pub fn reformat_date(input: String, parse: String, format: String) -> Result<Value,Error> {

    /*
     * Operation
     *
     */
    let date = NaiveDate::parse_from_str(input.as_str(),parse.as_str())
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, format!("input: '{}' could not be parsed by '{}', error: '{}'", input.as_str(), parse.as_str(), e)))?;
    let mut s = String::with_capacity(format.len()*2);
    date.format(format.as_str())
        .write_to(&mut s)
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, format!("format error occured writing output, '{:?}'", e)))?;
    Ok(Value::from(s))
}

#[test]
fn test_reformat_datetime() {
    use minijinja::{context,Environment};
    const DUT: &'static str = r#"{{ arg | reformat_date('%y%m%d','%Y-%m')}}"#;

    let mut env = Environment::new();
    env.add_filter("reformat_date",reformat_date);

    let out = env.template_from_str(DUT)
        .unwrap()
        .render(context!(arg => "250911"))
        .unwrap();
    assert_eq!("2025-09", out.as_str());
}
