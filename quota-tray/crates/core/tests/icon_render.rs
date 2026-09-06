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

#[test]
fn pill_ring_differs_across_quota_thresholds() {
    let empty_state = TrayState {
        providers: vec![snap(ProviderId::Claude, Some(0.0), false)],
        shared_mascot_fill: Some(0.0),
    };
    let quarter_state = TrayState {
        providers: vec![snap(ProviderId::Claude, Some(25.0), false)],
        shared_mascot_fill: Some(25.0),
    };
    let warn_state = TrayState {
        providers: vec![snap(ProviderId::Claude, Some(75.0), false)],
        shared_mascot_fill: Some(75.0),
    };
    let full_state = TrayState {
        providers: vec![snap(ProviderId::Claude, Some(100.0), false)],
        shared_mascot_fill: Some(100.0),
    };

    let empty_buf = render_tray_rgba(&empty_state, 32);
    let quarter_buf = render_tray_rgba(&quarter_state, 32);
    let warn_buf = render_tray_rgba(&warn_state, 32);
    let full_buf = render_tray_rgba(&full_state, 32);

    assert_ne!(empty_buf, quarter_buf);
    assert_ne!(quarter_buf, warn_buf);
    assert_ne!(warn_buf, full_buf);

    // Canvas edges (0, 0) must be 100% transparent (no background container)
    assert_eq!(
        empty_buf[3], 0,
        "corner pixel alpha must be 0 (transparent)"
    );
    assert_eq!(
        quarter_buf[3], 0,
        "corner pixel alpha must be 0 (transparent)"
    );
    assert_eq!(warn_buf[3], 0, "corner pixel alpha must be 0 (transparent)");
}

#[test]
fn warning_threshold_activates_alert_color_on_upper_pills() {
    let normal_70 = TrayState {
        providers: vec![snap(ProviderId::Claude, Some(70.0), false)],
        shared_mascot_fill: Some(70.0),
    };
    let alert_80 = TrayState {
        providers: vec![snap(ProviderId::Claude, Some(80.0), false)],
        shared_mascot_fill: Some(80.0),
    };

    let png_70 = paint_tray_icon(&normal_70);
    let png_80 = paint_tray_icon(&alert_80);

    assert_ne!(png_70, png_80);

    let rgba_80 = render_tray_rgba(&alert_80, 32);
    // Alert coral is #D94B34 (217, 75, 52). Verify presence in pixels.
    let has_alert_coral = rgba_80
        .chunks(4)
        .any(|px| px[0] == 0xD9 && px[1] == 0x4B && px[2] == 0x34);
    assert!(
        has_alert_coral,
        "alert state must contain WARNING_CORAL (#D94B34) pixels"
    );
}

#[test]
fn generates_sample_png_assets() {
    let scratch_dir = std::path::Path::new("/Users/lovinmaxwell/.gemini/antigravity-ide/brain/7da05395-a586-44ff-9545-331e7cf38762/scratch");
    if !scratch_dir.exists() {
        return;
    }
    for &(name, pct) in &[
        ("tray_0pct.png", Some(0.0)),
        ("tray_25pct.png", Some(25.0)),
        ("tray_50pct.png", Some(50.0)),
        ("tray_75pct.png", Some(75.0)),
        ("tray_100pct.png", Some(100.0)),
    ] {
        let state = TrayState {
            providers: vec![snap(ProviderId::Claude, pct, false)],
            shared_mascot_fill: pct,
        };
        let png = paint_tray_icon(&state);
        let _ = std::fs::write(scratch_dir.join(name), png);
    }
}
