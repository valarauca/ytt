use std::collections::HashMap;
use std::sync::OnceLock;
use std::borrow::Cow;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{DeserializeOwned};
use serde::de::value::StrDeserializer;
use regex::{Regex};

/// Used to template values with environment values
///
/// For example if you service has something like `foo=bar` in the environment.
/// Then `WithEnv<Url>` value who's string is `http://${foo}.com` will be read
/// into the program as `http://bar.com`.
///
/// This is wildly useful for deployment scenarios.
pub struct WithEnv<T> {
    value: T,
    from_config: String,
}
impl<T> WithEnv<T> {
    pub fn into_inner(self) -> T {
        self.value
    }
}
impl<T: std::fmt::Debug> std::fmt::Debug for WithEnv<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WithEnv")
            .field(&self.from_config, &self.value)
            .finish()
    }
}
impl<T: Clone> Clone for WithEnv<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            from_config: self.from_config.clone(),
        }
    }
}
impl<T: PartialEq<T>> PartialEq for WithEnv<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}
impl<T> std::ops::Deref for WithEnv<T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a Self::Target { &self.value }
}
impl<T> AsRef<T> for WithEnv<T> {
    fn as_ref<'a>(&'a self) -> &'a T { &self.value }
}
impl<T> Serialize for WithEnv<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.from_config)
    }
}

impl<'de,T: DeserializeOwned> Deserialize<'de> for WithEnv<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let cow: Cow<'de,str> = <Cow<'de,str> as Deserialize<'de>>::deserialize::<D>(deserializer)?;
        let from_config: String = cow.as_ref().to_string();
        let cow: Cow<'de,str> = template_string(cow);
        let value: T = <T as Deserialize>::deserialize(StrDeserializer::new(cow.as_ref()))?;
        Ok(Self {
            value,
            from_config,
        })
    }
}


static VAR: OnceLock<Regex> = OnceLock::new();
fn get_var() -> &'static Regex {
    VAR.get_or_init(|| {
        Regex::new(r#"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}"#).unwrap()
    })
}
static ENV: OnceLock<HashMap<String,String>> = OnceLock::new();
fn get_env() -> &'static HashMap<String,String> {
    ENV.get_or_init(|| {
        std::env::vars().collect()
    })
}

fn template_string<'a,S>(string: S) -> Cow<'a,str>
where
    Cow<'a,str>: From<S>,
{
    let string: Cow<'a,str> = string.into();
    let regexp = get_var();
    if !regexp.is_match(string.as_ref()) {
        return string;
    }
    let env = get_env();
    let input_ref = string.as_ref();
    let mut buffer = String::with_capacity(input_ref.len());
    let mut splits = regexp.split(input_ref);
    let mut caps = regexp.captures_iter(input_ref);
    to_string(&mut splits, &mut caps, &mut buffer, env);
    Cow::Owned(buffer)
}

fn to_string<'r,'h>(
    splits: &mut regex::Split<'r,'h>,
    caps: &mut regex::CaptureMatches<'r,'h>,
    buffer: &mut String,
    env: &HashMap<String,String>,
) {
    let mut split: Option<&'h str> = splits.next();
    let mut cap: Option<regex::Captures<'h>> = caps.next();

    enum LoopAction<'a> {
        PushStr(&'a str),
        LookupVar(regex::Captures<'a>),
    }

    loop {
        let action = match (split.take(),cap.take()) {
            (None,None) => return,
            (Some(s),None) => LoopAction::PushStr(s),
            (None,Some(c)) => LoopAction::LookupVar(c),
            (Some(s),Some(c)) => {
                let split_start_addr = s.as_ptr().addr();
                let cap_start_addr = c.get(0).unwrap().as_str().as_ptr().addr();
                if split_start_addr < cap_start_addr {
                    cap = Some(c);
                    LoopAction::PushStr(s)
                } else {
                    split = Some(s);
                    LoopAction::LookupVar(c)
                }
            }
        };
        match action {
           LoopAction::PushStr(x) => {
               buffer.push_str(x);
               split = splits.next();
           },
           LoopAction::LookupVar(x) => {
               if let Some(value) = x.get(1).map(|m| env.get(m.as_str()).map(|value| value.as_str())).flatten() {
                   buffer.push_str(value);
               }
               cap = caps.next();
           }
        };
    }
}


#[test]
fn test_template_string() {
    let _ = ENV.set( vec![
        ("foo","bar")
        ].into_iter().map(|(k,v)| (k.to_string(),v.to_string())).collect::<HashMap<String,String>>()
    );

    assert_eq!(template_string("${foo}"), "bar");
    assert_eq!(template_string("${foo}baz"), "barbaz");
    assert_eq!(template_string("baz${foo}"), "bazbar");

    assert_eq!(template_string(" ${foo} ${foo}  ${foo}  "), " bar bar  bar  ");
}
