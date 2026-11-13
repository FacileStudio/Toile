use super::*;
use serde_json::{Value, json};

fn boxes(shapes: &[NewShape]) -> Vec<(f64, f64, f64, f64)> {
    let stickies = shapes.iter().filter(|s| s.kind == Kind::Sticky);
    stickies.map(|s| (s.x.unwrap(), s.y.unwrap(), s.w.unwrap(), s.h.unwrap())).collect()
}

fn apart(shapes: &[NewShape]) -> bool {
    let boxes = boxes(shapes);
    boxes.iter().enumerate().all(|(i, a)| {
        boxes[i + 1..].iter().all(|b| a.0 + a.2 <= b.0 || b.0 + b.2 <= a.0 || a.1 + a.3 <= b.1 || b.1 + b.3 <= a.1)
    })
}

fn chart(decision: &str) -> Flowchart {
    let nodes: Vec<Value> = (0..6).map(|i| json!({"id": format!("w{i}"), "text": "Wide", "kind": "step"})).collect();
    let mut all = vec![json!({"id": "s", "text": "Start", "kind": "start"})];
    all.extend(nodes);
    all.push(json!({"id": "d", "text": "Ok?", "kind": decision}));
    let mut edges: Vec<Value> = (0..6).map(|i| json!({"from": "s", "to": format!("w{i}")})).collect();
    edges.push(json!({"from": "w0", "to": "d"}));
    serde_json::from_value(json!({"nodes": all, "edges": edges})).unwrap()
}

#[test]
fn a_flowchart_rows_follow_the_edges_and_bad_edges_drop() {
    let chart: Flowchart = serde_json::from_value(json!({
        "nodes": [
            {"id": "s", "text": "Start", "kind": "start"},
            {"id": "d", "text": "Ok?", "kind": "decision"},
            {"id": "e", "text": "Done", "kind": "end"},
            {"id": "s", "text": "again", "kind": "step"}
        ],
        "edges": [{"from": "s", "to": "d"}, {"from": "d", "to": "e"}, {"from": "e", "to": "d"}, {"from": "d", "to": "zz"}]
    }))
    .unwrap();
    let shapes = flowchart(chart, None);
    let ys: Vec<f64> = shapes.iter().filter_map(|s| s.y).collect();
    assert_eq!(ys, vec![0.0, 300.0, 600.0]);
    assert_eq!(shapes.iter().filter(|s| s.kind == Kind::Arrow).count(), 3);
    assert_eq!(shapes.iter().filter(|s| s.kind == Kind::Sticky).count(), 3);
    assert_eq!(shapes[0].color.as_deref(), Some("#A9EE7E"));
    assert_eq!(shapes[1].color.as_deref(), Some("#EE7E89"));
}

#[test]
fn a_wide_flowchart_stays_apart_and_its_decision_takes_a_valid_accent() {
    assert!(apart(&flowchart(chart("decision"), None)));
    let color = |accent| flowchart(chart("decision"), accent).into_iter().find(|s| s.text.as_deref() == Some("Ok?"));
    assert_eq!(color(Some("#12aB9f")).unwrap().color.as_deref(), Some("#12aB9f"));
    for bad in ["red", "#12345", "#1234567", "#12345g", "#ééé", "123456#"] {
        assert_eq!(color(Some(bad)).unwrap().color.as_deref(), Some("#EE7E89"), "{bad}");
    }
    let step = flowchart(chart("step"), Some("#000000")).into_iter().find(|s| s.text.as_deref() == Some("Ok?"));
    assert_eq!(step.unwrap().color.as_deref(), Some("#AD9EF0"));
}

#[test]
fn a_mindmap_puts_colored_branches_on_both_sides_of_the_title() {
    let map: Mindmap = serde_json::from_value(json!({
        "title": "Launch",
        "branches": [{"text": "Site", "children": ["Copy", "Hero"]}, {"text": "Press", "children": []}]
    }))
    .unwrap();
    let shapes = mindmap(map);
    let get = |handle: &str| shapes.iter().find(|s| s.handle.as_deref() == Some(handle)).unwrap();
    let x = |handle: &str| get(handle).x.unwrap();
    assert!(x("b0") > x("root") && x("b1") < x("root") && x("b0l0") > x("b0"));
    assert_eq!(shapes.iter().filter_map(|s| s.x).fold(f64::INFINITY, f64::min), 0.0);
    assert_eq!(shapes.iter().filter(|s| s.kind == Kind::Arrow).count(), 4);
    assert!(shapes.iter().all(|s| matches!(s.kind, Kind::Sticky | Kind::Arrow)));
    assert_eq!(get("root").color.as_deref(), Some("#EEB47E"));
    assert_eq!(get("b0l1").color, get("b0").color);
    assert_ne!(get("b0").color, get("b1").color);
}

#[test]
fn a_full_mindmap_keeps_every_sticky_apart_and_off_the_root_fill() {
    let branch =
        |i: usize| json!({"text": format!("B{i}"), "children": (0..6).map(|n| format!("L{n}")).collect::<Vec<_>>()});
    let map: Mindmap =
        serde_json::from_value(json!({"title": "Big", "branches": (0..8).map(branch).collect::<Vec<_>>()})).unwrap();
    let shapes = mindmap(map);
    assert_eq!(boxes(&shapes).len(), 1 + 8 + 48);
    assert!(apart(&shapes));
    let fills = shapes.iter().filter(|s| s.handle.as_deref() != Some("root")).filter_map(|s| s.color.as_deref());
    assert!(fills.clone().all(|c| c != ROOT_FILL && FILLS.contains(&c)));
}

#[test]
fn stickies_cycle_the_suite_fills_inside_their_frame() {
    let notes: Stickies =
        serde_json::from_value(json!({"title": "Ideas", "stickies": ["a", "b", "c", "d", "e", "f", "g"]})).unwrap();
    let shapes = stickies(notes);
    let colors: Vec<&str> = shapes[1..].iter().filter_map(|s| s.color.as_deref()).collect();
    assert_eq!(colors, [FILLS.as_slice(), &FILLS[..1]].concat());
    assert_eq!(shapes[0].kind, Kind::Frame);
}
