use std::sync::OnceLock;

use minijinja::{Environment,Error};

static DESERIALIZE_ENV: OnceLock<Environment<'static>> = OnceLock::new();
pub (crate) fn is_valid_expression(arg: &str) -> Result<(),Error> {
    let e: &'static Environment<'static> = DESERIALIZE_ENV.get_or_init(|| Environment::new());
    let _ = e.compile_expression(arg)?;
    Ok(())
}

pub(crate) fn is_valid_template(arg: &str) -> Result<(), minijinja::Error> {
    let e: &'static Environment<'static> = DESERIALIZE_ENV.get_or_init(|| Environment::new());
    let _ = e.template_from_str(arg)?;
    Ok(())
}

