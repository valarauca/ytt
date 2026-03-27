use minijinja::Environment;
use serde::de::{Visitor, Deserializer, Deserialize, Error as DError};
use serde::ser::{Serialize, Serializer};

use crate::template::globals::{is_valid_template};

/// Template permits serializing/deserializating minijinja templates
#[derive(Clone,Debug)]
pub struct Template {
    data: String,
}
impl AsRef<str> for Template {
    /// returns the template unexpanded
    fn as_ref<'b>(&'b self) -> &'b str {
        self.data.as_ref()
    }
}
impl Template {
    /// Creates a temporary value
    pub fn new(data: String) -> Result<Self, minijinja::Error> {
        is_valid_template(&data)?;
        Ok(Self { data })
    }

    /// Serializes input data to a value
    pub fn render<S>(&self, env: &Environment<'_>, ctx: S) -> Result<String,minijinja::Error>
    where
        S: Serialize,
    {
        validate_environment(env, self.data.as_ref(), &ctx)?;
        let t = env.template_from_str(self.data.as_ref())?;
        t.render(ctx)
    }
}

fn validate_environment<S>(env: &Environment<'_>, template: &str, ctx: &S) -> Result<(),minijinja::Error>
where
    S: Serialize,
{
    let template = env.template_from_str(template)?;
    let undefined = template.undeclared_variables(true);
    if undefined.is_empty() {
        return Ok(())
    }
    let mut arg = String::new();
    for undefined_str in undefined.iter() {
        arg.clear();
        arg.push_str(undefined_str.as_ref());
        arg.push_str(" is defined");
        let expr = env.compile_expression(arg.as_str())?;
        let value = expr.eval(ctx)?;
        debug_assert!(value.kind() == minijinja::value::ValueKind::Bool);
        arg.clear();
        if !value.is_true() {
            return Err(minijinja::Error::new(minijinja::ErrorKind::MissingArgument, format!("variable: '{}' is not defined", undefined_str.as_str())));
        }
    }
    Ok(())
}

struct TemplateVisitor;

impl<'de> Visitor<'de> for TemplateVisitor {
    type Value = Template;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a valid template string")
    }

    fn visit_str<E: DError>(self, data: &str) -> Result<Self::Value,E> {
        is_valid_template(data)
            .map_err(|err| E::custom(err))?;
        Ok(Template { data: data.to_string() })
    }

    fn visit_string<E: DError>(self, data: String) -> Result<Self::Value, E> {
        is_valid_template(&data)
            .map_err(|err| E::custom(err))?;
        Ok(Template { data })
    }
}

impl<'de> Deserialize<'de> for Template {
    fn deserialize<D>(deserializer: D) -> Result<Template, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(TemplateVisitor)
    }
}
impl Serialize for Template {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.data.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;
    use serde_json;
    use minijinja::context;

    #[test]
    fn test_template_serialize() {
        let template = Template::new("Hello {{ name }}!".to_string()).unwrap();
        let json = serde_json::to_string(&template).unwrap();
        assert_eq!(json, r#""Hello {{ name }}!""#);
    }

    #[test]
    fn test_invalid_template() {
        let json = r#""Hello {{ invalid_syntax""#;
        let result: Result<Template, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_serialization() {
        let template = Template::new("Hello {{ name }}!".to_string()).unwrap();
        let json = serde_json::to_string(&template).unwrap();
        let t: Template = serde_json::from_str(&json).unwrap();

        let env = Environment::new();
        let out = t.render(&env, context!{ name => "Foo" }).unwrap();
        assert_eq!(out.as_str(), "Hello Foo!");
    }
}
