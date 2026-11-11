mod common;

use std::collections::{BTreeMap, BTreeSet};

use api::modules::board::schema;
use api::modules::export::jsoncanvas::{self, Canvas};
use api::modules::export::model::{Kind, Shape};
use api::modules::export::read;
use api::modules::import::{build, excalidraw, jsoncanvas as canvas_import};

const CANVAS: &[u8] = include_bytes!("fixtures/board.canvas");
const EXCALIDRAW: &[u8] = include_bytes!("fixtures/scene.excalidraw");

fn through_a_board(shapes: &[Shape]) -> Vec<Shape> {
    let doc = build::document(shapes);
    schema::check_all(&doc).unwrap();
    read::shapes(&doc)
}

type NodeKey = (String, String, i64, i64, i64, i64);

fn nodes(canvas: &Canvas) -> BTreeSet<NodeKey> {
    let key = |n: &api::modules::export::jsoncanvas::Node| {
        (n.id.clone(), n.kind.clone(), n.x as i64, n.y as i64, n.width as i64, n.height as i64)
    };
    canvas.nodes.iter().filter(|n| n.kind != "widget").map(key).collect()
}

fn edges(canvas: &Canvas) -> BTreeSet<[String; 5]> {
    let side = |s: &Option<String>| s.clone().unwrap_or_default();
    canvas
        .edges
        .iter()
        .map(|e| [e.id.clone(), e.from_node.clone(), side(&e.from_side), e.to_node.clone(), side(&e.to_side)])
        .collect()
}

#[test]
fn a_json_canvas_round_trip_keeps_nodes_and_edges() {
    let imported = canvas_import::import(CANVAS).unwrap();
    assert_eq!(imported.skipped, 3, "the unknown node, the dangling edge and one edge label");
    let shapes = through_a_board(&imported.shapes);
    let (out, skipped) = jsoncanvas::export(&shapes.iter().collect::<Vec<_>>(), "/api/boards/b/assets/");
    assert_eq!(skipped, 0);
    let source: Canvas = serde_json::from_slice(CANVAS).unwrap();
    assert_eq!(nodes(&out), nodes(&source));
    let mut want = edges(&source);
    want.retain(|e| e[3] != "ghost");
    assert_eq!(edges(&out), want);
    let texts: BTreeMap<_, _> = out.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    assert_eq!(texts["note1"].text.as_deref(), Some("# Idée\nUne note"));
    assert_eq!(texts["link1"].url.as_deref(), Some("https://jsoncanvas.org"));
    assert_eq!(texts["file1"].file.as_deref(), Some("Notes/plan.md"));
    assert_eq!(texts["group1"].label.as_deref(), Some("Sprint"));
    let ends: Vec<_> = out.edges.iter().map(|e| (e.from_end.as_deref(), e.to_end.as_deref())).collect();
    assert!(ends.contains(&(Some("arrow"), Some("none"))) && ends.contains(&(Some("none"), Some("arrow"))));
}

#[test]
fn json_canvas_groups_become_frames_holding_their_nodes() {
    let shapes = through_a_board(&canvas_import::import(CANVAS).unwrap().shapes);
    let by_id: BTreeMap<_, _> = shapes.iter().map(|s| (s.id.as_str(), s)).collect();
    assert_eq!(by_id["group1"].kind, Kind::Frame);
    assert_eq!(by_id["note1"].parent.as_deref(), Some("group1"));
    assert_eq!(by_id["file1"].parent, None);
    assert_eq!(by_id["note1"].style.fill.as_deref(), Some("#e0de71"));
    assert_eq!(by_id["edge2"].head.as_deref(), Some("start"));
}

#[test]
fn an_excalidraw_scene_imports_every_supported_element() {
    let imported = excalidraw::import(EXCALIDRAW).unwrap();
    assert_eq!(imported.skipped, 2, "the image and the arrow label");
    let shapes = through_a_board(&imported.shapes);
    let by_id: BTreeMap<_, _> = shapes.iter().map(|s| (s.id.as_str(), s)).collect();
    let kinds: BTreeSet<_> = shapes.iter().map(|s| s.kind.name()).collect();
    assert_eq!(kinds, ["arrow", "diamond", "ellipse", "frame", "ink", "line", "rect", "text"].into());
    assert!(!by_id.contains_key("label-1") && !by_id.contains_key("gone-1"));
    let rect = by_id["rect-1"];
    assert_eq!((rect.text.as_str(), rect.parent.as_deref()), ("Début", Some("frame-1")));
    assert_eq!((rect.style.font.as_deref(), rect.style.stroke.as_deref()), (Some("mono"), Some("#e03131")));
    let arrow = by_id["arrow-1"];
    assert_eq!(arrow.from.as_ref().map(|b| (b.shape.as_str(), b.anchor)), Some(("rect-1", [1.0, 0.5])));
    assert_eq!(arrow.to.as_ref().map(|b| (b.shape.as_str(), b.anchor)), Some(("ellipse-1", [0.0, 0.5])));
    assert_eq!(arrow.head.as_deref(), Some("end"));
    assert_eq!(by_id["draw-1"].points, vec![0.0, 0.0, 0.2, 10.0, 5.0, 0.5, 20.0, 10.0, 0.8, 50.0, 20.0, 0.4]);
    assert_eq!(by_id["ellipse-1"].rot, 0.5);
    assert_eq!(by_id["frame-1"].name, "Atelier");
}

#[test]
fn imported_shapes_stack_in_file_order() {
    let keys: Vec<String> = (0..4000).step_by(61).map(build::z_key).collect();
    assert!(keys.windows(2).all(|w| w[0] < w[1]), "{keys:?}");
    assert_eq!((build::z_key(0), build::z_key(61), build::z_key(62)), ("a0".into(), "az".into(), "b00".into()));
    assert_eq!(build::z_key(62 + 3844), "c000");
}

#[test]
fn files_that_are_not_boards_are_refused() {
    assert!(excalidraw::import(b"{\"type\":\"other\",\"elements\":[]}").is_err());
    assert!(canvas_import::import(b"not json").is_err());
    assert!(excalidraw::import(b"{}").is_err());
}
