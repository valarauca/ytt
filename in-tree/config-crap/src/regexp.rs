use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self};

use crate::boolean::Boolean;

/// Regexp pattern for serializing/deserializing regex.
#[derive(Clone, Debug)]
pub struct Regexp {
    regexp: Regex,
    caps: RegexpCaptureGroups,
    opts: Option<Box<RegexConfig>>,
}

impl Regexp {
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        let regexp = Regex::new(pattern)?;
        let caps = RegexpCaptureGroups::new(&regexp);
        Ok(Regexp {
            regexp,
            caps: caps,
            opts: None,
        })
    }

    /// Returns the regexp
    pub fn regexp<'a>(&'a self) -> &'a Regex {
        &self.regexp
    }

    /// Returns information about the captures
    pub fn capture_group_info<'a>(&'a self) -> &'a RegexpCaptureGroups {
        &self.caps
    }
    
    /// Returns the string representation of a regexp
    pub fn as_str(&self) -> &str {
        self.regexp.as_str()
    }

    fn should_serialize_opts<'a>(&'a self) -> Option<&'a RegexConfig> {
        self.opts.as_ref()
            .into_iter()
            .filter_map(|config| -> Option<&'a RegexConfig> {
                if config.requires_serialization() {
                    Some(config)
                } else {
                    None
                }
            })
            .next()
    }

    fn with_config(config: RegexConfig) -> Result<Self, regex::Error> {
        let regexp = config.build_regex()?;
        let caps = RegexpCaptureGroups::new(&regexp);
        let opts = if config.requires_serialization() {
            Some(Box::new(config))
        } else {
            None
        };
        
        Ok(Regexp { regexp, caps, opts })
    }
}
impl AsRef<Regex> for Regexp {
    fn as_ref<'a>(&'a self) -> &'a Regex {
        &self.regexp
    }
}
impl std::ops::Deref for Regexp {
    type Target = Regex;
    fn deref<'a>(&'a self) -> &'a Self::Target {
        &self.regexp
    }
}
impl PartialEq for Regexp {
    fn eq(&self, other: &Self) -> bool {
        self.regexp.as_str().eq(other.regexp.as_str())
    }
}
impl PartialEq<str> for Regexp {
    fn eq(&self, other: &str) -> bool {
        self.regexp.as_str().eq(other)
    }
}
impl PartialEq<String> for Regexp {
    fn eq(&self, other: &String) -> bool {
        self.regexp.as_str().eq(other.as_str())
    }
}
impl Eq for Regexp { }
impl PartialOrd for Regexp {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.regexp.as_str().partial_cmp(other.regexp.as_str())
    }
}
impl Ord for Regexp {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.regexp.as_str().cmp(other.regexp.as_str())
    }
}
impl std::hash::Hash for Regexp {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.regexp.as_str().hash(state)
    }
}

/// Useful for inspecting regexes when you expect
/// them to capture specific values
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegexpCaptureGroups {
    groups: Box<[Option<String>]>,
}
impl RegexpCaptureGroups {

    fn new(r: &Regex) -> Self {
        let mut caps = r.capture_names();
        let _ = caps.next().expect("group 0 is implicit and should always be present");
        Self {
            groups: caps
                .map(|x| x.map(String::from))
                .collect::<Vec<Option<String>>>()
                .into_boxed_slice(),
        }
    }

