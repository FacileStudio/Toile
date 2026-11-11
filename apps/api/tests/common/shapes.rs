use std::path::PathBuf;

use api::modules::export::model::{Binding, Kind, Shape};
use api::modules::export::read;
use api::modules::export::svg::{self, Drawing, Options};
use api::modules::import::build;

pub fn shape(id: &str, kind: Kind, at: [f64; 4]) -> Shape {
    let mut shape = Shape::new(id, kind);
    [shape.x, shape.y, shape.w, shape.h] = at;
    shape
}

pub fn bind(shape: &str, anchor: [f64; 2]) -> Option<Binding> {
    Some(Binding { shape: shape.to_owned(), anchor })
}

pub fn draw(shapes: &[Shape], frame: Option<&str>) -> Option<Drawing> {
    let doc = build::document(shapes);
    let read = read::shapes(&doc);
    svg::render(&read, &Options { frame, background: None, asset_base: "/api/boards/b/assets/" })
}

pub fn snapshot(name: &str, svg: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots").join(format!("{name}.svg"));
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(&path, svg).unwrap();
        return;
    }
    let want =
        std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("no snapshot {name}; run with UPDATE_SNAPSHOTS=1"));
    assert_eq!(svg, want, "snapshot {name} changed; rerun with UPDATE_SNAPSHOTS=1 if intended");
}
