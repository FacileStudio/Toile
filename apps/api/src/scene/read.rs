use yrs::{Any, GetString, Map, MapRef, Out, ReadTxn};

/// One shape as plain values, read from its `Y.Map`. Absent numbers read as 0, absent strings
/// as empty.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Shape {
    pub id: String,
    pub kind: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub z: String,
    pub parent: Option<String>,
    pub text: String,
    pub name: String,
    pub color: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub url: String,
    pub title: String,
}

fn any<T: ReadTxn>(map: &MapRef, txn: &T, key: &str) -> Option<Any> {
    match map.get(txn, key) {
        Some(Out::Any(any)) => Some(any),
        _ => None,
    }
}

fn number<T: ReadTxn>(map: &MapRef, txn: &T, key: &str) -> f64 {
    match any(map, txn, key) {
        Some(Any::Number(n)) => n.as_f64().unwrap_or(0.0),
        _ => 0.0,
    }
}

fn string<T: ReadTxn>(map: &MapRef, txn: &T, key: &str) -> String {
    match any(map, txn, key) {
        Some(Any::String(s)) => s.to_string(),
        _ => String::new(),
    }
}

fn field(value: Option<Any>, key: &str) -> Option<String> {
    match value {
        Some(Any::Map(map)) => match map.get(key) {
            Some(Any::String(s)) => Some(s.to_string()),
            _ => None,
        },
        _ => None,
    }
}

fn text<T: ReadTxn>(map: &MapRef, txn: &T) -> String {
    match map.get(txn, "text") {
        Some(Out::YText(text)) => text.get_string(txn),
        _ => String::new(),
    }
}

/// The shape `id` held in `value`; `None` for anything that is not a map.
pub fn shape<T: ReadTxn>(txn: &T, id: &str, value: &Out) -> Option<Shape> {
    let Out::YMap(map) = value else { return None };
    let style = any(map, txn, "style");
    let color = field(style.clone(), "fill").or_else(|| field(style, "stroke")).unwrap_or_default();
    let parent = Some(string(map, txn, "parent")).filter(|p| !p.is_empty());
    Some(Shape {
        id: id.to_owned(),
        kind: string(map, txn, "type"),
        x: number(map, txn, "x"),
        y: number(map, txn, "y"),
        w: number(map, txn, "w"),
        h: number(map, txn, "h"),
        z: string(map, txn, "z"),
        parent,
        text: text(map, txn),
        name: string(map, txn, "name"),
        color,
        from: field(any(map, txn, "from"), "shape"),
        to: field(any(map, txn, "to"), "shape"),
        url: string(map, txn, "url"),
        title: string(map, txn, "title"),
    })
}

/// Every shape of `shapes`, in paint order: by `z`, then by id.
pub fn all<T: ReadTxn>(txn: &T, shapes: &MapRef) -> Vec<Shape> {
    let mut out: Vec<Shape> = shapes.iter(txn).filter_map(|(id, value)| shape(txn, id, &value)).collect();
    out.sort_by(|a, b| (a.z.as_str(), a.id.as_str()).cmp(&(b.z.as_str(), b.id.as_str())));
    out
}
