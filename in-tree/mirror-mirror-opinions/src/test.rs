use std::collections::BTreeMap;
use mirror_mirror::Reflect;
use crate::update_reflect;

#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct MyStruct {
    inner: Vec<Option<usize>>,
}

#[test]
fn test_equal_length_no_change() {
    let mut old = MyStruct {
        inner: vec![Some(1), None, Some(3)],
    };
    let new = MyStruct {
        inner: vec![Some(1), None, Some(3)],
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(!result, "expected no modification when values are identical");
    assert_eq!(old.inner, vec![Some(1), None, Some(3)]);
}

#[test]
fn test_option_none_equality() {
    let mut old = Option::<usize>::None;
    let new = Option::<usize>::None;
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(!result, "no updates should occur");
    assert_eq!(&old,&new);
}

#[test]
fn test_option_some_equality() {
    let mut old = Option::<usize>::Some(1usize);
    let new = Option::<usize>::Some(1usize);
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(!result, "no updates should occur");
    assert_eq!(&old,&new);
}

#[test]
fn test_equal_length_with_changes() {
    let mut old = MyStruct {
        inner: vec![Some(1), None, Some(3)],
    };
    let new = MyStruct {
        inner: vec![Some(10), Some(20), None],
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when values differ");
    assert_eq!(old.inner, vec![Some(10), Some(20), None]);
}

#[test]
fn test_equal_length_partial_change() {
    let mut old = MyStruct {
        inner: vec![Some(1), None, Some(3)],
    };
    let new = MyStruct {
        inner: vec![Some(1), Some(2), Some(3)],
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when middle element changes");
    assert_eq!(old.inner, vec![Some(1), Some(2), Some(3)]);
}

#[test]
fn test_vector_grows() {
    let mut old = MyStruct {
        inner: vec![Some(1)],
    };
    let new = MyStruct {
        inner: vec![Some(1), Some(2), Some(3)],
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when vector grows");
    assert_eq!(old.inner, vec![Some(1), Some(2), Some(3)]);
}

#[test]
fn test_vector_shrinks() {
    let mut old = MyStruct {
        inner: vec![Some(1), Some(2), Some(3)],
    };
    let new = MyStruct {
        inner: vec![Some(1)],
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when vector shrinks");
    assert_eq!(old.inner, vec![Some(1)]);
}

#[test]
fn test_empty_to_populated() {
    let mut old = MyStruct { inner: vec![] };
    let new = MyStruct {
        inner: vec![Some(1), None, Some(3)],
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when going from empty to populated");
    assert_eq!(old.inner, vec![Some(1), None, Some(3)]);
}

#[test]
fn test_populated_to_empty() {
    let mut old = MyStruct {
        inner: vec![Some(1), None, Some(3)],
    };
    let new = MyStruct { inner: vec![] };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when going from populated to empty");
    assert_eq!(old.inner, Vec::<Option<usize>>::new());
}

#[test]
fn test_both_empty() {
    let mut old = MyStruct { inner: vec![] };
    let new = MyStruct { inner: vec![] };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(!result, "expected no modification when both are empty");
    assert_eq!(old.inner, Vec::<Option<usize>>::new());
}

#[test]
fn test_grow_with_value_changes() {
    let mut old = MyStruct {
        inner: vec![Some(1), Some(2)],
    };
    let new = MyStruct {
        inner: vec![Some(10), Some(20), Some(30), Some(40)],
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when growing with different values");
    assert_eq!(old.inner, vec![Some(10), Some(20), Some(30), Some(40)]);
}

#[test]
fn test_shrink_with_value_changes() {
    let mut old = MyStruct {
        inner: vec![Some(1), Some(2), Some(3), Some(4)],
    };
    let new = MyStruct {
        inner: vec![Some(10), Some(20)],
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when shrinking with different values");
    assert_eq!(old.inner, vec![Some(10), Some(20)]);
}

#[test]
fn test_none_to_some_transitions() {
    let mut old = MyStruct {
        inner: vec![None, None, None],
    };
    let new = MyStruct {
        inner: vec![Some(1), Some(2), Some(3)],
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when None becomes Some");
    assert_eq!(old.inner, vec![Some(1), Some(2), Some(3)]);
}

#[test]
fn test_some_to_none_transitions() {
    let mut old = MyStruct {
        inner: vec![Some(1), Some(2), Some(3)],
    };
    let new = MyStruct {
        inner: vec![None, None, None],
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when Some becomes None");
    assert_eq!(old.inner, vec![None, None, None]);
}

#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct MyItem {
    field_a: BTreeMap<u64, Option<bool>>,
}

#[test]
fn test_map_no_change() {
    let mut old = MyItem {
        field_a: BTreeMap::from([(1, Some(true)), (2, None), (3, Some(false))]),
    };
    let new = MyItem {
        field_a: BTreeMap::from([(1, Some(true)), (2, None), (3, Some(false))]),
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(!result, "expected no modification when maps are identical");
    assert_eq!(old.field_a, new.field_a);
}

#[test]
fn test_map_same_keys_different_values() {
    let mut old = MyItem {
        field_a: BTreeMap::from([(1, Some(true)), (2, None), (3, Some(false))]),
    };
    let new = MyItem {
        field_a: BTreeMap::from([(1, Some(false)), (2, Some(true)), (3, None)]),
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when values differ");
    assert_eq!(old.field_a, BTreeMap::from([(1, Some(false)), (2, Some(true)), (3, None)]));
}

#[test]
fn test_map_add_keys() {
    let mut old = MyItem {
        field_a: BTreeMap::from([(1, Some(true))]),
    };
    let new = MyItem {
        field_a: BTreeMap::from([(1, Some(true)), (2, Some(false)), (3, None)]),
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when keys are added");
    assert_eq!(old.field_a, BTreeMap::from([(1, Some(true)), (2, Some(false)), (3, None)]));
}

#[test]
fn test_map_remove_keys() {
    let mut old = MyItem {
        field_a: BTreeMap::from([(1, Some(true)), (2, Some(false)), (3, None)]),
    };
    let new = MyItem {
        field_a: BTreeMap::from([(1, Some(true))]),
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when keys are removed");
    assert_eq!(old.field_a, BTreeMap::from([(1, Some(true))]));
}

#[test]
fn test_map_both_empty() {
    let mut old = MyItem {
        field_a: BTreeMap::new(),
    };
    let new = MyItem {
        field_a: BTreeMap::new(),
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(!result, "expected no modification when both maps empty");
    assert_eq!(old.field_a, BTreeMap::new());
}

#[test]
fn test_map_empty_to_populated() {
    let mut old = MyItem {
        field_a: BTreeMap::new(),
    };
    let new = MyItem {
        field_a: BTreeMap::from([(1, Some(true)), (2, None)]),
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when going from empty to populated");
    assert_eq!(old.field_a, BTreeMap::from([(1, Some(true)), (2, None)]));
}

#[test]
fn test_map_populated_to_empty() {
    let mut old = MyItem {
        field_a: BTreeMap::from([(1, Some(true)), (2, None)]),
    };
    let new = MyItem {
        field_a: BTreeMap::new(),
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when going from populated to empty");
    assert_eq!(old.field_a, BTreeMap::new());
}

#[test]
fn test_map_mixed_changes() {
    let mut old = MyItem {
        field_a: BTreeMap::from([(1, Some(true)), (2, Some(false)), (3, None)]),
    };
    let new = MyItem {
        field_a: BTreeMap::from([(2, Some(true)), (3, None), (4, Some(false))]),
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification with mixed key changes");
    assert_eq!(old.field_a, BTreeMap::from([(2, Some(true)), (3, None), (4, Some(false))]));
}

#[test]
fn test_map_disjoint_keys() {
    let mut old = MyItem {
        field_a: BTreeMap::from([(1, Some(true)), (2, Some(false))]),
    };
    let new = MyItem {
        field_a: BTreeMap::from([(3, None), (4, Some(true))]),
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when keys are completely different");
    assert_eq!(old.field_a, BTreeMap::from([(3, None), (4, Some(true))]));
}

#[test]
fn test_map_single_value_change() {
    let mut old = MyItem {
        field_a: BTreeMap::from([(1, Some(true)), (2, Some(false)), (3, None)]),
    };
    let new = MyItem {
        field_a: BTreeMap::from([(1, Some(true)), (2, Some(true)), (3, None)]),
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when single value changes");
    assert_eq!(old.field_a, BTreeMap::from([(1, Some(true)), (2, Some(true)), (3, None)]));
}

#[test]
fn test_map_none_to_some_values() {
    let mut old = MyItem {
        field_a: BTreeMap::from([(1, None), (2, None), (3, None)]),
    };
    let new = MyItem {
        field_a: BTreeMap::from([(1, Some(true)), (2, Some(false)), (3, Some(true))]),
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when None becomes Some");
    assert_eq!(old.field_a, BTreeMap::from([(1, Some(true)), (2, Some(false)), (3, Some(true))]));
}

#[test]
fn test_map_some_to_none_values() {
    let mut old = MyItem {
        field_a: BTreeMap::from([(1, Some(true)), (2, Some(false)), (3, Some(true))]),
    };
    let new = MyItem {
        field_a: BTreeMap::from([(1, None), (2, None), (3, None)]),
    };
    let result = update_reflect(&mut old, &new).unwrap();
    assert!(result, "expected modification when Some becomes None");
    assert_eq!(old.field_a, BTreeMap::from([(1, None), (2, None), (3, None)]));
}
