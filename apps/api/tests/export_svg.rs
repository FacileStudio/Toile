mod common;

use api::modules::export::model::{Kind, Shape};
use common::shapes::{bind, draw, shape, snapshot};

fn boxes() -> Vec<(&'static str, Vec<Shape>)> {
    let mut sticky = shape("s1", Kind::Sticky, [0.0, 0.0, 200.0, 200.0]);
    sticky.text = "Idées pour la rétro & <suite>".into();
    let mut text = shape("t1", Kind::Text, [0.0, 0.0, 240.0, 60.0]);
    (text.text, text.style.align, text.style.size) = ("Titre\nsous-titre".into(), Some("left".into()), Some(24.0));
    let mut rect = shape("r1", Kind::Rect, [10.0, 10.0, 160.0, 90.0]);
    (rect.style.fill, rect.style.stroke, rect.text) = (Some("#dbeafe".into()), Some("#1d4ed8".into()), "Box".into());
    rect.rot = std::f64::consts::FRAC_PI_8;
    let mut ellipse = shape("e1", Kind::Ellipse, [0.0, 0.0, 120.0, 80.0]);
    (ellipse.style.stroke, ellipse.style.width) = (Some("#16a34a".into()), Some(4.0));
    let mut diamond = shape("d1", Kind::Diamond, [0.0, 0.0, 100.0, 100.0]);
    (diamond.style.fill, diamond.text, diamond.style.font) =
        (Some("url(#evil)".into()), "Go?".into(), Some("mono".into()));
    vec![
        ("sticky", vec![sticky]),
        ("text", vec![text]),
        ("rect", vec![rect]),
        ("ellipse", vec![ellipse]),
        ("diamond", vec![diamond]),
    ]
}

fn connectors() -> Vec<(&'static str, Vec<Shape>)> {
    let mut line = shape("l1", Kind::Line, [0.0, 0.0, 0.0, 0.0]);
    line.points = vec![0.0, 0.0, 80.0, 40.0, 160.0, 0.0];
    let a = shape("a", Kind::Rect, [0.0, 0.0, 100.0, 60.0]);
    let b = shape("b", Kind::Ellipse, [300.0, 200.0, 100.0, 60.0]);
    let mut arrow = shape("ar", Kind::Arrow, [0.0, 0.0, 0.0, 0.0]);
    arrow.points = vec![0.0, 0.0, 10.0, 10.0];
    (arrow.from, arrow.to, arrow.head) = (bind("a", [1.0, 0.5]), bind("b", [0.0, 0.5]), Some("both".into()));
    vec![("line", vec![line]), ("arrow", vec![a, b, arrow])]
}

fn others() -> Vec<(&'static str, Vec<Shape>)> {
    let mut ink = shape("i1", Kind::Ink, [50.0, 50.0, 0.0, 0.0]);
    ink.points = (0..12).flat_map(|i| [f64::from(i) * 10.0, (f64::from(i) * 0.7).sin() * 20.0, 0.5]).collect();
    let mut image = shape("im", Kind::Image, [0.0, 0.0, 160.0, 120.0]);
    image.asset = "abc-123/../../etc".into();
    let mut card = shape("c1", Kind::Card, [0.0, 0.0, 280.0, 110.0]);
    (card.url, card.title, card.card_kind) = ("https://docs.facile.studio/d/1".into(), "Plan v1".into(), "docs".into());
    let mut frame = shape("f1", Kind::Frame, [0.0, 0.0, 300.0, 200.0]);
    frame.name = "Sprint 12".into();
    let mut child = shape("f1c", Kind::Sticky, [200.0, 150.0, 200.0, 120.0]);
    (child.parent, child.text) = (Some("f1".into()), "Débordement".into());
    vec![("ink", vec![ink]), ("image", vec![image]), ("card", vec![card]), ("frame", vec![frame, child])]
}

#[test]
fn every_shape_type_matches_its_snapshot() {
    for (name, shapes) in boxes().into_iter().chain(connectors()).chain(others()) {
        let drawing = draw(&shapes, None).unwrap();
        assert!(drawing.svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""), "{name}");
        snapshot(name, &drawing.svg);
    }
}

#[test]
fn unsafe_values_never_reach_the_svg() {
    let all: String = boxes().into_iter().chain(others()).map(|(_, s)| draw(&s, None).unwrap().svg).collect();
    assert!(!all.contains("url(#evil)") && !all.contains("../"));
    assert!(all.contains("Idées pour la rétro</tspan>") && all.contains("&amp; &lt;suite&gt;"));
}

#[test]
fn shapes_stack_by_fractional_index_not_by_id() {
    let below = shape("zz", Kind::Ellipse, [0.0, 0.0, 10.0, 10.0]);
    let above = shape("aa", Kind::Rect, [0.0, 0.0, 10.0, 10.0]);
    let svg = draw(&[below, above], None).unwrap().svg;
    assert!(svg.find("<ellipse").unwrap() < svg.find("<rect").unwrap());
}

#[test]
fn a_frame_filter_draws_only_that_frame() {
    let (_, mut shapes) = others().pop().unwrap();
    shapes.push(shape("out", Kind::Diamond, [900.0, 900.0, 50.0, 50.0]));
    let drawing = draw(&shapes, Some("f1")).unwrap();
    assert!(drawing.svg.contains("Sprint 12") && !drawing.svg.contains("<polygon"));
    assert!(drawing.svg.contains("clip-path=\"url(#frame-1)\""));
    assert_eq!((drawing.width, drawing.height), (368.0, 296.0));
    assert!(draw(&shapes, Some("nope")).is_none() && draw(&shapes, Some("out")).is_none());
}
