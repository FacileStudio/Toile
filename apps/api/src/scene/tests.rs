use yrs::{Map, Transact};

use super::change::Change;
use super::input::{Kind, NewShape};
use super::plan::plan;
use super::read::{self, Shape};
use super::{text, write};

fn counter() -> impl FnMut() -> String {
    let mut n = 0;
    move || {
        n += 1;
        format!("n{n}")
    }
}

fn sticky(id: &str, x: f64) -> Shape {
    Shape { id: id.into(), kind: "sticky".into(), x, w: 200.0, h: 200.0, z: "V".into(), ..Shape::default() }
}

#[test]
fn new_shapes_land_right_of_the_content_and_children_inside_their_new_frame() {
    let mut frame = NewShape::new(Kind::Frame, "Ideas");
    frame.handle = Some("f".into());
    let mut child = NewShape::new(Kind::Sticky, "one");
    child.frame = Some("f".into());
    let mut arrow = NewShape::new(Kind::Arrow, "");
    (arrow.from, arrow.to) = (Some("old".into()), Some("f".into()));
    let out = plan(&[sticky("old", 0.0)], vec![arrow, child, frame], counter()).unwrap();
    let ids: Vec<(&str, &str)> = out.iter().map(|p| (p.placed.id.as_str(), p.placed.kind.name())).collect();
    assert_eq!(ids, [("n1", "frame"), ("n2", "sticky"), ("n3", "arrow")]);
    assert_eq!((out[0].placed.x, out[0].placed.w, out[0].placed.h), (360.0, 280.0, 320.0));
    assert_eq!((out[1].placed.x, out[1].placed.y, out[1].parent.as_deref()), (400.0, 80.0, Some("n1")));
    assert_eq!((out[2].from.as_deref(), out[2].to.as_deref(), out[2].end), (Some("old"), Some("n1"), (400.0, 60.0)));
    assert_eq!([&out[0].z, &out[1].z, &out[2].z], ["W", "X", "Y"]);
}

#[test]
fn unknown_refs_and_reused_refs_are_refused() {
    let mut arrow = NewShape::new(Kind::Arrow, "");
    arrow.from = Some("ghost".into());
    assert!(plan(&[], vec![arrow], counter()).unwrap_err().contains("ghost"));
    let mut a = NewShape::new(Kind::Sticky, "a");
    a.handle = Some("x".into());
    let b = NewShape { handle: Some("x".into()), ..a.clone() };
    assert!(plan(&[], vec![a, b], counter()).unwrap_err().contains("twice"));
}

#[test]
fn planned_shapes_pass_the_schema_and_read_back_then_change_and_go() {
    let doc = crate::modules::board::new_doc();
    let shapes = doc.get_or_insert_map(crate::modules::board::SHAPES);
    let mut frame = NewShape::new(Kind::Frame, "F");
    frame.handle = Some("f".into());
    let child =
        NewShape { frame: Some("f".into()), color: Some("#fde68a".into()), ..NewShape::new(Kind::Sticky, "hi") };
    write::insert(&mut doc.transact_mut(), &shapes, &plan(&[], vec![frame, child], counter()).unwrap());
    crate::modules::board::schema::check_all(&doc).unwrap();
    let change = Change {
        id: "n1".into(),
        text: Some("Renamed".into()),
        x: Some(100.0),
        y: None,
        w: None,
        h: None,
        color: None,
        frame: None,
    };
    super::change::apply(&mut doc.transact_mut(), &shapes, &[change]).unwrap();
    let all = read::all(&doc.transact(), &shapes);
    assert_eq!(
        text::render(&all),
        "frame n1 \"Renamed\" at 100,0 280x320\n  sticky n2 at 140,80 200x200 #fde68a: \"hi\"\n"
    );
    let gone = write::remove(&mut doc.transact_mut(), &shapes, &["n1".into()]).unwrap();
    assert_eq!((gone, shapes.len(&doc.transact())), (vec!["n1".to_owned(), "n2".to_owned()], 0));
}
