//! One shape of `shapes`: its type, its keys and their values.

use yrs::{Any, Map, Out, ReadTxn};

use super::schema::MAX_INK_POINTS;
use super::values::{self, Rule};

const COMMON: &[(&str, Rule)] = &[
    ("type", Rule::Str),
    ("x", Rule::Number),
    ("y", Rule::Number),
    ("w", Rule::Number),
    ("h", Rule::Number),
    ("rot", Rule::Number),
    ("z", Rule::Str),
    ("parent", Rule::Str),
    ("style", Rule::Style),
    ("locked", Rule::Bool),
];

const TEXTUAL: &[(&str, Rule)] = &[("text", Rule::Text)];
const LINE: &[(&str, Rule)] =
    &[("points", Rule::Points(2, MAX_INK_POINTS)), ("from", Rule::Binding), ("to", Rule::Binding), ("head", Rule::Str)];
const INK: &[(&str, Rule)] = &[("points", Rule::Points(3, MAX_INK_POINTS))];
const IMAGE: &[(&str, Rule)] = &[("asset", Rule::Str)];
const CARD: &[(&str, Rule)] = &[("url", Rule::Str), ("title", Rule::Str), ("kind", Rule::Str)];
const FRAME: &[(&str, Rule)] = &[("name", Rule::Str)];

/// Every shape type of schema 1.
pub const TYPES: &[&str] =
    &["sticky", "text", "rect", "ellipse", "diamond", "line", "arrow", "ink", "image", "card", "frame"];

/// The keys a shape of `kind` may hold beyond the common ones; `None` for an unknown type.
pub fn extra(kind: &str) -> Option<&'static [(&'static str, Rule)]> {
    match kind {
        "sticky" | "text" | "rect" | "ellipse" | "diamond" => Some(TEXTUAL),
        "line" | "arrow" => Some(LINE),
        "ink" => Some(INK),
        "image" => Some(IMAGE),
        "card" => Some(CARD),
        "frame" => Some(FRAME),
        _ => None,
    }
}

fn rule(kind: &'static [(&'static str, Rule)], key: &str) -> Option<Rule> {
    COMMON.iter().chain(kind).find(|(name, _)| *name == key).map(|(_, rule)| *rule)
}

/// Checks the shape `id`: a `Y.Map` with a known `type`, `x` and `y`, and only the keys its type
/// allows, each holding what schema 1 says.
pub fn check<T: ReadTxn>(txn: &T, id: &str, value: &Out) -> Result<(), String> {
    let fail = |message: String| format!("shape {id}: {message}");
    let Out::YMap(map) = value else { return Err(fail("must be a Y.Map".to_owned())) };
    if map.as_ref().block_len != 0 {
        return Err(fail("must not hold list content".to_owned()));
    }
    let kind = match map.get(txn, "type") {
        Some(Out::Any(Any::String(kind))) => kind,
        _ => return Err(fail("needs a string type".to_owned())),
    };
    let allowed = extra(&kind).ok_or_else(|| fail(format!("has an unknown type {kind}")))?;
    if !map.contains_key(txn, "x") || !map.contains_key(txn, "y") {
        return Err(fail("needs x and y".to_owned()));
    }
    map.iter(txn).try_for_each(|(key, value)| {
        let rule = rule(allowed, key).ok_or_else(|| fail(format!("has an unknown key {key}")))?;
        values::check(txn, rule, &value).map_err(|e| fail(format!("{key} {e}")))
    })
}
