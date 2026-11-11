use std::collections::HashMap;

use yrs::{Any, Doc, Map, MapPrelim, Text, TextPrelim, Transact};

use super::schema::{self, MAX_TEXT};
use super::{SHAPES, from_state, new_doc, seed};

fn add(doc: &Doc, id: &str, fields: Vec<(&str, Any)>) {
    let shapes = doc.get_or_insert_map(SHAPES);
    let mut txn = doc.transact_mut();
    shapes.insert(&mut txn, id, fields.into_iter().collect::<MapPrelim>());
}

fn sticky(text: &str) -> Doc {
    let doc = from_state(&seed(), []).unwrap();
    add(&doc, "a", vec![("type", "sticky".into()), ("x", 1.0.into()), ("y", Any::from(2))]);
    let shapes = doc.get_or_insert_map(SHAPES);
    let mut txn = doc.transact_mut();
    let shape = shapes.get(&txn, "a").unwrap().cast::<yrs::MapRef>().unwrap();
    shape.insert(&mut txn, "text", TextPrelim::new(text));
    drop(txn);
    doc
}

#[test]
fn a_seeded_board_and_a_sticky_with_text_pass() {
    assert!(schema::check_all(&from_state(&seed(), []).unwrap()).is_ok());
    assert!(schema::check_all(&sticky("hello")).is_ok());
}

#[test]
fn text_past_the_limit_is_refused() {
    let doc = sticky("x");
    let shapes = doc.get_or_insert_map(SHAPES);
    let mut txn = doc.transact_mut();
    let text = shapes.get(&txn, "a").unwrap().cast::<yrs::MapRef>().unwrap().get(&txn, "text").unwrap();
    text.cast::<yrs::TextRef>().unwrap().push(&mut txn, &"y".repeat(MAX_TEXT));
    drop(txn);
    assert!(schema::check_all(&doc).unwrap_err().contains("longer than"));
}

#[test]
fn unknown_types_keys_and_bad_numbers_are_refused() {
    let base = || vec![("type", Any::from("rect")), ("x", 0.0.into()), ("y", 0.0.into())];
    let cases: Vec<(Vec<(&str, Any)>, &str)> = vec![
        (vec![("type", "blob".into()), ("x", 0.0.into()), ("y", 0.0.into())], "unknown type"),
        ([base(), vec![("colour", "red".into())]].concat(), "unknown key"),
        ([base(), vec![("w", f64::NAN.into())]].concat(), "finite"),
        ([base(), vec![("h", 2e7.into())]].concat(), "finite"),
        ([base(), vec![("points", Any::from(vec![1.0, 2.0]))]].concat(), "unknown key"),
        (vec![("type", "rect".into()), ("x", 0.0.into())], "needs x and y"),
        ([base(), vec![("locked", "yes".into())]].concat(), "boolean"),
    ];
    for (fields, expected) in cases {
        let doc = new_doc();
        add(&doc, "s", fields);
        let error = schema::check_all(&doc).unwrap_err();
        assert!(error.contains(expected), "{error} does not say {expected}");
    }
}

#[test]
fn ink_points_come_in_threes_and_bindings_have_unit_anchors() {
    let ink = |points: Vec<f64>| {
        vec![("type", "ink".into()), ("x", 0.0.into()), ("y", 0.0.into()), ("points", points.into())]
    };
    let doc = new_doc();
    add(&doc, "ok", ink(vec![1.0, 2.0, 0.5]));
    assert!(schema::check_all(&doc).is_ok());
    add(&doc, "bad", ink(vec![1.0, 2.0]));
    assert!(schema::check_all(&doc).unwrap_err().contains("points"));
    let binding = |a: f64| {
        let anchor = Any::from(vec![a, 0.5]);
        Any::from(HashMap::from([("shape".to_owned(), Any::from("ok")), ("anchor".to_owned(), anchor)]))
    };
    let arrow = |a: f64| vec![("type", "arrow".into()), ("x", 0.0.into()), ("y", 0.0.into()), ("from", binding(a))];
    let doc = new_doc();
    add(&doc, "a", arrow(1.0));
    assert!(schema::check_all(&doc).is_ok());
    add(&doc, "b", arrow(1.5));
    assert!(schema::check_all(&doc).unwrap_err().contains("anchor"));
}

#[test]
fn foreign_roots_and_a_wrong_schema_are_refused() {
    let doc = new_doc();
    doc.get_or_insert_map("extra").insert(&mut doc.transact_mut(), "k", 1);
    assert!(schema::check_all(&doc).unwrap_err().contains("unknown root"));
    let doc = new_doc();
    doc.get_or_insert_map(super::META).insert(&mut doc.transact_mut(), "schema", 2);
    assert!(schema::check_all(&doc).unwrap_err().contains("meta.schema"));
}

#[test]
fn the_watch_names_the_shapes_a_transaction_touched() {
    let doc = sticky("hi");
    let touched = schema::watch(&doc);
    let shapes = doc.get_or_insert_map(SHAPES);
    {
        let mut txn = doc.transact_mut();
        let shape = shapes.get(&txn, "a").unwrap().cast::<yrs::MapRef>().unwrap();
        shape.insert(&mut txn, "x", 9.0);
    }
    add(&doc, "b", vec![("type", "frame".into()), ("x", 0.0.into()), ("y", 0.0.into())]);
    let seen = touched.lock().unwrap();
    assert!(!seen.all);
    assert_eq!(seen.shapes.len(), 2);
}
