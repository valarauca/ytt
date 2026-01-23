#[cfg(test)]
mod test;

#[allow(unused_imports)]
use mirror_mirror::{
    Map, Reflect, ReflectMut, ReflectRef, ScalarMut, ScalarOwned, ScalarRef, TypeDescriptor, Value,
    Struct,TupleStruct,Tuple,Enum,Array,List,
    map::MapError,
    enum_::VariantKind,
    reflect_eq,
    type_info::{MapType, Type},
};
use std::borrow::Cow;

#[derive(Debug)]
pub enum UpdateError {
    /// Used for when 2 types have incompatible fields
    Incompatible {
        existing: Cow<'static, str>,
        update: Cow<'static, str>,
    },
    UnequalArrays,
    /// Map has non-scalar keys
    ///
    /// This is just annoying to handle and probably isn't
    /// going to happen as no serialization format supports
    /// this.
    NonScalarKeys,
    MapError(MapError),
}
impl UpdateError {
    fn incompatible(existing: &dyn Reflect, update: &dyn Reflect) -> UpdateError {
        Self::Incompatible {
            existing: existing.type_name().to_string().into(),
            update: update.type_name().to_string().into(),
        }
    }
}
impl From<MapError> for UpdateError {
    fn from(m: MapError) -> Self {
        UpdateError::MapError(m)
    }
}

pub fn update_reflect<'a, 'b>(
    old: &'a mut dyn Reflect,
    new: &'b dyn Reflect,
) -> Result<bool, UpdateError> {
    if old.type_id() != new.type_id() {
        return Err(UpdateError::incompatible(old,new));
    }

    let mut modified = false;
    match (old.reflect_mut(),new.reflect_ref()) {
        (ReflectMut::Struct(old),ReflectRef::Struct(new)) => {
            modified |= update_struct(old,new)?;
        },
        (ReflectMut::TupleStruct(old),ReflectRef::TupleStruct(new)) => {
            modified |= update_tuple_struct(old,new)?;
        },
        (ReflectMut::Tuple(old),ReflectRef::Tuple(new)) => {
            modified |= update_tuple(old,new)?;
        },
        (ReflectMut::Enum(old),ReflectRef::Enum(new)) => {
            modified |= update_enum(old,new)?;
        },
        (ReflectMut::List(old),ReflectRef::List(new)) => {
            modified |= update_list(old,new)?;
        },
        (ReflectMut::Array(old),ReflectRef::Array(new)) => {
            modified |= update_array(old,new)?;
        },
        (ReflectMut::Map(old),ReflectRef::Map(new)) => {
            modified |= update_map(old,new)?;
        },
        (ReflectMut::Scalar(old),ReflectRef::Scalar(new)) => {
            modified |= update_scalar(old,new)?;
        },
        (ReflectMut::Opaque(old),ReflectRef::Opaque(new)) => {
            if let Some(true) = reflect_eq(old,new) {
                return Ok(false);
            }
            old.patch(new);
            return Ok(true);
        }
        _ => panic!("impossible condition, types are identical"),
    };
    debug_assert_eq!(reflect_eq(old,new), Some(true));
    Ok(modified)
}

fn update_list(old: &mut dyn List, new: &dyn List) -> Result<bool, UpdateError> {
    debug_assert!(old.type_id() == new.type_id());

    let mut modified = false;
    if old.len() > new.len() {
        while let Some(_) = old.pop() {
            modified |= true;
            if old.len() == new.len() {
                break;
            }
        }
        debug_assert!(old.len() == new.len());
    }
    if new.len() > old.len() {
        let mut idx = old.len();
        loop {
            if idx >= new.len() { 
                break;
            }
            old.try_push(new.get(idx).unwrap()).unwrap();
            idx += 1;
            modified |= true;
        }
        debug_assert!(old.len() == new.len());
    }
    if old.len() == new.len() {
        modified |= update_array(old,new)?;
    }
    Ok(modified)
}

fn update_array(old: &mut dyn Array, new: &dyn Array) -> Result<bool, UpdateError> {
    debug_assert!(old.type_id() == new.type_id());

    if old.is_empty() && new.is_empty() {
        return Ok(false);
    } else if old.len() != new.len() {
        return Err(UpdateError::UnequalArrays);
    }

    let mut modified = false;
    for i in 0..old.len() {
        let old_index = old.get_mut(i).unwrap();
        let new_index = new.get(i).unwrap();
        modified |= update_reflect(old_index, new_index)?;
    }
    Ok(modified)
}

