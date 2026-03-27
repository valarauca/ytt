
use minijinja::Environment;
use serde::{Serialize,Deserialize};

use crate::template::expression::Expression;

/// When handles 
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct When<T> {
    when: Option<Expression>,
    #[serde(flatten)]
    data: T,
}
impl<T> When<T> {
    /// runs the expression if a truthy value is returned, the interior value is returned.
    ///
    /// if the expression returns false, then `Option::None` is returned.
    pub fn get_interior<'b,S>(&'b self, env: &Environment, ctx: &S) -> Result<Option<&'b T>,minijinja::Error>
    where
        S: Serialize,
    {
        match &self.when {
            Some(expr) => {
                let resolve = expr.resolve(env, ctx)?;
                if resolve.is_true() {
                    Ok(Some(&self.data))
                } else {
                    Ok(None)
                }
            }
            None => {
                Ok(Some(&self.data))
            }
        }
    }
}

