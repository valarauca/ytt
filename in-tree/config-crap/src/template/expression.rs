
use minijinja::{Environment,Value};
use serde::de::{Visitor, Deserializer, Deserialize, Error as DError};
use serde::ser::{Serialize, Serializer};

use crate::template::globals::{is_valid_expression};

/// Expression permits serializing/deserializating minijinja templates
#[derive(Clone,Debug)]
pub struct Expression {
    data: String,
}
impl AsRef<str> for Expression {
    /// returns the template unexpanded
    fn as_ref<'b>(&'b self) -> &'b str {
        self.data.as_ref()
    }
}
impl Expression {
    /// Creates a temporary value
    pub fn new(data: String) -> Result<Self, minijinja::Error> {
        is_valid_expression(&data)?;
        Ok(Self { data })
    }

    /// Serializes input data to a value
    pub fn resolve<S>(&self, env: &Environment<'_>, ctx: S) -> Result<Value,minijinja::Error>
    where
        S: Serialize,
    {
        validate_environment(env, self.data.as_ref(), &ctx)?;
        let t = env.compile_expression(self.data.as_ref())?;
        t.eval(ctx)
    }
}


fn validate_environment<S>(env: &Environment<'_>, arg: &str, ctx: &S) -> Result<minijinja::Value,minijinja::Error>
where
    S: Serialize,
{
    let expr = env.compile_expression(arg)?;
    let undeclared = expr.undeclared_variables(true);
    if !undeclared.is_empty() {
        let mut arg = String::new();
        for expr_str in undeclared {
            arg.clear();
            arg.push_str(expr_str.as_ref());
            arg.push_str(" is defined");
            let expr = env.compile_expression(arg.as_str())?;
            let value = expr.eval(ctx)?;
            debug_assert!(value.kind() == minijinja::value::ValueKind::Bool);
            arg.clear();
            if !value.is_true() {
                return Err(minijinja::Error::new(minijinja::ErrorKind::MissingArgument, format!("variable: '{}' is not defined", expr_str.as_str())));
            }
        }
    }
    expr.eval(ctx)
}

struct ExpressionVisitor;

impl<'de> Visitor<'de> for ExpressionVisitor {
    type Value = Expression;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a valid template string")
    }

    fn visit_string<E: DError>(self, data: String) -> Result<Self::Value, E> {
        is_valid_expression(data.as_str())
            .map_err(|err| E::custom(err))?;
        Ok(Expression { data })
    }
}

impl<'de> Deserialize<'de> for Expression {
    fn deserialize<D>(deserializer: D) -> Result<Expression, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ExpressionVisitor)
    }
}
impl Serialize for Expression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.data.as_ref())
    }
}