fn update_enum(old: &mut dyn Enum, new: &dyn Enum) -> Result<bool, UpdateError> {
    debug_assert!(old.type_id() == new.type_id());
   
    if old.variant_name() != new.variant_name() {
        old.patch(new);
        return Ok(true);
    }
    debug_assert_eq!(old.variant_kind(), new.variant_kind());

    match old.variant_kind() {
        VariantKind::Unit => {
            return Ok(false);
        }
        VariantKind::Tuple | VariantKind::Struct => {
            let mut modified = false;
            for i in 0..old.fields_len() {
                let old_field = old.field_at_mut(i).unwrap();
                let new_field = new.field_at(i).unwrap();
                modified |= update_reflect(old_field, new_field)?;
            }
            return Ok(modified);
        }
    };
}

fn update_tuple(old: &mut dyn Tuple, new: &dyn Tuple) -> Result<bool,UpdateError> {
    // these facts are validated in our caller
    assert!(old.type_id() == new.type_id());
    assert!(old.fields_len() == new.fields_len());

    let mut modified = false;
    for i in 0..old.fields_len() {
        // field existence was already checked/asserted
        let old_field = old.field_at_mut(i).unwrap();
        let new_field = new.field_at(i).unwrap();
        modified |= update_reflect(old_field, new_field)?;
    }
    Ok(modified)
}

fn update_tuple_struct(old: &mut dyn TupleStruct, new: &dyn TupleStruct) -> Result<bool,UpdateError> {
    // these facts are validated in our caller
    assert!(old.type_id() == new.type_id());
    assert!(old.fields_len() == new.fields_len());

    let mut modified = false;
    for i in 0..old.fields_len() {
        // field existence was already checked/asserted
        let old_field = old.field_at_mut(i).unwrap();
        let new_field = new.field_at(i).unwrap();
        modified |= update_reflect(old_field, new_field)?;
    }
    Ok(modified)
}

fn update_struct(old: &mut dyn Struct, new: &dyn Struct) -> Result<bool,UpdateError> {
    // these facts are validated in our caller
    assert!(old.type_id() == new.type_id());
    assert!(old.fields_len() == new.fields_len());

    let mut modified = false;
    for i in 0..old.fields_len() {
        // field existence was already checked/asserted
        let old_field = old.field_at_mut(i).unwrap();
        let new_field = new.field_at(i).unwrap();
        modified |= update_reflect(old_field, new_field)?;
    }
    Ok(modified)
}

fn update_map(old: &mut dyn Map, new: &dyn Map) -> Result<bool, UpdateError> {
    // this is checked in the caller
    assert!(old.type_id() == new.type_id());
    
    let mut remove = Vec::<ScalarOwned>::new();
    let mut add = Vec::<ScalarOwned>::new();

    let mut modified = key_comparison(old, new, &mut add, &mut remove)?;
    if add.is_empty() && remove.is_empty() {
        return Ok(modified);
    }
    modified = true;

    for key in remove.into_iter() {
        let k = key.as_reflect();
        old.try_remove(k)?;
    }
    for key in add.into_iter() {
        let k = key.as_reflect();
        let _ = old.try_insert(k, new.get(k).unwrap())?;
    }
    Ok(modified)
}

fn key_comparison(
    old: &mut dyn Map,
    new: &dyn Map,
    add: &mut Vec<ScalarOwned>,
    remove: &mut Vec<ScalarOwned>,
) -> Result<bool, UpdateError> {
    let mut old_iter = old.iter_mut()
         .map(|(key,value) : (&dyn Reflect, &mut dyn Reflect)| -> Result<(ScalarRef,&mut dyn Reflect),UpdateError> {
             key.as_scalar()
                 .ok_or_else(|| UpdateError::NonScalarKeys)
                 .map(|s| (s,value))
         });
    let mut new_iter = new.iter()
        .map(|(key,value) : (&dyn Reflect, &dyn Reflect)| -> Result<(ScalarRef,&dyn Reflect),UpdateError> {
            key.as_scalar()
                .ok_or_else(|| UpdateError::NonScalarKeys)
                .map(|s| (s,value))
        });

    let mut old_item: Option<(ScalarRef, &mut dyn Reflect)> = opt_err(old_iter.next())?;
    let mut new_item: Option<(ScalarRef, &dyn Reflect)> = opt_err(new_iter.next())?;
    let mut modified = false;
    loop {
        let (old_key, new_key) = match (old_item.take(), new_item.take()) {
            (None, None) => break,
            (Some(s), None) => {
                remove.push(s.0.into());
                old_item = opt_err(old_iter.next())?;
                continue;
            }
            (None, Some(s)) => {
                add.push(s.0.into());
                new_item = opt_err(new_iter.next())?;
                continue;
            }
            (Some(old_key), Some(new_key)) => (old_key, new_key),
        };
        if old_key.0 == new_key.0 {
            modified |= update_reflect(old_key.1, new_key.1)?;
            old_item = opt_err(old_iter.next())?;
            new_item = opt_err(new_iter.next())?;
        } else if old_key.0 < new_key.0 {
            remove.push(old_key.0.into());
            old_item = opt_err(old_iter.next())?;
            new_item = Some(new_key);
        } else {
            add.push(new_key.0.into());
            new_item = opt_err(new_iter.next())?;
            old_item = Some(old_key);
        }
    }
    Ok(modified)
}

