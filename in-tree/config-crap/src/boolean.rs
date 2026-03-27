use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self, Visitor};
use std::fmt;

/// Boolean type, repr exists to save size for optional fields
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum Boolean {
    True = 1,
    False = 2,
}
impl Default for Boolean {
    fn default() -> Self { Self::False }
}
impl Boolean {

    /// Converts to a boolean
    pub fn as_bool(&self) -> bool {
        match self {
            &Self::True => true,
            &Self::False => false,
        }
    }
    /// Converts to a boolean
    pub fn to_bool(&self) -> bool {
        match self {
            &Self::True => true,
            &Self::False => false,
        }
    }
}
impl std::ops::BitAnd<Boolean> for Boolean {
    type Output = bool;
    fn bitand(self, rhs: Boolean) -> bool {
        self.to_bool() & rhs.to_bool()
    }
}
impl std::ops::BitXor<Boolean> for Boolean {
    type Output = bool;
    fn bitxor(self, rhs: Boolean) -> bool {
        self.to_bool() ^ rhs.to_bool()
    }
}
impl std::ops::BitOr<Boolean> for Boolean {
    type Output = bool;
    fn bitor(self, rhs: Boolean) -> bool {
        self.to_bool() | rhs.to_bool()
    }
}
impl std::ops::Not for Boolean {
    type Output = bool;
    fn not(self) -> bool {
        !self.to_bool()
    }
}
impl From<&&bool> for Boolean {
    fn from(b: &&bool) -> Self {
        if **b {
            Self::True
        } else {
            Self::False
        }
    }
}
impl From<&bool> for Boolean {
    fn from(b: &bool) -> Self {
        if *b {
            Self::True
        } else {
            Self::False
        }
    }
}
impl From<bool> for Boolean {
    fn from(b: bool) -> Self {
        if b {
            Self::True
        } else {
            Self::False
        }
    }
}
impl From<&&Boolean> for bool {
    fn from(b: &&Boolean) -> Self {
        b.to_bool()
    }
}
impl From<&Boolean> for bool {
    fn from(b: &Boolean) -> Self {
        b.to_bool()
    }
}
impl From<Boolean> for bool {
    fn from(b: Boolean) -> Self {
        b.to_bool()
    }
}
const FALSE_CONST: &'static bool = &false;
const TRUE_CONST: &'static bool = &true;
impl AsRef<bool> for Boolean {
    fn as_ref<'a>(&'a self) -> &'a bool {
        if self.to_bool() {
            TRUE_CONST
        } else {
            FALSE_CONST
        }
    }
}
impl std::ops::Deref for Boolean {
    type Target = bool;
    fn deref<'a>(&'a self) -> &'a Self::Target {
        if self.to_bool() {
            TRUE_CONST
        } else {
            FALSE_CONST
        }
    }
}

impl Serialize for Boolean {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Boolean::True => serializer.serialize_bool(true),
            Boolean::False => serializer.serialize_bool(false),
        }
    }
}

impl<'de> Deserialize<'de> for Boolean {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(BooleanVisitor)
    }
}

struct BooleanVisitor;