    /// returns if this type has capture groups (beyond the `0` implict group)
    pub fn has_capture_groups(&self) -> bool {
        !self.groups.is_empty()
    }
    /// returns if a name is present in a regexp
    pub fn has_name<S: AsRef<str>>(&self, name: S) -> bool {
        self.groups.iter().filter_map(Option::as_ref).any(|x| x.as_str() == name.as_ref())
    }
    /// returns if a group is present
    pub fn has_group(&self, group: usize) -> bool {
        let x = match group.checked_sub(1) {
            None => return true,
            Some(x) => x,
        };
        x < self.groups.len()
    }
    /// converts a name into a group number, if it exists
    pub fn name_to_group_number<S: AsRef<str>>(&self, name: S) -> Option<usize> {
        self.groups.iter()
            .enumerate()
            .map(|(idx,s)| (idx+1,s))
            .filter_map(|(idx,s)| s.as_ref().map(|s| (idx,s)))
            .filter_map(|(idx,s)| (s == name.as_ref()).then(|| idx))
            .next()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
enum LineTerminator {
    #[serde(rename = "CRLF", alias = "crlf")]
    Crlf = 1,
    #[serde(rename = "CR", alias = "Cr", alias = "cr")]
    Cr = 2,
    #[serde(rename = "LF", alias = "lf")]
    Lf = 3,
    #[serde(rename = "NUL", alias = "nul", alias = "NULL", alias = "null")]
    Null = 4,
}

impl LineTerminator {
    fn setup_builder<'a>(&self, builder: &'a mut RegexBuilder) -> &'a mut RegexBuilder {
        match self {
            &Self::Crlf => builder.crlf(true),
            &Self::Cr => builder.line_terminator(0x0Du8),
            &Self::Lf => builder.line_terminator(0x0Au8),
            &Self::Null => builder.line_terminator(0x00u8),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct RegexConfig {
    regexp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    case_insensitive: Option<Boolean>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dfa_size_limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dot_matches_new_line: Option<Boolean>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ignore_whitespace: Option<Boolean>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line_terminator: Option<LineTerminator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    multi_line: Option<Boolean>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nest_limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    octal: Option<Boolean>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size_limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unicode: Option<Boolean>,
}

impl Default for RegexConfig {
    fn default() -> Self {
        RegexConfig {
            regexp: String::new(),
            case_insensitive: None,
            dfa_size_limit: None,
            dot_matches_new_line: None,
            ignore_whitespace: None,
            line_terminator: None,
            multi_line: None,
            nest_limit: None,
            octal: None,
            size_limit: None,
            unicode: None,
        }
    }
}

impl RegexConfig {
    fn requires_serialization(&self) -> bool {
        self != &Self::default()
    }

    fn build_regex(&self) -> Result<Regex, regex::Error> {
        let mut builder = RegexBuilder::new(&self.regexp);

        if let Some(ref case_insensitive) = self.case_insensitive {
            builder.case_insensitive(case_insensitive.as_bool());
        }

        if let Some(dfa_size_limit) = self.dfa_size_limit {
            builder.dfa_size_limit(dfa_size_limit);
        }

        if let Some(ref dot_matches_new_line) = self.dot_matches_new_line {
            builder.dot_matches_new_line(dot_matches_new_line.as_bool());
        }

        if let Some(ref ignore_whitespace) = self.ignore_whitespace {
            builder.ignore_whitespace(ignore_whitespace.as_bool());
        }

        if let Some(line_terminator) = self.line_terminator {
            line_terminator.setup_builder(&mut builder);
        }

        if let Some(ref multi_line) = self.multi_line {
            builder.multi_line(multi_line.as_bool());
        }

        if let Some(nest_limit) = self.nest_limit {
            builder.nest_limit(nest_limit);
        }

        if let Some(ref octal) = self.octal {
            builder.octal(octal.as_bool());
        }

        if let Some(size_limit) = self.size_limit {
            builder.size_limit(size_limit);
        }

        if let Some(ref unicode) = self.unicode {
            builder.unicode(unicode.as_bool());
        }

        builder.build()
    }
}

/// This type is provided to assist with deserialization
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Deserialize)]
#[serde(untagged)]
enum RegexDeserializeHelper {
    Complex(RegexConfig),
    Default(String),
}

impl Serialize for Regexp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.should_serialize_opts() {
            Some(opts) => opts.serialize(serializer),
            None => self.regexp.as_str().serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Regexp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = RegexDeserializeHelper::deserialize(deserializer)?;
        