fn update_scalar(
    old_scalar: ScalarMut<'_>,
    new_scalar: ScalarRef<'_>,
) -> Result<bool, UpdateError> {
    match (old_scalar, new_scalar) {
        (ScalarMut::usize(old), ScalarRef::usize(new)) => {
            if *old == new {
                return Ok(false);
            }
            *old = new;
        }
        (ScalarMut::u8(old), ScalarRef::u8(new)) => {
            if *old == new {
                return Ok(false);
            }
            *old = new;
        }
        (ScalarMut::u16(old), ScalarRef::u16(new)) => {
            if *old == new {
                return Ok(false);
            }
            *old = new;
        }
        (ScalarMut::u32(old), ScalarRef::u32(new)) => {
            if *old == new {
                return Ok(false);
            }
            *old = new;
        }
        (ScalarMut::u64(old), ScalarRef::u64(new)) => {
            if *old == new {
                return Ok(false);
            }
            *old = new;
        }
        (ScalarMut::u128(old), ScalarRef::u128(new)) => {
            if *old == new {
                return Ok(false);
            }
            *old = new;
        }
        (ScalarMut::i8(old), ScalarRef::i8(new)) => {
            if *old == new {
                return Ok(false);
            }
            *old = new;
        }
        (ScalarMut::i16(old), ScalarRef::i16(new)) => {
            if *old == new {
                return Ok(false);
            }
            *old = new;
        }
        (ScalarMut::i32(old), ScalarRef::i32(new)) => {
            if *old == new {
                return Ok(false);
            }
            *old = new;
        }
        (ScalarMut::i64(old), ScalarRef::i64(new)) => {
            if *old == new {
                return Ok(false);
            }
            *old = new;
        }
        (ScalarMut::i128(old), ScalarRef::i128(new)) => {
            if *old == new {
                return Ok(false);
            }
            *old = new;
        }
        (ScalarMut::f32(old), ScalarRef::f32(new)) => {
            if *old == new {
                return Ok(false);
            }
            *old = new;
        }
        (ScalarMut::f64(old), ScalarRef::f64(new)) => {
            if *old == new {
                return Ok(false);
            }
            *old = new;
        }
        (ScalarMut::bool(old), ScalarRef::bool(new)) => {
            if *old == new {
                return Ok(false);
            }
            *old = new;
        }
        (ScalarMut::char(old), ScalarRef::char(new)) => {
            if *old == new {
                return Ok(false);
            }
            *old = new;
        }
        (ScalarMut::Ipv4Addr(old), ScalarRef::Ipv4Addr(new)) => {
            if *old == new {
                return Ok(false);
            }
            old.clone_from(&new);
        }
        (ScalarMut::Ipv6Addr(old), ScalarRef::Ipv6Addr(new)) => {
            if *old == new {
                return Ok(false);
            }
            old.clone_from(&new);
        }
        (ScalarMut::Duration(old), ScalarRef::Duration(new)) => {
            if *old == new {
                return Ok(false);
            }
            old.clone_from(&new);
        }
        (ScalarMut::String(old), ScalarRef::String(new)) => {
            if old == new {
                return Ok(false);
            }
            old.clone_from(new);
        }
        (a, b) => {
            return Err(UpdateError::incompatible(a.as_reflect(), b.as_reflect()));
        }
    };
    Ok(true)
}

fn opt_err<T, E>(arg: Option<Result<T, E>>) -> Result<Option<T>, E> {
    match arg {
        None => Ok(None),
        Some(Ok(x)) => Ok(Some(x)),
        Some(Err(e)) => Err(e),
    }
}
