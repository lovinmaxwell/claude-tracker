use crate::types::{ProviderId, ProviderSnapshot, TrayState};
use image::{ImageBuffer, ImageError, Rgba, RgbaImage};
use std::f64::consts::{FRAC_PI_2, TAU};
use std::io::Cursor;
use thiserror::Error;

pub const MASCOT_GRID: usize = 16;

/// B = body, E = eye, . = empty — from Claude Tracker PixelMascot.
pub const MASCOT_ART: [&str; 16] = [
    "................",
    "................",
    "...BBBBBBBBBB...",
    "...BBBBBBBBBB...",
    "...BEEBBBBEEB...",
    "...BEEBBBBEEB...",
    ".BBBBBBBBBBBBBB.",
    ".BBBBBBBBBBBBBB.",
    ".BBBBBBBBBBBBBB.",
    "...BBBBBBBBBB...",
    "...BBBBBBBBBB...",
    "...BB.BB.BB.BB..",
    "...BB.BB.BB.BB..",
    "...BB.BB.BB.BB..",
    "................",
    "................",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RgbaColor {
    pub const fn to_rgba(self) -> Rgba<u8> {
        Rgba([self.r, self.g, self.b, self.a])
    }

    pub fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let h = hex.trim_start_matches('#');
        if h.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&h[0..2], 16).ok()?;
        let g = u8::from_str_radix(&h[2..4], 16).ok()?;
        let b = u8::from_str_radix(&h[4..6], 16).ok()?;
        Some(Self { r, g, b, a: 255 })
    }
}

/// Terracotta spent fill (#C96442).
pub const TERRACOTTA: RgbaColor = RgbaColor {
    r: 0xC9,
    g: 0x64,
    b: 0x42,
    a: 255,
};

/// Pale cream unspent body.
pub const CREAM: RgbaColor = RgbaColor {
    r: 0xF5,
    g: 0xED,
    b: 0xE6,
    a: 255,
};

const EYE: RgbaColor = RgbaColor {
    r: 0x2C,
    g: 0x2C,
    b: 0x2C,
    a: 255,
};

const TRANSPARENT: RgbaColor = RgbaColor {
    r: 0,
    g: 0,
    b: 0,
    a: 0,
};

/// Muted track for empty / unknown segment arc.
const SEGMENT_MUTED: RgbaColor = RgbaColor {
    r: 0xD4,
    g: 0xC4,
    b: 0xB4,
    a: 200,
};

#[derive(Debug, Error)]
pub enum IconError {
    #[error("png encode: {0}")]
    Encode(String),
}

fn body_row_bounds() -> (usize, usize) {
    let mut min = MASCOT_GRID;
    let mut max = 0;
    for (y, line) in MASCOT_ART.iter().enumerate() {
        if line.chars().any(|c| c == 'B' || c == 'E') {
            min = min.min(y);
            max = max.max(y);
        }
    }
    (min, max)
}

fn chip_color(id: &ProviderId) -> RgbaColor {
    RgbaColor::from_hex(id.chip_hex()).unwrap_or(SEGMENT_MUTED)
}

/// Fill-from-feet: `percent_used` 0–100. `None` → pale empty body (honest unknown).
pub fn render_mascot_rgba(percent_used: Option<f64>, pixel_size: u32) -> Vec<u8> {
    assert!(pixel_size > 0);
    assert_eq!(
        pixel_size as usize % MASCOT_GRID,
        0,
        "pixel_size must be a multiple of {MASCOT_GRID}"
    );
    let cell = (pixel_size as usize) / MASCOT_GRID;
    let fill_pct = percent_used.map(|p| p.clamp(0.0, 100.0)).unwrap_or(0.0);
    // Note: None and Some(0.0) both yield cream-only body; UI distinguishes via
    // stale/muted chrome, not by inventing a fake live zero glyph.
    let (body_min, body_max) = body_row_bounds();
    let body_h = (body_max - body_min + 1) as f64;
    let fill_rows = ((fill_pct / 100.0) * body_h).round() as usize;

    let mut img: RgbaImage =
        ImageBuffer::from_pixel(pixel_size, pixel_size, TRANSPARENT.to_rgba());

    for (y, line) in MASCOT_ART.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            if ch == '.' {
                continue;
            }
            let color = if ch == 'E' {
                EYE
            } else {
                // Fill from feet (bottom of body bounds upward).
                let row_from_feet = body_max.saturating_sub(y);
                if row_from_feet < fill_rows {
                    TERRACOTTA
                } else {
                    CREAM
                }
            };
            for dy in 0..cell {
                for dx in 0..cell {
                    let px = (x * cell + dx) as u32;
                    let py = (y * cell + dy) as u32;
                    img.put_pixel(px, py, color.to_rgba());
                }
            }
        }
    }

    img.into_raw()
}

pub fn render_mascot_png(
    percent_used: Option<f64>,
    pixel_size: u32,
) -> Result<Vec<u8>, IconError> {
    let raw = render_mascot_rgba(percent_used, pixel_size);
    let img: RgbaImage = ImageBuffer::from_raw(pixel_size, pixel_size, raw)
        .ok_or_else(|| IconError::Encode("buffer size mismatch".into()))?;
    let mut cursor = Cursor::new(Vec::new());
    img.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e: ImageError| IconError::Encode(e.to_string()))?;
    Ok(cursor.into_inner())
}