        match helper {
            RegexDeserializeHelper::Default(pattern) => {
                Regexp::new(&pattern)
                    .map_err(|e| de::Error::custom(format!("Invalid regex pattern '{}': {}", pattern, e)))
            }
            RegexDeserializeHelper::Complex(config) => {
                let pattern = config.regexp.clone();
                Regexp::with_config(config)
                    .map_err(|e| de::Error::custom(format!("Invalid regex pattern '{}': {}", pattern, e)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_simple_string_roundtrip() {
        let pattern = r"hello\s+world";
        let regexp = Regexp::new(pattern).expect("Valid regex");
        
        let serialized = serde_json::to_string(&regexp).expect("Serialization should work");
        
        let deserialized: Regexp = serde_json::from_str(&serialized).expect("Deserialization should work");
        
        assert_eq!(regexp.as_str(), deserialized.as_str());
        assert_eq!(regexp.should_serialize_opts(), deserialized.should_serialize_opts());
    }

    #[test]
    fn test_complex_config_roundtrip() {
        let config = RegexConfig {
            regexp: r"test\d+".to_string(),
            case_insensitive: Some(Boolean::from(true)),
            multi_line: Some(Boolean::from(true)),
            unicode: Some(Boolean::from(false)),
            ..Default::default()
        };
        
        let regexp = Regexp::with_config(config).expect("Valid regex config");
        
        let serialized = serde_json::to_string(&regexp).expect("Serialization should work");
        
        let deserialized: Regexp = serde_json::from_str(&serialized).expect("Deserialization should work");
        
        assert_eq!(regexp.as_str(), deserialized.as_str());
        assert_eq!(regexp.should_serialize_opts(), deserialized.should_serialize_opts());
    }

    #[test]
    fn test_string_to_config_to_string() {
        let simple_json = r#""simple_pattern""#;
        let regexp1: Regexp = serde_json::from_str(simple_json).expect("Should deserialize");
        
        let config = RegexConfig {
            regexp: regexp1.as_str().to_string(),
            case_insensitive: Some(Boolean::from(true)),
            ..Default::default()
        };
        
        let regexp2 = Regexp::with_config(config).expect("Should create from config");
        
        let serialized = serde_json::to_string(&regexp2).expect("Should serialize");
        
        let regexp3: Regexp = serde_json::from_str(&serialized).expect("Should deserialize");
        
        assert_eq!(regexp1.as_str(), regexp2.as_str());
        assert_eq!(regexp2.as_str(), regexp3.as_str());
        
        assert!(!regexp1.should_serialize_opts().is_some());
        assert!(regexp2.should_serialize_opts().is_some());
        assert!(regexp3.should_serialize_opts().is_some());
    }

    #[test]
    fn test_invalid_regex_error() {
        let invalid_json = r#""[invalid regex"#;
        let result: Result<Regexp, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_line_terminator_serialization() {
        let config = RegexConfig {
            regexp: "test".to_string(),
            line_terminator: Some(LineTerminator::Crlf),
            ..Default::default()
        };

        let regexp = Regexp::with_config(config).expect("Valid config");
        let serialized = serde_json::to_string(&regexp).expect("Should serialize");
        let deserialized: Regexp = serde_json::from_str(&serialized).expect("Should deserialize");
        
        assert_eq!(regexp.as_str(), deserialized.as_str());
    }

    #[test]
    fn test_regexp_capture_groups_named() {
        let r = Regexp::new(r#"(?<year>[0-9]{4})-(?<month>[0-9]{2})-(?<day>[0-9])"#).unwrap();
        let caps = r.capture_group_info();

        assert!(caps.has_capture_groups());
        assert!(caps.has_group(0));

        assert!(caps.has_name("year"));
        assert_eq!(caps.name_to_group_number("year").unwrap(), 1usize);
        assert!(caps.has_group(1));

        assert!(caps.has_name("month"));
        assert_eq!(caps.name_to_group_number("month").unwrap(), 2usize);
        assert!(caps.has_group(2));

        assert!(caps.has_name("day"));
        assert_eq!(caps.name_to_group_number("day").unwrap(), 3usize);
        assert!(caps.has_group(3));
    }
}
