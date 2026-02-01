macro_rules! limited_float {
    (
        pub struct $type: ident
        where
            Low: $low: expr,
            High: $high: expr,
        {
            inner: f64,
        }
    ) => {
        #[allow(dead_code)]
        #[derive(Clone,Copy,PartialEq,PartialOrd)]
        #[repr(transparent)]
        pub struct $type {
            inner: f64,
        }
        impl $type {
            #[allow(dead_code)]
            pub fn clamp_new(arg: f64) -> Self {
                let low: f64 = { $low } as f64;
                let high: f64 = { $high } as f64;
                let arg = if arg.is_nan() {
                    0.0f64
                } else if arg.is_infinite() {
                    0.0f64
                } else {
                    arg
                };
                Self {
                    inner: f64::clamp(arg, low, high),
                }
            }
        }
        impl std::fmt::Debug for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <f64 as std::fmt::Debug>::fmt(&self.inner, f)
            }
        }
        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <f64 as std::fmt::Display>::fmt(&self.inner, f)
            }
        }
        impl std::ops::Add<Self> for $type {
            type Output = $type;
            fn add(self, rhs: Self) -> Self::Output {
                Self::clamp_new(self.inner + rhs.inner)
            }
        }
        impl std::ops::Sub<Self> for $type {
            type Output = $type;
            fn sub(self, rhs: Self) -> Self::Output {
                Self::clamp_new(self.inner - rhs.inner)
            }
        }
        impl std::ops::Mul<Self> for $type {
            type Output = $type;
            fn mul(self, rhs: Self) -> Self::Output {
                Self::clamp_new(self.inner * rhs.inner)
            }
        }
        impl std::ops::Div<Self> for $type {
            type Output = $type;
            fn div(self, rhs: Self) -> Self::Output {
                Self::clamp_new(self.inner / rhs.inner)
            }
        }
        impl std::ops::Neg for $type {
            type Output = $type;
            fn neg(self) -> Self::Output {
                let x = <f64 as std::ops::Neg>::neg(self.inner);
                Self::clamp_new(x)
            }
        }
        impl Eq for $type { }
        impl Ord for $type {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                if self.inner == other.inner {
                    std::cmp::Ordering::Equal
                } else if self.inner < other.inner {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            }
        }
        impl std::hash::Hash for $type {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                /*
                 * f64 bit layout is extremely standardized
                 * once you make NaN & Inf unrepresentable.
                 * This method 'largely' exists to ensure
                 * we're compatible we caching system(s) which
                 * will reside in RAM.
                 * Amusing RISC-V, ARM, and AMD64 use the same
                 * float format, for this is even portable.
                 * 
                 */
                self.inner.to_bits().hash(state);
            }
        }
        impl std::convert::TryFrom<f64> for $type {
            type Error = anyhow::Error;
            fn try_from(x: f64) -> Result<Self,anyhow::Error> {
                let name: &'static str = std::any::type_name::<Self>();
                let low: f64 = { $low } as f64;
                let high: f64 = { $high } as f64;
                if x.is_nan() {
                    anyhow::bail!("{} cannot be NaN", name);
                } else if x.is_infinite() {
                    anyhow::bail!("{} cannot be +/-Inf", name);
                } else if x < low {
                    anyhow::bail!("{} cannot be less than {}", name, low);
                } else if x > high {
                    anyhow::bail!("{} cannot be greater than {}", name, high);
                }
                Ok($type { inner: x })
            }
        }
        impl std::convert::TryFrom<f32> for $type {
            type Error = anyhow::Error;
            fn try_from(x: f32) -> Result<Self,anyhow::Error> {
                Self::try_from(x as f64)
            }
        }
        impl<'de> serde::de::Deserialize<'de> for $type {
            fn deserialize<D>(d: D) -> Result<Self,D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                let x: f64 = <f64 as serde::de::Deserialize>::deserialize(d)?;
                Self::try_from(x)
                    .map_err(|e| <D::Error as serde::de::Error>::custom(e))
            }
        }
        impl serde::ser::Serialize for $type {
            fn serialize<S>(&self, s: S) -> Result<S::Ok,S::Error>
            where
                S: serde::ser::Serializer,
            {
                s.serialize_f64(self.inner)
            }
        }
        impl From<$type> for f64 {
            fn from(x: $type) -> f64 {
                x.inner
            }
        }
        impl mlua::UserData for $type {
            fn add_methods<M: mlua::UserDataMethods<$type>>(m: &mut M) {
                m.add_function("clamp_new", |_, arg: mlua::Value| -> mlua::Result<Self> {
                    if !arg.is_integer() || !arg.is_number() {
                        return Err(
                        mlua::Error::FromLuaConversionError{ from: std::any::type_name::<mlua::Value>(), to: std::any::type_name::<Self>().to_string(), message: Some("only numeric & integer are supported".to_string()) });
                    }

                    let arg = if arg.is_integer() {
                        arg.as_isize().unwrap() as f64
                    } else {
                        arg.as_f64().unwrap()
                    };

                    Ok(Self::clamp_new(arg))
                });
                m.add_meta_function(mlua::MetaMethod::Eq, |_, arg: (mlua::UserDataRef<Self>,mlua::UserDataRef<Self>)| -> mlua::Result<bool> {
                    let lhs = *arg.0;
                    let rhs = *arg.1;
                    Ok(lhs == rhs)
                });
                m.add_meta_method(mlua::MetaMethod::ToString, |_, this: &Self, _: () | -> mlua::Result<String> {
                    Ok(this.inner.to_string())
                });
            }
        }
    };
}


limited_float! {
    pub struct Temperature
    where
        Low: 0.0f64,
        High: 2.0f64,
    {
        inner: f64,
    }
}

limited_float! {
    pub struct Bias
    where
        Low: -100.0f64,
        High: 100.0f64,
    {
        inner: f64,
    }
}

limited_float! {
    pub struct MinP
    where
        Low: 0.0f64,
        High: 1.0f64,
    {
        inner: f64,
    }
}

limited_float! {
    pub struct RepetitionPenalty
    where
        Low: 0.0f64,
        High: 2.0f64,
    {
        inner: f64,
    }
}

limited_float! {
    pub struct TopA
    where
        Low: 0.0f64,
        High: 1.0f64,
    {
        inner: f64,
    }
}

limited_float! {
    pub struct PresencePenalty
    where
        Low: -2.0f64,
        High: 2.0f64,
    {
        inner: f64,
    }
}

limited_float! {
    pub struct FrequencyPenalty
    where
        Low: -2.0f64,
        High: 2.0f64,
    {
        inner: f64,
    }
}

