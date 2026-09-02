use image::{ImageBuffer, ImageError, Rgba, RgbaImage};
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
