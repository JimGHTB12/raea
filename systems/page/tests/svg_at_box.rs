// raea: SvgImageData::tree_at, a no-viewBox SVG's percentages against its box (H5)
fn at(src: &str, w: f32, h: f32) -> (f32, f32, f32, f32) {
    let svg = blitz_dom::node::SvgImageData::from_data(src.as_bytes(), &Default::default()).unwrap();
    let tree = svg.tree_at(w, h);
    let b = tree.root().abs_bounding_box();
    (tree.size().width(), tree.size().height(), b.width(), b.height())
}

#[test]
fn a_percent_size_without_a_viewbox_resolves_against_the_box() {
    let rect = r#"<rect width="100%" height="100%" fill="green"/>"#;
    let ns = r#"xmlns="http://www.w3.org/2000/svg""#;
    assert_eq!(at(&format!("<svg {ns}>{rect}</svg>"), 200.0, 150.0), (200.0, 150.0, 200.0, 150.0));
    assert_eq!(at(&format!("<svg {ns} width='50%' font-size=\"16\">{rect}</svg>"), 120.0, 80.0), (120.0, 80.0, 120.0, 80.0));
    // a fixed height stays; the width is the box's
    assert_eq!(at(&format!("<svg {ns} stroke-width=\"2\" height=\"40\">{rect}</svg>"), 120.0, 80.0), (120.0, 40.0, 120.0, 40.0));
    // a viewBox keeps its own size
    assert_eq!(at(&format!("<svg {ns} viewBox=\"0 0 10 10\">{rect}</svg>"), 120.0, 80.0).2, 10.0);
}