impl<'de> Visitor<'de> for BooleanVisitor {
    type Value = Boolean;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a boolean, integer (0/1), or string representing a boolean value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(if value { Boolean::True } else { Boolean::False })
    }

    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_i64(value as i64)
    }

    fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_i64(value as i64)
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_i64(value as i64)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            1 => Ok(Boolean::True),
            0 => Ok(Boolean::False),
            _ => Err(E::custom(format!("invalid integer for Boolean: {}. Expected 0 or 1", value))),
        }
    }

    fn visit_i128<E>(self, value: i128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            1 => Ok(Boolean::True),
            0 => Ok(Boolean::False),
            _ => Err(E::custom(format!("invalid integer for Boolean: {}. Expected 0 or 1", value))),
        }
    }

    fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_u64(value as u64)
    }

    fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_u64(value as u64)
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_u64(value as u64)
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            1 => Ok(Boolean::True),
            0 => Ok(Boolean::False),
            _ => Err(E::custom(format!("invalid integer for Boolean: {}. Expected 0 or 1", value))),
        }
    }

    fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            1 => Ok(Boolean::True),
            0 => Ok(Boolean::False),
            _ => Err(E::custom(format!("invalid integer for Boolean: {}. Expected 0 or 1", value))),
        }
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {

        let normalized = value.to_lowercase();
        match normalized.as_str().trim() {
            "true" | "yes" | "t" | "y" | "positive" | "ack" => Ok(Boolean::True),
            "false" | "no" | "f" | "n" | "negative" | "nack" => Ok(Boolean::False),
            _ => Err(E::custom(format!(
                "invalid string for Boolean: '{}'. Expected one of: true, yes, t, y, positive, ack, false, no, f, n, negative, nack (case insensitive)",
                value
            ))),
        }
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_serialize() {
        assert_eq!(serde_json::to_string(&Boolean::True).unwrap(), "true");
        assert_eq!(serde_json::to_string(&Boolean::False).unwrap(), "false");
    }

    #[test]
    fn test_deserialize_bool_json_string() {
        assert_eq!(serde_json::from_str::<Boolean>("true").unwrap(), Boolean::True);
        assert_eq!(serde_json::from_str::<Boolean>("false").unwrap(), Boolean::False);
    }

    #[test]
    fn test_deserialize_integer() {
        assert_eq!(serde_json::from_str::<Boolean>("1").unwrap(), Boolean::True);
        assert_eq!(serde_json::from_str::<Boolean>("0").unwrap(), Boolean::False);
        
        // Test error cases
        assert!(serde_json::from_str::<Boolean>("2").is_err());
        assert!(serde_json::from_str::<Boolean>("-1").is_err());
    }

    #[test]
    fn test_deserialize_string() {
        // True values
        assert_eq!(serde_json::from_str::<Boolean>("\"true\"").unwrap(), Boolean::True);
        assert_eq!(serde_json::from_str::<Boolean>("\"TRUE\"").unwrap(), Boolean::True);
        assert_eq!(serde_json::from_str::<Boolean>("\"yes\"").unwrap(), Boolean::True);
        assert_eq!(serde_json::from_str::<Boolean>("\"YES\"").unwrap(), Boolean::True);
        assert_eq!(serde_json::from_str::<Boolean>("\"t\"").unwrap(), Boolean::True);
        assert_eq!(serde_json::from_str::<Boolean>("\"T\"").unwrap(), Boolean::True);
        assert_eq!(serde_json::from_str::<Boolean>("\"y\"").unwrap(), Boolean::True);
        assert_eq!(serde_json::from_str::<Boolean>("\"Y\"").unwrap(), Boolean::True);
        assert_eq!(serde_json::from_str::<Boolean>("\"positive\"").unwrap(), Boolean::True);
        assert_eq!(serde_json::from_str::<Boolean>("\"POSITIVE\"").unwrap(), Boolean::True);
        assert_eq!(serde_json::from_str::<Boolean>("\"ack\"").unwrap(), Boolean::True);
        assert_eq!(serde_json::from_str::<Boolean>("\"ACK\"").unwrap(), Boolean::True);

        // False values
        assert_eq!(serde_json::from_str::<Boolean>("\"false\"").unwrap(), Boolean::False);
        assert_eq!(serde_json::from_str::<Boolean>("\"faLSe\"").unwrap(), Boolean::False);
        assert_eq!(serde_json::from_str::<Boolean>("\"no\"").unwrap(), Boolean::False);
        assert_eq!(serde_json::from_str::<Boolean>("\"NO\"").unwrap(), Boolean::False);
        assert_eq!(serde_json::from_str::<Boolean>("\"f\"").unwrap(), Boolean::False);
        assert_eq!(serde_json::from_str::<Boolean>("\"F\"").unwrap(), Boolean::False);
        assert_eq!(serde_json::from_str::<Boolean>("\"n\"").unwrap(), Boolean::False);
        assert_eq!(serde_json::from_str::<Boolean>("\"N\"").unwrap(), Boolean::False);
        assert_eq!(serde_json::from_str::<Boolean>("\"negative\"").unwrap(), Boolean::False);
        assert_eq!(serde_json::from_str::<Boolean>("\"NEGATIVE\"").unwrap(), Boolean::False);
        assert_eq!(serde_json::from_str::<Boolean>("\"nack\"").unwrap(), Boolean::False);
        assert_eq!(serde_json::from_str::<Boolean>("\"NACK\"").unwrap(), Boolean::False);

        // Error cases
        assert!(serde_json::from_str::<Boolean>("\"invalid\"").is_err());
        assert!(serde_json::from_str::<Boolean>("\"maybe\"").is_err());
    }
}
