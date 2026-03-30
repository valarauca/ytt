
use reqwest::header::{HeaderValue,HeaderName};
use serde::de::{self};
use serde::ser::{self};


/*
 * Header Name Boilerplate
 *
 */
#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub struct HHeaderName(
    pub HeaderName
);
impl PartialOrd for HHeaderName {
    fn partial_cmp(&self, arg: &Self) -> Option<std::cmp::Ordering> {
        self.0.as_str().partial_cmp(arg.0.as_str())
    }
}
impl Ord for HHeaderName {
    fn cmp(&self, arg: &Self) -> std::cmp::Ordering {
        self.0.as_str().cmp(arg.0.as_str())
    }
}
impl ser::Serialize for HHeaderName {
    fn serialize<S: ser::Serializer>(&self, s: S) -> Result<S::Ok,S::Error> {
        s.serialize_str(self.0.as_str())
    }
}
impl<'de> de::Deserialize<'de> for HHeaderName {
    fn deserialize<D: de::Deserializer<'de>>(d: D) -> Result<Self,D::Error> {
        d.deserialize_str(HeaderNameVisitor)
    }
}
struct HeaderNameVisitor;
impl<'de> de::Visitor<'de> for HeaderNameVisitor {
    type Value = HHeaderName;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "a valid http header name")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value,E> {
        Ok(HHeaderName(HeaderName::from_bytes(v.as_bytes())
            .map_err(|e| E::custom(format!("{:?}", e)))?))
    }
}

/*
 * Header Value Boilerplate
 *
 */
#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub struct HHeaderValue(
    pub HeaderValue
);
impl Default for HHeaderValue {
    fn default() -> Self {
        Self ( HeaderValue::from_static("") )
    }
}
impl<'de> de::Deserialize<'de> for HHeaderValue {
    fn deserialize<D: de::Deserializer<'de>>(d: D) -> Result<Self,D::Error> {
        d.deserialize_str(HeaderValueVisitor)
    }
}
impl ser::Serialize for HHeaderValue {
    fn serialize<S: ser::Serializer>(&self, s: S) -> Result<S::Ok,S::Error> {
        fn header_string_error<E: ser::Error>(h: &HeaderValue) -> Result<&str, E> {
            h.to_str()
                .map_err(|e| E::custom(format!("cannot convert header value '{:?}' to string, converstion error '{:?}'", h, e)))
        }

        s.serialize_str(header_string_error(&self.0)?)
    }
}

struct HeaderValueVisitor;
impl<'de> de::Visitor<'de> for HeaderValueVisitor {
    type Value = HHeaderValue;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "a string containing only code points 32-255 (inclusive) excluding 127 (del)")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value,E> {
        Ok(HHeaderValue(HeaderValue::from_bytes(v.as_bytes())
            .map_err(|e| E::custom(format!("{:?}", e)))?))
    }
}
