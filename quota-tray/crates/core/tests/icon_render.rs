use quota_tray_core::{render_mascot_png, render_mascot_rgba, MASCOT_GRID};

#[test]
fn rgba_buffer_size_matches_pixel_size() {
    let size = 32;
    let buf = render_mascot_rgba(Some(50.0), size);
    assert_eq!(buf.len(), (size * size * 4) as usize);
}

#[test]
fn none_percent_is_pale_not_full_terracotta() {
    let size = 16;
    let empty = render_mascot_rgba(None, size);
    let full = render_mascot_rgba(Some(100.0), size);
    assert_ne!(empty, full, "unknown must not render as fully spent");
}

#[test]
fn higher_percent_changes_pixels_vs_zero() {
    let size = 16;
    let zero = render_mascot_rgba(Some(0.0), size);
    let high = render_mascot_rgba(Some(80.0), size);
    assert_ne!(zero, high);
}

#[test]
fn png_has_signature_and_decodes_to_grid_multiple() {
    let png = render_mascot_png(Some(42.0), 32).expect("png");
    assert_eq!(&png[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    let img = image::load_from_memory(&png).expect("decode").to_rgba8();
    assert_eq!(img.width(), 32);
    assert_eq!(img.height(), 32);
    assert_eq!(MASCOT_GRID, 16);
}
