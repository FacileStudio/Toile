mod common;

use api::modules::export::model::Kind;
use api::modules::export::raster::{self, MAX_SIDE};
use common::shapes::{draw, shape};

fn png_size(png: &[u8]) -> (u32, u32) {
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    let word = |at: usize| u32::from_be_bytes(png[at..at + 4].try_into().unwrap());
    (word(16), word(20))
}

#[test]
fn a_png_has_the_drawing_size_times_the_scale() {
    let mut sticky = shape("s", Kind::Sticky, [0.0, 0.0, 200.0, 100.0]);
    sticky.text = "Bonjour".into();
    let drawing = draw(&[sticky], None).unwrap();
    let png = raster::png(&drawing, 2.0, MAX_SIDE).unwrap();
    assert_eq!(png_size(&png), (2 * 268, 2 * 168));
    assert!(png.len() > 1000);
}

#[test]
fn a_huge_board_is_capped_at_the_longest_side() {
    let far = shape("far", Kind::Rect, [0.0, 0.0, 100_000.0, 10.0]);
    let drawing = draw(&[far], None).unwrap();
    let (w, h) = png_size(&raster::png(&drawing, 2.0, 480).unwrap());
    assert_eq!(w, 480);
    assert!((1..10).contains(&h));
}

#[test]
fn a_pdf_page_has_the_drawing_size_in_points() {
    let rect = shape("r", Kind::Rect, [0.0, 0.0, 400.0, 300.0]);
    let drawing = draw(&[rect], None).unwrap();
    let pdf = raster::pdf(&drawing).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
    let text = String::from_utf8_lossy(&pdf);
    assert!(text.contains("/MediaBox [0 0 468 368]"), "{}", &text[..text.len().min(600)]);
}
