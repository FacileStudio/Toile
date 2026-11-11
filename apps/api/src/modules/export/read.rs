use std::collections::HashMap;

use yrs::{Any, Doc, GetString, Map, MapRef, Out, ReadTxn, Transact};

use super::model::{Binding, Kind, Shape, Style};

fn number(value: Option<&Any>) -> Option<f64> {
    match value? {
        Any::Number(n) => n.as_f64().filter(|n| n.is_finite()),
        _ => None,
    }
}

fn string(value: Option<&Any>) -> Option<String> {
    match value? {
        Any::String(s) => Some(s.to_string()),
        _ => None,
    }
}

fn style(value: Option<&Any>) -> Style {
    let Some(Any::Map(map)) = value else { return Style::default() };
    Style {
        fill: string(map.get("fill")),
        stroke: string(map.get("stroke")),
        width: number(map.get("width")),
        font: string(map.get("font")),
        size: number(map.get("size")),
        align: string(map.get("align")),
    }
}

fn binding(value: Option<&Any>) -> Option<Binding> {
    let Some(Any::Map(map)) = value else { return None };
    let shape = string(map.get("shape"))?;
    let Some(Any::Array(anchor)) = map.get("anchor") else { return None };
    let (u, v) = (number(anchor.first())?, number(anchor.get(1))?);
    Some(Binding { shape, anchor: [u.clamp(0.0, 1.0), v.clamp(0.0, 1.0)] })
}

fn values<T: ReadTxn>(txn: &T, map: &MapRef) -> (HashMap<String, Any>, String) {
    let mut plain = HashMap::new();
    let mut text = String::new();
    for (key, value) in map.iter(txn) {
        match value {
            Out::Any(any) => {
                plain.insert(key.to_owned(), any);
            }
            Out::YText(t) if key == "text" => text = t.get_string(txn),
            _ => {}
        }
    }
    (plain, text)
}

fn shape<T: ReadTxn>(txn: &T, id: &str, map: &MapRef) -> Option<Shape> {
    let (v, text) = values(txn, map);
    let mut shape = Shape::new(id, Kind::parse(&string(v.get("type"))?)?);
    let n = |key: &str| number(v.get(key)).unwrap_or(0.0);
    let s = |key: &str| string(v.get(key)).unwrap_or_default();
    (shape.x, shape.y, shape.w, shape.h, shape.rot) = (n("x"), n("y"), n("w").max(0.0), n("h").max(0.0), n("rot"));
    (shape.z, shape.asset, shape.url, shape.title, shape.name) = (s("z"), s("asset"), s("url"), s("title"), s("name"));
    shape.card_kind = s("kind");
    shape.parent = string(v.get("parent"));
    shape.style = style(v.get("style"));
    shape.text = text;
    if let Some(Any::Array(points)) = v.get("points") {
        shape.points = points.iter().map(|p| number(Some(p)).unwrap_or(0.0)).collect();
    }
    (shape.from, shape.to) = (binding(v.get("from")), binding(v.get("to")));
    shape.head = string(v.get("head"));
    Some(shape)
}

/// Every shape of `doc` the renderers understand, in no particular order. A shape whose type or
/// layout cannot be read is left out rather than failing the whole board.
pub fn shapes(doc: &Doc) -> Vec<Shape> {
    let root = doc.get_or_insert_map("shapes");
    let txn = doc.transact();
    root.iter(&txn)
        .filter_map(|(id, value)| match value {
            Out::YMap(map) => shape(&txn, id, &map),
            _ => None,
        })
        .collect()
}
