use std::collections::HashMap;

use url::Url;
use minijinja::{
    value::{Value,ValueKind},
    Error, ErrorKind,
};

pub fn url(input: Value) -> Result<Value,Error> {
    let url_str_opt = match input.kind() {
        ValueKind::String => input.as_str(),
        _ => None
    };
    let url_str = url_str_opt
        .ok_or_else(||Error::new(ErrorKind::NonPrimitive, "url filter expects a string input"))?;
    let url = Url::parse(url_str)
        .map_err(|e| Error::new(ErrorKind::NonPrimitive, format!("input: '{}' could not parsed as url, error: '{:?}'", url_str, e)))?;

    let mut info = HashMap::<String,Value>::new();

    insert(&mut info, "authority", || Some(url.authority()));
    insert(&mut info, "host", || url.host_str());
    insert(&mut info, "domain", || url.domain());
    insert(&mut info, "password", || url.password());
    insert(&mut info, "username", || Some(url.username()));
    insert(&mut info, "scheme", || Some(url.scheme()));
    insert(&mut info, "port", || url.port());
    insert(&mut info, "path", || Some(url.path()));
    insert(&mut info, "path_parts", || url.path_segments().map(|x| x.into_iter().map(|x| String::from(x)).collect::<Vec<String>>()));
    insert(&mut info, "fragment", || url.fragment());
    insert(&mut info, "query", || url.query());
    insert(&mut info, "query_pairs", || Some(url.query_pairs().into_owned().map(|(k,v)| (k,Value::from(v))).collect::<HashMap<String,Value>>()));

    Ok(Value::from(info))
}
fn insert<F,V>(map: &mut HashMap<String,Value>, name: &'static str, value: F)
where
    F: FnOnce() -> Option<V>,
    Value: From<V>,
{
    if let Some(v) = (value)().map(Value::from) {
        map.insert(name.to_string(), v);
    }
}


#[test]
fn test_url_filter() {
    use minijinja::{context,Environment};
    let mut env = Environment::new();
    env.add_filter("url",url);
    let out = env.template_from_str("{{ (arg | url).host }}")
        .unwrap()
        .render(context!(arg => "https://c10.patreonusercontent.com/4/patreon-media/p/post/138623176/d1c5cffe4e66445db23aef91a7f70588/eyJhIjoxLCJwIjoxfQ%3D%3D/1.zip?token-hash=N1hzj4BpkAEaWRHSoXJdS3_RvqXtDTFMPvMCiHB5G_M%3D&token-time=1757894400"))
        .unwrap();
    assert_eq!("c10.patreonusercontent.com", out.as_str());
}
