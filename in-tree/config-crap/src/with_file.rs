use std::borrow::Cow;
use std::fmt::{Debug,Display};
use std::marker::PhantomData;
use std::path::{PathBuf,Path};
use std::fs::{OpenOptions,canonicalize};
use std::io::{BufReader,Read};

use serde::de::{Deserializer,DeserializeOwned,Visitor, Error as DError,Deserialize};

use crate::middleware::Format;

/// Read a totally different file and deserialize it.
pub struct WithFile<F,T> {
    _marker: PhantomData<fn(F)>,
    data: T,
}
impl<F,T: Clone> Clone for WithFile<F,T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            _marker: PhantomData,
        }
    }
}
impl<F,T: Debug> Debug for WithFile<F,T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <T as Debug>::fmt(&self.data, f)
    }
}
impl<F,T: Display> Display for WithFile<F,T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <T as Display>::fmt(&self.data, f)
    }
}
impl<F,T> AsRef<T> for WithFile<F,T> {
    fn as_ref<'a>(&'a self) -> &'a T {
        &self.data
    }
}
impl<F,T> std::ops::Deref for WithFile<F,T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a Self::Target {
        &self.data
    }
}

#[derive(Default)]
struct StringVisitor {
    _sized: usize,
}
impl<'de> Visitor<'de> for StringVisitor {
    type Value = Cow<'de,str>;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "a valid path")
    }
    fn visit_borrowed_str<E: DError>(self, v: &'de str) -> Result<Self::Value,E> {
        Ok(Cow::from(v))
    }
    fn visit_string<E: DError>(self, v: String) -> Result<Self::Value,E> {
        Ok(Cow::from(v))
    }
    fn visit_str<E: DError>(self, v: &str) -> Result<Self::Value,E> {
        Ok(Cow::from(v.to_string()))
    }
}

impl<'de,F,T> Deserialize<'de> for WithFile<F,T>
where
    T: DeserializeOwned,
    F: Format,
{
    fn deserialize<DD>(d: DD) -> Result<Self,DD::Error>
    where
        DD: Deserializer<'de>,
    {
        let input: Cow<'de,str> = d.deserialize_str(StringVisitor::default())?;
        let input_path = Path::new(input.as_ref());
        let p = if input_path.is_file() {
            Cow::from(input_path)
        } else {
            let new_path: PathBuf = canonicalize(&input_path)
                .map_err(|e| DD::Error::custom(format!("recieved input: '{}' failed to canonicalize this path, error: '{:?}'", input.as_ref(), e)))?;
            new_path.try_exists()
                .map_err(|e| DD::Error::custom(format!("recieved input: '{}', canonicalized this into: '{:?}', calling stat returned error: '{:?}'", input.as_ref(), &new_path, e)))?;
            Cow::from(new_path)
        };
        let mut s = String::with_capacity(4096);
        { 
            let f = OpenOptions::new()
                .read(true)
                .write(false)
                .create(false)
                .open(p.as_ref())
                .map_err(|e| DD::Error::custom(format!("recieved input: '{}', converted to path as: '{:?}', opening handle returned: '{:?}'", input.as_ref(), p.as_ref(), e)))?;
            // Magic Number: Buffer Size, 2MiB (2 * 1024 * 1024).
            //
            // Why?
            //
            // Most SSDs have blocks in the 64KiB-256KiB range. If you
            // want to do a continious read, not partial read, 64K ain't
            // enough for anyone.
            let mut r = BufReader::with_capacity(2 * 1024 * 1024, f);
            r.read_to_string(&mut s)
            .map_err(|e| DD::Error::custom(format!("recieved input: '{}', converted to path as: '{:?}', error occured while reading file: '{:?}'", input.as_ref(), p.as_ref(), e)))?;
        }
        let data: T = F::deserialize_str::<T,DD::Error>(s.as_str())?;
        Ok(Self {
            data,
            _marker: PhantomData,
        })
    }
}
