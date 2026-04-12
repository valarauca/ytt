use std::{
	collections::{
        btree_set::{BTreeSet},
        btree_map::{BTreeMap,Entry},
    },
    borrow::Cow,
};

use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, Visitor,Error,Unexpected},
};

use super::method::{Method};

#[derive(Clone,PartialEq,Eq, Debug)]
pub struct MethodConfig {
    pub info: BTreeMap<String,BTreeSet<Method>>,
}

impl<'de> Deserialize<'de> for MethodConfig {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(MethodConfigVisitor)
    }
}


struct MethodConfigVisitor;

impl<'de> Visitor<'de> for MethodConfigVisitor {

    type Value = MethodConfig;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a map of method specs to service strings")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {

        let mut info: BTreeMap<String, BTreeSet<Method>> = BTreeMap::new();
        let mut seen = BTreeSet::<Method>::new();

        while let Some((key, service)) = map.next_entry::<String, String>()? {

            let methods = Method::parse_method_key::<A::Error>(key.as_ref())?;

            // a.k.a.: Do this sets overlap
            if let Some(same) = seen.intersection(&methods).next() {
                let msg = format!("route has overlapping verbs '{}'", same);
                return Err(<A::Error as Error>::invalid_value(Unexpected::Str(&key), &(msg.as_str())));
            }
            seen.extend(methods.iter().cloned());
            match info.entry(service) {
                Entry::Vacant(e) => {
                    e.insert(methods);
                }
                Entry::Occupied(mut e) => {
                    e.get_mut().extend(methods);
                }
            };
        }

        Ok(MethodConfig { info })
    }
}
