use quota_tray_core::{
    paint_tray_icon, render_mascot_png, render_mascot_rgba, render_tray_rgba, ProviderId,
    ProviderSnapshot, TrayState, MASCOT_GRID,
};
use time::OffsetDateTime;

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

fn snap(id: ProviderId, pct: Option<f64>, stale: bool) -> ProviderSnapshot {
    ProviderSnapshot {
        provider: id,
        fetched_at: OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap(),
        windows: vec![],
        headline_percent: pct,
        stale,
        error: if stale && pct.is_none() {
            Some("Waiting for the first reading".into())
        } else {
            None
        },
    }
}

#[test]
fn tray_icon_with_providers_differs_from_mascot_only() {
    let state = TrayState {
        providers: vec![
            snap(ProviderId::Claude, Some(70.0), false),
            snap(ProviderId::Cursor, Some(40.0), false),
        ],
        shared_mascot_fill: Some(70.0),
    };
    let with_ring = render_tray_rgba(&state, 32);
    let mascot_only = render_mascot_rgba(Some(70.0), 32);
    // Different canvas composition (inset + ring) — must not equal full-size mascot alone.
    assert_ne!(with_ring, mascot_only);
    let png = paint_tray_icon(&state);
    assert_eq!(&png[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
}

#[test]
fn muted_empty_segment_still_paints_pixels_when_headline_none() {
    let empty_headline = TrayState {
        providers: vec![snap(ProviderId::Copilot, None, false)],
        shared_mascot_fill: None,
    };
    let with_pct = TrayState {
        providers: vec![snap(ProviderId::Copilot, Some(90.0), false)],
        shared_mascot_fill: Some(90.0),
    };
    let a = render_tray_rgba(&empty_headline, 32);
    let b = render_tray_rgba(&with_pct, 32);
    assert_ne!(a, b, "muted empty segment must differ from filled segment");
    // Empty-headline glyph still has non-transparent ring pixels (not blank canvas).
    let opaque = a.chunks(4).filter(|px| px[3] > 0).count();
    assert!(opaque > 40, "expected muted track pixels, got {opaque}");
}
