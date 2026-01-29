use reqwest::{
    header::{HeaderName, HeaderValue, ACCEPT, AUTHORIZATION},
    Method,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenRouterBaseConfig {
    #[serde(
        deserialize_with = "deserialize_url",
        serialize_with = "serialize_url"
    )]
    pub site_url: Url,
    pub api_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_referer: Option<String>,
}

impl OpenRouterBaseConfig {
    pub fn new<S>(api_key: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            site_url: Url::parse("https://openrouter.ai/").expect("default URL is valid"),
            api_key: api_key.into(),
            site_title: None,
            site_referer: None,
        }
    }

    pub fn with_site_url(mut self, site_url: Url) -> Self {
        self.site_url = site_url;
        self
    }

    pub fn with_site_title<S>(mut self, site_title: S) -> Self
    where
        S: Into<String>,
    {
        self.site_title = Some(site_title.into());
        self
    }

    pub fn with_site_referer<S>(mut self, site_referer: S) -> Self
    where
        S: Into<String>,
    {
        self.site_referer = Some(site_referer.into());
        self
    }

    pub fn build_request(&self, method: Method, path: &str) -> Result<reqwest::Request, url::ParseError> {
        let url = self.site_url.join(path)?;
        let mut req = reqwest::Request::new(method, url);

        let headers = req.headers_mut();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        if let Ok(auth_value) = HeaderValue::from_str(&format!("Bearer {}", &self.api_key)) {
            headers.insert(AUTHORIZATION, auth_value);
        }

        if let Some(ref title) = self.site_title {
            if let Ok(value) = HeaderValue::from_str(title) {
                headers.insert(HeaderName::from_static("x-title"), value);
            }
        }

        if let Some(ref referer) = self.site_referer {
            if let Ok(value) = HeaderValue::from_str(referer) {
                headers.insert(HeaderName::from_static("http-referer"), value);
            }
        }

        Ok(req)
    }

    pub fn get(&self, path: &str) -> Result<reqwest::Request, url::ParseError> {
        self.build_request(Method::GET, path)
    }

    pub fn post(&self, path: &str) -> Result<reqwest::Request, url::ParseError> {
        self.build_request(Method::POST, path)
    }
}

fn deserialize_url<'de, D>(d: D) -> Result<Url, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    <Url as serde::de::Deserialize<'de>>::deserialize(d)
}

fn serialize_url<S>(url: &Url, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
{
    s.serialize_str(url.as_str())
}




