//! Checks of the plain values a shape holds.

use yrs::{Any, GetString, Out, ReadTxn};

use super::schema::{MAX_COORD, MAX_TEXT};

/// What one shape key may hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// A finite number within ±[`MAX_COORD`].
    Number,
    /// A plain string of at most [`MAX_TEXT`] characters.
    Str,
    /// `true` or `false`.
    Bool,
    /// A plain object of `fill`, `stroke`, `width`, `font`, `size`, `align`.
    Style,
    /// A `Y.Text` of at most [`MAX_TEXT`] characters.
    Text,
    /// A flat number array whose length is a multiple of the stride, with at most this many points.
    Points(usize, usize),
    /// `{ shape, anchor: [0..1, 0..1] }`.
    Binding,
}

fn plain(value: &Out) -> Result<&Any, String> {
    match value {
        Out::Any(any) => Ok(any),
        _ => Err("must be a plain value".to_owned()),
    }
}

/// A finite number within ±[`MAX_COORD`].
pub fn number(any: &Any) -> Result<f64, String> {
    let n = match any {
        Any::Number(n) => n.as_f64().ok_or("must be a number")?,
        _ => return Err("must be a number".to_owned()),
    };
    match n.is_finite() && n.abs() <= MAX_COORD {
        true => Ok(n),
        false => Err(format!("must be finite and within ±{MAX_COORD}")),
    }
}

fn string(any: &Any) -> Result<(), String> {
    match any {
        Any::String(s) if s.chars().count() <= MAX_TEXT => Ok(()),
        Any::String(_) => Err(format!("is longer than {MAX_TEXT} characters")),
        _ => Err("must be a string".to_owned()),
    }
}

fn style(any: &Any) -> Result<(), String> {
    let Any::Map(map) = any else { return Err("must be an object".to_owned()) };
    map.iter().try_for_each(|(key, value)| match key.as_str() {
        "fill" | "stroke" | "font" | "align" => string(value).map_err(|e| format!("{key} {e}")),
        "width" | "size" => number(value).map(|_| ()).map_err(|e| format!("{key} {e}")),
        _ => Err(format!("has an unknown key {key}")),
    })
}

fn points(any: &Any, stride: usize, max: usize) -> Result<(), String> {
    let Any::Array(items) = any else { return Err("must be a number array".to_owned()) };
    if items.len() % stride != 0 || items.len() / stride > max {
        return Err(format!("must hold at most {max} points of {stride} numbers"));
    }
    items.iter().try_for_each(|item| number(item).map(|_| ()))
}

fn binding(any: &Any) -> Result<(), String> {
    let Any::Map(map) = any else { return Err("must be an object".to_owned()) };
    if map.keys().any(|key| key != "shape" && key != "anchor") {
        return Err("may hold only shape and anchor".to_owned());
    }
    string(map.get("shape").ok_or("needs a shape")?)?;
    let Some(Any::Array(anchor)) = map.get("anchor") else { return Err("needs an anchor".to_owned()) };
    let unit = |a: &Any| number(a).is_ok_and(|n| (0.0..=1.0).contains(&n));
    match anchor.len() == 2 && anchor.iter().all(unit) {
        true => Ok(()),
        false => Err("anchor must be two numbers in 0..1".to_owned()),
    }
}

/// Checks `value` against `rule`.
pub fn check<T: ReadTxn>(txn: &T, rule: Rule, value: &Out) -> Result<(), String> {
    match rule {
        Rule::Text => match value {
            Out::YText(text) if text.get_string(txn).chars().count() <= MAX_TEXT => Ok(()),
            Out::YText(_) => Err(format!("is longer than {MAX_TEXT} characters")),
            _ => Err("must be a Y.Text".to_owned()),
        },
        Rule::Number => number(plain(value)?).map(|_| ()),
        Rule::Str => string(plain(value)?),
        Rule::Bool => matches!(plain(value)?, Any::Bool(_)).then_some(()).ok_or_else(|| "must be a boolean".to_owned()),
        Rule::Style => style(plain(value)?),
        Rule::Points(stride, max) => points(plain(value)?, stride, max),
        Rule::Binding => binding(plain(value)?),
    }
}