fn blit_centered(dst: &mut RgbaImage, src: &RgbaImage) {
    let ox = (dst.width().saturating_sub(src.width())) / 2;
    let oy = (dst.height().saturating_sub(src.height())) / 2;
    for y in 0..src.height() {
        for x in 0..src.width() {
            let p = *src.get_pixel(x, y);
            if p[3] == 0 {
                continue;
            }
            dst.put_pixel(ox + x, oy + y, p);
        }
    }
}

fn angle_in_arc(theta: f64, start: f64, end: f64) -> bool {
    let t = theta.rem_euclid(TAU);
    let s = start.rem_euclid(TAU);
    let e = end.rem_euclid(TAU);
    if (e - s).abs() < f64::EPSILON {
        return false;
    }
    if s <= e {
        t >= s && t <= e
    } else {
        // wrap across 0
        t >= s || t <= e
    }
}

fn paint_ring_pixel(
    img: &mut RgbaImage,
    cx: f64,
    cy: f64,
    inner_r: f64,
    outer_r: f64,
    start: f64,
    end: f64,
    color: RgbaColor,
    dashed: bool,
) {
    let w = img.width();
    let h = img.height();
    for y in 0..h {
        for x in 0..w {
            let dx = x as f64 + 0.5 - cx;
            let dy = y as f64 + 0.5 - cy;
            let r = (dx * dx + dy * dy).sqrt();
            if r < inner_r || r > outer_r {
                continue;
            }
            let theta = dy.atan2(dx);
            if !angle_in_arc(theta, start, end) {
                continue;
            }
            if dashed {
                // ~8 dashes around a full circle
                let dash_bin = ((theta.rem_euclid(TAU) / TAU) * 16.0).floor() as i32;
                if dash_bin % 2 != 0 {
                    continue;
                }
            }
            img.put_pixel(x, y, color.to_rgba());
        }
    }
}

/// Draw one arc segment per enabled provider (chip colors; muted when no %).
pub fn paint_provider_segments(img: &mut RgbaImage, providers: &[ProviderSnapshot]) {
    let n = providers.len();
    if n == 0 {
        return;
    }
    let cx = img.width() as f64 / 2.0;
    let cy = img.height() as f64 / 2.0;
    let outer_r = (img.width().min(img.height()) as f64) / 2.0 - 0.5;
    let inner_r = outer_r - 3.25;
    let gap = 0.12_f64;
    let sweep = TAU / n as f64;

    for (i, snap) in providers.iter().enumerate() {
        let start = -FRAC_PI_2 + i as f64 * sweep + gap / 2.0;
        let end = start + sweep - gap;
        let chip = chip_color(&snap.provider);
        let unavailable = snap.stale || snap.error.is_some();
        let muted_empty = snap.headline_percent.is_none();

        // Track (always present for enabled providers).
        let track = if muted_empty || unavailable {
            SEGMENT_MUTED
        } else {
            chip.with_alpha(90)
        };
        paint_ring_pixel(
            img,
            cx,
            cy,
            inner_r,
            outer_r,
            start,
            end,
            track,
            unavailable && muted_empty,
        );

        if let Some(pct) = snap.headline_percent {
            let frac = (pct.clamp(0.0, 100.0) / 100.0).clamp(0.0, 1.0);
            if frac <= 0.0 {
                continue;
            }
            let fill_end = start + (end - start) * frac;
            let fill = if unavailable {
                chip.with_alpha(140)
            } else {
                chip
            };
            paint_ring_pixel(
                img,
                cx,
                cy,
                inner_r,
                outer_r,
                start,
                fill_end,
                fill,
                unavailable,
            );
        }
    }
}

/// RGBA tray glyph: cream/terracotta mascot + multi-segment ring.
pub fn render_tray_rgba(state: &TrayState, pixel_size: u32) -> Vec<u8> {
    assert!(pixel_size >= 16);
    let mut canvas: RgbaImage =
        ImageBuffer::from_pixel(pixel_size, pixel_size, TRANSPARENT.to_rgba());

    // Inset mascot so the segment ring sits in the outer band.
    let mascot_size = (pixel_size * 3 / 4).max(16);
    let mascot_size = mascot_size - (mascot_size % MASCOT_GRID as u32);
    let mascot_size = if mascot_size == 0 {
        MASCOT_GRID as u32
    } else {
        mascot_size
    };
    let mascot_raw = render_mascot_rgba(state.shared_mascot_fill, mascot_size);
    let mascot: RgbaImage = ImageBuffer::from_raw(mascot_size, mascot_size, mascot_raw)
        .expect("mascot buffer size");
    blit_centered(&mut canvas, &mascot);
    paint_provider_segments(&mut canvas, &state.providers);
    canvas.into_raw()
}

/// Tray-sized mascot + segment PNG from [`TrayState`].
pub fn paint_tray_icon(state: &TrayState) -> Vec<u8> {
    const TRAY_PIXEL_SIZE: u32 = 32;
    let raw = render_tray_rgba(state, TRAY_PIXEL_SIZE);
    let img: RgbaImage = ImageBuffer::from_raw(TRAY_PIXEL_SIZE, TRAY_PIXEL_SIZE, raw)
        .expect("tray buffer size");
    let mut cursor = Cursor::new(Vec::new());
    if img
        .write_to(&mut cursor, image::ImageFormat::Png)
        .is_ok()
    {
        return cursor.into_inner();
    }
    render_mascot_png(None, TRAY_PIXEL_SIZE).expect("encode empty mascot png")
}
