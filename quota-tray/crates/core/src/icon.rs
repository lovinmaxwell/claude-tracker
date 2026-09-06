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

/// Alert warning coral (#D94B34) for >= 75% usage.
pub const WARNING_CORAL: RgbaColor = RgbaColor {
    r: 0xD9,
    g: 0x4B,
    b: 0x34,
    a: 255,
};

/// Muted track for inactive pill capsules in the segmented ring.
pub const TRACK_MUTED: RgbaColor = RgbaColor {
    r: 0xDF,
    g: 0xD2,
    b: 0xC4,
    a: 70,
};

pub const TOTAL_PILLS: usize = 12;

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

fn dist_to_curved_capsule(
    px: f64,
    py: f64,
    cx: f64,
    cy: f64,
    r_mid: f64,
    start_theta: f64,
    end_theta: f64,
) -> f64 {
    let dx = px - cx;
    let dy = py - cy;
    let r = (dx * dx + dy * dy).sqrt();
    let theta = dy.atan2(dx);

    let arc_len = (end_theta - start_theta).rem_euclid(TAU);
    let delta = (theta - start_theta).rem_euclid(TAU);

    if delta <= arc_len {
        (r - r_mid).abs()
    } else {
        let past_end = (theta - end_theta).rem_euclid(TAU);
        let (ex, ey) = if past_end < std::f64::consts::PI {
            (cx + r_mid * end_theta.cos(), cy + r_mid * end_theta.sin())
        } else {
            (cx + r_mid * start_theta.cos(), cy + r_mid * start_theta.sin())
        };
        let edx = px - ex;
        let edy = py - ey;
        (edx * edx + edy * edy).sqrt()
    }
}

fn paint_pill(
    img: &mut RgbaImage,
    cx: f64,
    cy: f64,
    r_mid: f64,
    r_cap: f64,
    start_theta: f64,
    end_theta: f64,
    color: RgbaColor,
) {
    if color.a == 0 {
        return;
    }
    let r_max = r_mid + r_cap + 1.0;
    let r_min = (r_mid - r_cap - 1.0).max(0.0);
    let r_max2 = r_max * r_max;
    let r_min2 = r_min * r_min;

    let w = img.width();
    let h = img.height();

    for y in 0..h {
        let py = y as f64 + 0.5;
        let dy = py - cy;
        let dy2 = dy * dy;

        for x in 0..w {
            let px = x as f64 + 0.5;
            let dx = px - cx;
            let r2 = dx * dx + dy2;

            if r2 < r_min2 || r2 > r_max2 {
                continue;
            }

            let d = dist_to_curved_capsule(px, py, cx, cy, r_mid, start_theta, end_theta);
            if d >= r_cap + 0.5 {
                continue;
            }

            let cov = (r_cap + 0.5 - d).clamp(0.0, 1.0);
            let src_a = (color.a as f64 * cov).round() as u8;
            if src_a == 0 {
                continue;
            }

            let pixel = img.get_pixel_mut(x, y);
            if pixel[3] == 0 {
                *pixel = Rgba([color.r, color.g, color.b, src_a]);
            } else {
                let sa = src_a as f64 / 255.0;
                let da = pixel[3] as f64 / 255.0;
                let out_a = sa + da * (1.0 - sa);
                if out_a > 0.0 {
                    let out_r = ((color.r as f64 * sa + pixel[0] as f64 * da * (1.0 - sa)) / out_a).round() as u8;
                    let out_g = ((color.g as f64 * sa + pixel[1] as f64 * da * (1.0 - sa)) / out_a).round() as u8;
                    let out_b = ((color.b as f64 * sa + pixel[2] as f64 * da * (1.0 - sa)) / out_a).round() as u8;
                    *pixel = Rgba([out_r, out_g, out_b, (out_a * 255.0).round() as u8]);
                }
            }
        }
    }
}

/// Draw 12 discrete curved pill segments around the perimeter.
pub fn paint_provider_segments(img: &mut RgbaImage, providers: &[ProviderSnapshot]) {
    let cx = img.width() as f64 / 2.0;
    let cy = img.height() as f64 / 2.0;
    let scale = (img.width().min(img.height()) as f64) / 32.0;
    let r_mid = 13.4 * scale;
    let r_cap = 1.35 * scale;
    let alpha_half = 0.10_f64;

    let mut pill_colors = [TRACK_MUTED; TOTAL_PILLS];

    if providers.is_empty() {
        // Inactive track pills for all 12 slots when waiting or unconfigured
    } else if providers.len() == 1 {
        let snap = &providers[0];
        let chip = chip_color(&snap.provider);
        let unavailable = snap.stale || snap.error.is_some();
        if let Some(pct) = snap.headline_percent {
            let frac = (pct.clamp(0.0, 100.0) / 100.0).clamp(0.0, 1.0);
            let filled = (frac * TOTAL_PILLS as f64).round() as usize;
            for (i, slot) in pill_colors.iter_mut().enumerate() {
                if i < filled {
                    let c = if i >= 9 {
                        WARNING_CORAL
                    } else {
                        chip
                    };
                    *slot = if unavailable { c.with_alpha(140) } else { c };
                } else if unavailable && snap.headline_percent.is_none() {
                    *slot = TRACK_MUTED.with_alpha(40);
                }
            }
        } else if unavailable {
            for slot in pill_colors.iter_mut() {
                *slot = TRACK_MUTED.with_alpha(40);
            }
        }
    } else {
        let n = providers.len();
        for (p_idx, snap) in providers.iter().enumerate() {
            let start_pill = p_idx * TOTAL_PILLS / n;
            let end_pill = (p_idx + 1) * TOTAL_PILLS / n;
            let num_pills = end_pill - start_pill;
            let chip = chip_color(&snap.provider);
            let unavailable = snap.stale || snap.error.is_some();

            if let Some(pct) = snap.headline_percent {
                let frac = (pct.clamp(0.0, 100.0) / 100.0).clamp(0.0, 1.0);
                let filled = (frac * num_pills as f64).round() as usize;
                let warn_threshold = (num_pills * 3) / 4;
                for k in 0..num_pills {
                    let pill_i = start_pill + k;
                    if k < filled {
                        let c = if pct >= 75.0 && k >= warn_threshold {
                            WARNING_CORAL
                        } else {
                            chip
                        };
                        pill_colors[pill_i] = if unavailable { c.with_alpha(140) } else { c };
                    } else if unavailable && snap.headline_percent.is_none() {
                        pill_colors[pill_i] = TRACK_MUTED.with_alpha(40);
                    }
                }
            } else if unavailable {
                for k in 0..num_pills {
                    pill_colors[start_pill + k] = TRACK_MUTED.with_alpha(40);
                }
            }
        }
    }

    let step = TAU / TOTAL_PILLS as f64;
    for (i, &color) in pill_colors.iter().enumerate() {
        let theta_mid = -FRAC_PI_2 + (i as f64 + 0.5) * step;
        let start_theta = theta_mid - alpha_half;
        let end_theta = theta_mid + alpha_half;
        paint_pill(img, cx, cy, r_mid, r_cap, start_theta, end_theta, color);
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
