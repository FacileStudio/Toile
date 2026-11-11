use std::collections::{HashMap, HashSet};

use yrs::{Any, In, Map, MapPrelim, MapRef, Out, TextPrelim, TransactionMut};

use super::input::{Kind, Planned};
use super::read::{self, Shape};

fn binding(id: &str) -> In {
    let anchor: Vec<Any> = vec![0.5.into(), 0.5.into()];
    In::Any(Any::from(HashMap::from([("shape".to_owned(), Any::from(id)), ("anchor".to_owned(), Any::from(anchor))])))
}

/// The `style` holding `color` as the fill or the stroke of `kind`.
pub fn style(kind: Kind, color: &str) -> Any {
    let key = if kind.stroked() { "stroke" } else { "fill" };
    Any::from(HashMap::from([(key.to_owned(), Any::from(color))]))
}

fn path(p: &Planned, fields: &mut Vec<(&'static str, In)>) {
    let points: Vec<Any> = [0.0, 0.0, p.end.0, p.end.1].into_iter().map(Any::from).collect();
    fields.push(("points", In::Any(Any::from(points))));
    if let Some(from) = &p.from {
        fields.push(("from", binding(from)));
    }
    if let Some(to) = &p.to {
        fields.push(("to", binding(to)));
    }
    if p.placed.kind == Kind::Arrow {
        fields.push(("head", In::Any("end".into())));
    }
}

/// The `Y.Map` content of `p`: only keys that hold something, as schema 1 wants.
pub fn prelim(p: &Planned) -> MapPrelim {
    let s = &p.placed;
    let mut fields: Vec<(&'static str, In)> = vec![("type", In::Any(s.kind.name().into()))];
    for (key, value) in [("x", s.x), ("y", s.y), ("w", s.w), ("h", s.h)] {
        fields.push((key, In::Any(value.into())));
    }
    fields.push(("z", In::Any(p.z.as_str().into())));
    if let Some(parent) = &p.parent {
        fields.push(("parent", In::Any(parent.as_str().into())));
    }
    if let Some(color) = &p.color {
        fields.push(("style", In::Any(style(s.kind, color))));
    }
    match s.kind {
        Kind::Frame => fields.push(("name", In::Any(p.text.as_str().into()))),
        kind if kind.is_path() => path(p, &mut fields),
        _ => fields.push(("text", In::from(TextPrelim::new(p.text.as_str())))),
    }
    fields.into_iter().collect()
}

/// Writes every planned shape into `shapes`.
pub fn insert(txn: &mut TransactionMut, shapes: &MapRef, planned: &[Planned]) {
    for p in planned {
        shapes.insert(txn, p.placed.id.as_str(), prelim(p));
    }
}

/// `ids` and every shape inside a frame among them, as the client deletes a frame.
pub fn with_children(all: &[Shape], ids: &[String]) -> HashSet<String> {
    let mut gone: HashSet<String> = ids.iter().cloned().collect();
    for shape in all {
        if shape.parent.as_ref().is_some_and(|p| gone.contains(p)) {
            gone.insert(shape.id.clone());
        }
    }
    gone
}

/// Removes `ids` and their frames' children from `shapes`; answers the ids removed. An id the
/// board does not hold is refused.
pub fn remove(txn: &mut TransactionMut, shapes: &MapRef, ids: &[String]) -> Result<Vec<String>, String> {
    let all = read::all(txn, shapes);
    if let Some(missing) = ids.iter().find(|id| !matches!(shapes.get(txn, id.as_str()), Some(Out::YMap(_)))) {
        return Err(format!("no shape {missing:?} on the board"));
    }
    let mut gone: Vec<String> = with_children(&all, ids).into_iter().collect();
    gone.sort();
    for id in &gone {
        shapes.remove(txn, id);
    }
    Ok(gone)
}
