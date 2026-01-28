
use reqwest::ClientBuilder;


/// Abstracts away from of the boilerplate of doing configuration
pub trait Apply: Sized {
    fn apply(arg: &Option<Self>, builder: ClientBuilder) -> ClientBuilder {
        match arg {
            &Option::None => builder,
            Option::Some(arg) => arg.apply_opts(builder),
        }
    }

    fn apply_opts(&self, builder: ClientBuilder) -> ClientBuilder;
}
