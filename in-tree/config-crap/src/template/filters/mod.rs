
mod url_filter;
use self::url_filter::{url};
mod regexp;
use self::regexp::{regex_match};
mod parse_timestamp;
use self::parse_timestamp::{parse_datetime,reformat_datetime,reformat_date};

/// Adds extended filters
///
/// * `url` performs URL parsing, returns an object with fields containing data about the URL
/// * `regex_match` returns a boolean if the input matches the argument
pub fn extended_filters(env: &mut minijinja::Environment) {
    
    env.add_filter("url", url);
    env.add_filter("regex_match", regex_match);
    env.add_filter("parse_datetime", parse_datetime);
    env.add_filter("reformat_datetime", reformat_datetime);
    env.add_filter("reformat_date", reformat_date);
}
