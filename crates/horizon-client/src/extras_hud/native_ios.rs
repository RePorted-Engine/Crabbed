//! Native mobile overlays, independent of the Wasm mod runtime.
use super::*;
use crate::extras::FpsReading;
/// The FPS counter's text.
pub fn fps_text(r: FpsReading, lows: bool) -> String {
    if lows {
        format!("{} fps  min {}  1% {}", r.fps, r.min, r.low)
    } else {
        format!("{} fps", r.fps)
    }
}
pub fn draw_fps(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    corners: &mut Corners,
    corner: Corner,
    reading: FpsReading,
    lows: bool,
) {
    let text = fps_text(reading, lows);
    let w = HudBatch::text_width(atlas, &text) + 4.0;
    let (x, y) = corners.place(corner, w, 12.0);
    batch.fill(atlas, x, y, w, 12.0, BACKGROUND);
    batch.text(
        atlas,
        &text,
        x + 2.0,
        y + 2.0,
        rgba(255, 255, 255, 255),
        true,
    );
}

/// What the keystroke overlay shows.
pub struct Keystrokes {
    /// Forward, left, back, right: held, and the bound key's name.
    pub moves: [(bool, String); 4],
    pub jump: bool,
    pub attack: bool,
    pub use_: bool,
    /// Clicks per second of the left and right buttons, when shown.
    pub cps: Option<(usize, usize)>,
}
const KEY: f32 = 22.0;
const GAP: f32 = 2.0;
/// The overlay's size.
pub fn keystrokes_size() -> (f32, f32) {
    (3.0 * KEY + 2.0 * GAP, 3.0 * KEY + 3.0 * GAP + 10.0)
}
fn key_box(batch: &mut HudBatch, atlas: &HudAtlas, rect: [f32; 4], held: bool, lines: &[&str]) {
    let [x, y, w, h] = rect;
    let (fill, ink) = if held {
        (rgba(255, 255, 255, 176), rgba(20, 20, 20, 255))
    } else {
        (BACKGROUND, rgba(255, 255, 255, 255))
    };
    batch.fill(atlas, x, y, w, h, fill);
    let total = lines.len() as f32 * 9.0 - 1.0;
    for (i, line) in lines.iter().enumerate() {
        let tw = HudBatch::text_width(atlas, line);
        let tx = (x + (w - tw) / 2.0).round();
        let ty = (y + (h - total) / 2.0).round() + i as f32 * 9.0;
        batch.text(atlas, line, tx, ty, ink, !held);
    }
}
pub fn draw_keystrokes(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    corners: &mut Corners,
    corner: Corner,
    k: &Keystrokes,
) {
    let (w, h) = keystrokes_size();
    let (x, y) = corners.place(corner, w, h);
    let step = KEY + GAP;
    let [forward, left, back, right] = &k.moves;
    key_box(
        batch,
        atlas,
        [x + step, y, KEY, KEY],
        forward.0,
        &[&forward.1],
    );
    for (i, key) in [left, back, right].into_iter().enumerate() {
        key_box(
            batch,
            atlas,
            [x + i as f32 * step, y + step, KEY, KEY],
            key.0,
            &[&key.1],
        );
    }
    let mw = (w - GAP) / 2.0;
    let my = y + 2.0 * step;
    for (i, (held, label, cps)) in [
        (k.attack, "LMB", k.cps.map(|c| c.0)),
        (k.use_, "RMB", k.cps.map(|c| c.1)),
    ]
    .into_iter()
    .enumerate()
    {
        let rate = cps.map(|n| format!("{n} CPS"));
        let lines: Vec<&str> = std::iter::once(label).chain(rate.as_deref()).collect();
        key_box(
            batch,
            atlas,
            [x + i as f32 * (mw + GAP), my, mw, KEY],
            held,
            &lines,
        );
    }
    // The space bar: a line in a short key.
    let sy = my + step;
    key_box(batch, atlas, [x, sy, w, 10.0], k.jump, &[]);
    let ink = if k.jump {
        rgba(20, 20, 20, 255)
    } else {
        rgba(255, 255, 255, 255)
    };
    batch.fill(atlas, x + w / 2.0 - 12.0, sy + 4.0, 24.0, 2.0, ink);
}
