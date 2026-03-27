//! For all Range Integers
//!
//! The MIN/MAX are treated as `std::ops::RangeInclusive` or `[MIN,MAX]`
//! this is to permit a number to encode its own maximum & minimum.
//!

macro_rules! range_int_def {
    (
        $(pub struct $name: ident<$kind: ty> -> $ser: ident );*
        $(;)*
    ) => {
        $(
        #[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]
        #[repr(transparent)]
        pub struct $name<const MIN: $kind, const MAX: $kind> {
            data: $kind,
        }
        impl<const MIN: $kind, const MAX: $kind> $name<MIN,MAX> {


            const fn range() -> std::ops::RangeInclusive<$kind> { MIN..=MAX }

            /// If `arg` is `None` or out of range, it returns self.
            ///
            /// This purpose of this function is to sanatize user inputs
            /// against a known good value from a configuration.
            pub fn check_or_set_default<T>(&self, arg: Option<T>) -> $kind
            where
                $kind: std::convert::TryFrom<T>,
            {
                use std::convert::TryFrom;
                match arg.map(|x| <$kind as TryFrom<T>>::try_from(x).ok()).flatten() {
                    Some(x) => {
                        if x >= MIN && x <= MAX {
                            x
                        } else {
                            self.data
                        }
                    }
                    _ => self.data,
                }
            }

            fn from_abstract<T,X,E>(arg: T) -> Result<Self,E>
            where
                T: Copy + std::fmt::Display,
                X: std::fmt::Display,
                $kind: std::convert::TryFrom<T,Error=X>,
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                use std::any::type_name;

                let data: $kind = <$kind as TryFrom<T>>::try_from(arg)
                    .map_err(|e| E::custom(format!("cannot convert: {} into {}, error: {}", type_name::<T>(), type_name::<$kind>(), e)))?;
                if Self::range().contains(&data) {
                    Ok(Self { data })
                } else {
                    Err(E::custom(format!("input value: '{}' out-of-bounds, must be in [{}..={}]", arg, MIN, MAX)))
                }
            }
        }
        impl<const MIN: $kind, const MAX: $kind> AsRef<$kind> for $name<MIN,MAX> {
            fn as_ref<'a>(&'a self) -> &'a $kind { &self.data }
        }
        impl<const MIN: $kind, const MAX: $kind> std::ops::Deref for $name<MIN,MAX> {
            type Target = $kind;
            fn deref<'a>(&'a self) -> &'a $kind { &self.data }
        }
        impl<const MIN: $kind, const MAX: $kind> PartialEq<$kind> for $name<MIN,MAX> {
            fn eq(&self, rhs: &$kind) -> bool {
                self.data.eq(rhs)
            }
        }
        impl<const MIN: $kind, const MAX: $kind> PartialOrd<$kind> for $name<MIN,MAX> {
            fn partial_cmp(&self, rhs: &$kind) -> Option<std::cmp::Ordering> {
                self.data.partial_cmp(rhs)
            }
        }
        impl<const MIN: $kind, const MAX: $kind> From<$name<MIN,MAX>> for $kind {
            fn from(x: $name<MIN,MAX>) -> $kind { x.data }
        }
        impl<const MIN: $kind, const MAX: $kind> std::convert::TryFrom<$kind> for $name<MIN,MAX> {
            type Error = std::ops::RangeInclusive<$kind>;
            fn try_from(value: $kind) -> Result<Self,Self::Error> {
                let r = Self::range();
                if r.contains(&value) {
                    Ok(Self { data: value }) 
                } else {
                    Err(r)
                }
            }
        }
        impl<const MIN: $kind, const MAX: $kind> serde::ser::Serialize for $name<MIN,MAX> {
            fn serialize<S: serde::ser::Serializer>(&self, s: S) -> Result<S::Ok,S::Error> {
                s.$ser(self.data)
            }
        }
        impl<'de,const MIN: $kind, const MAX: $kind> serde::de::Deserialize<'de> for $name<MIN,MAX> {
            fn deserialize<D>(d: D) -> Result<Self,D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {

                struct RangeIntVisitor<const MIN: $kind, const MAX: $kind>;

                impl<'de, const MIN: $kind, const MAX: $kind> serde::de::Visitor<'de> for RangeIntVisitor<MIN,MAX> {
                    type Value = $name<MIN,MAX>;

                    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(f, "integer value of type: '{}' in range [{}..={}]", std::any::type_name::<$kind>(), MIN, MAX)
                    }

                    fn visit_i64<E>(self, v: i64) -> Result<Self::Value,E>
                    where
                        E: serde::de::Error,
                    {
                        Self::Value::from_abstract::<i64,_,E>(v)
                    }

                    fn visit_u64<E>(self, v: u64) -> Result<Self::Value,E>
                    where
                        E: serde::de::Error,
                    {
                        Self::Value::from_abstract::<u64,_,E>(v)
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value,E>
                    where
                        E: serde::de::Error,
                    {
                        let v = v.trim();
                        if let Some(v) = v.strip_prefix("Oo") {
                            let x: i128 = i128::from_str_radix(v, 8)
                                .map_err(|e| E::custom(e))?;
                            return Self::Value::from_abstract::<i128,_,E>(x);
                        }
                        if let Some(v) = v.strip_prefix("Ob") {
                            let x: i128 = i128::from_str_radix(v, 2)
                                .map_err(|e| E::custom(e))?;
                            return Self::Value::from_abstract::<i128,_,E>(x);
                        }
                        if let Some(v) = v.strip_prefix("Ox") {
                            let x: i128 = i128::from_str_radix(v, 16)
                                .map_err(|e| E::custom(e))?;
                            return Self::Value::from_abstract::<i128,_,E>(x);
                        }
                        let x = i128::from_str_radix(v,10)
                            .map_err(|e| E::custom(e))?;
                        Self::Value::from_abstract::<i128,_,E>(x)
                    }
                }

                d.deserialize_any(RangeIntVisitor::<MIN,MAX>)
            }
        }
        )*
    };
}

range_int_def! {
    pub struct U8Range<u8> -> serialize_u8;
    pub struct U16Range<u16> -> serialize_u16;
    pub struct U32Range<u32> -> serialize_u32;
    pub struct U64Range<u64> -> serialize_u64;

    pub struct I8Range<i8> -> serialize_i8;
    pub struct I16Range<i16> -> serialize_i16;
    pub struct I32Range<i32> -> serialize_i32;
    pub struct I64Range<i64> -> serialize_i64;
}


#[test]
fn check_range_is_valid() {
    use serde::{Serialize,Deserialize};
    use serde_json::from_str;

    #[derive(Serialize,Deserialize)]
    struct TestCase{ value: U8Range<1u8,100u8> }
    const DUT: &'static str = r#"{ "value": 15 }"#;

    let x = from_str::<TestCase>(&DUT).unwrap();
    assert_eq!(x.value, 15u8);
}

#[test]
#[should_panic]
fn check_range_is_invalid() {
    use serde::{Serialize,Deserialize};
    use serde_json::from_str;

    #[derive(Serialize,Deserialize)]
    struct TestCase{ value: U8Range<1u8,100u8> }
    const DUT: &'static str = r#"{ "value": 101 }"#;
    from_str::<TestCase>(&DUT).unwrap();
}
