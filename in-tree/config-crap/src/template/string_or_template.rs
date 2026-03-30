use serde::de::{Visitor, Deserializer, Deserialize, Error as DError};

use crate::template::template::{Template};

/// Converts the value into a template or string.
///
/// Will attempt to parse the input as a minijinja::Template
/// provided there is at least 1 pair of `{% %}` or `{{ }}`.
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub enum StringOrTemplate {
    String(String),
    Template(Template),
}
impl AsRef<str> for StringOrTemplate {
    /// returns the template unexpanded
    fn as_ref<'b>(&'b self) -> &'b str {
        match self {
            &Self::String(ref c) => c.as_ref(),
            &Self::Template(ref t) => t.as_ref(),
        }
    }
}
impl StringOrTemplate {

    /// Attempt to construct a new value
    pub fn new(data: String) -> Result<Self,minijinja::Error> {
        fn is_template(arg: &str) -> bool {
            (arg.match_indices("{%").count() > 1 && arg.match_indices("%}").count() > 1)
                ||
            (arg.match_indices("{{").count() > 1 && arg.match_indices("}}").count() > 1)
        }
        if is_template(data.as_str()) {
            Ok(Self::Template(Template::new(data)?))
        } else {
            Ok(Self::String(data)) 
        }
    }

    /// Is this value a string
    pub fn is_string(&self) -> bool {
        match self {
            &Self::String(_) => true,
            _ => false
        }
    }

    /// Is this value a template
    pub fn is_template(&self) -> bool {
        match self {
            &Self::Template(_) => true,
            _ => false
        }
    }
}


struct StringOrTemplateVisitor;

impl<'de> Visitor<'de> for StringOrTemplateVisitor {
    type Value = StringOrTemplate;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a valid template string or just a string")
    }

    fn visit_string<E: DError>(self, data: String) -> Result<Self::Value, E> {
        Ok(StringOrTemplate::new(data)
            .map_err(|err| E::custom(err))?)
    }
}

impl<'de> Deserialize<'de> for StringOrTemplate {
    fn deserialize<D>(deserializer: D) -> Result<StringOrTemplate, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(StringOrTemplateVisitor)
    }
}
