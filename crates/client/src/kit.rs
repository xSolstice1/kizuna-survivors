//! UI kit: fonts, theme colours, rounded panels, buttons, bars and icon slots.
//! Night-shrine look: deep indigo lacquer panels, gold trim, vermilion and sakura accents.

use std::cell::RefCell;

use macroquad::prelude::*;

pub const INK: Color = Color::new(0.96, 0.94, 1.0, 1.0);
pub const DIM: Color = Color::new(0.74, 0.71, 0.86, 1.0);
pub const FAINT: Color = Color::new(0.74, 0.71, 0.86, 0.55);
pub const GOLD: Color = Color::new(1.0, 0.82, 0.42, 1.0);
pub const VERMILION: Color = Color::new(0.93, 0.33, 0.33, 1.0);
pub const SAKURA: Color = Color::new(1.0, 0.45, 0.65, 1.0);
pub const JADE: Color = Color::new(0.45, 0.95, 0.65, 1.0);
pub const SKY: Color = Color::new(0.35, 0.8, 1.0, 1.0);
pub const PANEL: Color = Color::new(0.075, 0.06, 0.14, 0.93);
pub const PANEL_HI: Color = Color::new(0.14, 0.10, 0.24, 0.97);
pub const EDGE: Color = Color::new(1.0, 0.82, 0.42, 0.35);

// ───────────────────────── Virtual resolution ─────────────────────────
// All UI is laid out for a 1280x720 canvas. Smaller screens (phones) scale it down to fit;
// UI modules import these instead of macroquad's, so their maths stays in virtual pixels.

const UI_W: f32 = 1280.0;
const UI_H: f32 = 720.0;

/// Physical pixels per virtual UI pixel. Rendering is high-DPI, so this is above 1 on
/// dense screens (phones, retina laptops) and the UI is drawn at full resolution.
pub fn ui_scale() -> f32 {
    (macroquad::window::screen_width() / UI_W).min(macroquad::window::screen_height() / UI_H)
}

pub fn screen_width() -> f32 {
    macroquad::window::screen_width() / ui_scale()
}

pub fn screen_height() -> f32 {
    macroquad::window::screen_height() / ui_scale()
}

pub fn mouse_position() -> (f32, f32) {
    let (x, y) = macroquad::input::mouse_position();
    let s = ui_scale();
    (x / s, y / s)
}

/// Physical screen position -> virtual UI position.
pub fn to_ui(p: Vec2) -> Vec2 {
    p / ui_scale()
}

/// Sets a camera so subsequent drawing uses virtual UI coordinates.
pub fn begin_ui() {
    let (w, h) = (screen_width(), screen_height());
    set_camera(&Camera2D { target: vec2(w / 2.0, h / 2.0), zoom: vec2(2.0 / w, 2.0 / h), ..Default::default() });
}

/// Smooth circles: macroquad's default uses 20 segments, which shows on big circles.
pub fn draw_circle(x: f32, y: f32, r: f32, c: Color) {
    macroquad::shapes::draw_poly(x, y, circle_sides(r), r, 0.0, c);
}

pub fn draw_circle_lines(x: f32, y: f32, r: f32, thickness: f32, c: Color) {
    macroquad::shapes::draw_poly_lines(x, y, circle_sides(r), r, 0.0, thickness, c);
}

fn circle_sides(r: f32) -> u8 {
    (r * 0.25).clamp(8.0, 40.0) as u8
}

/// Clips drawing to a rectangle in virtual UI coordinates (None = no clip).
pub fn clip(r: Option<Rect>) {
    let k = ui_scale() * screen_dpi_scale();
    let mut gl = unsafe { get_internal_gl() };
    gl.flush();
    gl.quad_gl.scissor(r.map(|r| ((r.x * k) as i32, (r.y * k) as i32, (r.w * k).ceil() as i32, (r.h * k).ceil() as i32)));
}

thread_local! {
    static FONTS: RefCell<Option<(Font, Font)>> = const { RefCell::new(None) };
}

/// Loads the UI fonts (Nunito Bold for body text, ExtraBold for titles). Call once at startup.
pub fn init() {
    let body = load_ttf_font_from_bytes(include_bytes!("../../../assets/fonts/Nunito-Bold.ttf")).expect("body font");
    let title = load_ttf_font_from_bytes(include_bytes!("../../../assets/fonts/Nunito-ExtraBold.ttf")).expect("title font");
    FONTS.with(|f| *f.borrow_mut() = Some((body, title)));
}

fn with_font<R>(size: f32, f: impl FnOnce(Option<&Font>) -> R) -> R {
    FONTS.with(|fonts| {
        let fonts = fonts.borrow();
        let font = fonts.as_ref().map(|(b, t)| if size >= 30.0 { t } else { b });
        f(font)
    })
}

/// Text is rasterised at the physical pixel size (then scaled back into UI units), so it is
/// sharp at any UI scale. Sizes are quantised to keep the glyph atlas small.
fn raster(size: f32) -> (u16, f32) {
    // UI units -> CSS pixels (ui_scale) -> device pixels (dpi).
    // Coarse steps: every new raster size adds glyphs to the shared atlas, and each addition
    // re-uploads the whole atlas to the GPU, so keep the set of sizes small.
    let want = size * ui_scale() * screen_dpi_scale();
    let step = if want < 24.0 { 2.0 } else if want < 48.0 { 4.0 } else { 8.0 };
    let phys = (want / step).round().max(2.0) * step;
    (phys as u16, size / phys)
}

/// Draws text with its baseline at `y` (same convention as macroquad's draw_text).
pub fn text(s: &str, x: f32, y: f32, size: f32, color: Color) {
    let (fs, sc) = raster(size);
    with_font(size, |font| {
        draw_text_ex(s, x, y, TextParams { font, font_size: fs, font_scale: sc, color, ..Default::default() });
    });
}

pub fn measure(s: &str, size: u16) -> TextDimensions {
    let (fs, sc) = raster(size as f32);
    with_font(size as f32, |font| measure_text(s, font, fs, sc))
}

/// Largest size up to `max` at which `s` fits in `width` (never below 9).
pub fn fit(s: &str, width: f32, max: f32) -> f32 {
    let w = measure(s, max as u16).width;
    if w <= width {
        max
    } else {
        (max * width / w).floor().max(9.0)
    }
}

/// Text shrunk to fit `width`.
pub fn text_fit(s: &str, x: f32, y: f32, width: f32, size: f32, color: Color) {
    text(s, x, y, fit(s, width, size), color);
}

/// Text centred on `cx`, shrunk to fit `width`.
pub fn text_fit_centered(s: &str, cx: f32, y: f32, width: f32, size: f32, color: Color) {
    let sz = fit(s, width, size);
    let d = measure(s, sz as u16);
    text(s, cx - d.width / 2.0, y, sz, color);
}

/// Word-wraps `s` into lines no wider than `width` at `size`.
pub fn wrap(s: &str, width: f32, size: u16) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in s.split_whitespace() {
        let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if measure(&candidate, size).width > width && !line.is_empty() {
            out.push(std::mem::take(&mut line));
            line = word.to_string();
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

/// Text with a soft drop shadow (for titles over busy backgrounds).
pub fn text_shadow(s: &str, x: f32, y: f32, size: f32, color: Color) {
    text(s, x + 2.0, y + 2.0, size, Color::new(0.0, 0.0, 0.0, 0.55 * color.a));
    text(s, x, y, size, color);
}

pub fn centered(s: &str, cx: f32, y: f32, size: f32, color: Color) {
    let d = measure(s, size as u16);
    text_shadow(s, cx - d.width / 2.0, y, size, color);
}

// ───────────────────────── Shapes ─────────────────────────

pub fn rounded(x: f32, y: f32, w: f32, h: f32, r: f32, c: Color) {
    // One triangle fan around the perimeter, so translucent fills don't double-blend at overlaps.
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    let centre = vec2(x + w / 2.0, y + h / 2.0);
    let seg = 5;
    let mut pts: Vec<Vec2> = Vec::with_capacity(4 * (seg + 1));
    for (cx, cy, a0) in [(x + w - r, y + r, 270.0f32), (x + w - r, y + h - r, 0.0), (x + r, y + h - r, 90.0), (x + r, y + r, 180.0)] {
        for k in 0..=seg {
            let t = (a0 + 90.0 * k as f32 / seg as f32).to_radians();
            pts.push(vec2(cx + t.cos() * r, cy + t.sin() * r));
        }
    }
    for i in 0..pts.len() {
        draw_triangle(centre, pts[i], pts[(i + 1) % pts.len()], c);
    }
}

pub fn rounded_outline(x: f32, y: f32, w: f32, h: f32, r: f32, thick: f32, c: Color) {
    let r = r.min(w / 2.0).min(h / 2.0);
    draw_line(x + r, y, x + w - r, y, thick, c);
    draw_line(x + r, y + h, x + w - r, y + h, thick, c);
    draw_line(x, y + r, x, y + h - r, thick, c);
    draw_line(x + w, y + r, x + w, y + h - r, thick, c);
    let seg = 6;
    for (cx, cy, a0) in [(x + r, y + r, 180.0f32), (x + w - r, y + r, 270.0), (x + w - r, y + h - r, 0.0), (x + r, y + h - r, 90.0)] {
        for k in 0..seg {
            let t0 = (a0 + 90.0 * k as f32 / seg as f32).to_radians();
            let t1 = (a0 + 90.0 * (k + 1) as f32 / seg as f32).to_radians();
            draw_line(cx + t0.cos() * r, cy + t0.sin() * r, cx + t1.cos() * r, cy + t1.sin() * r, thick, c);
        }
    }
}

/// A lacquered panel: dark fill, a faint top sheen and gold trim.
pub fn panel(x: f32, y: f32, w: f32, h: f32) {
    panel_ex(x, y, w, h, PANEL, EDGE);
}

pub fn panel_ex(x: f32, y: f32, w: f32, h: f32, fill: Color, edge: Color) {
    rounded(x + 3.0, y + 5.0, w, h, 12.0, Color::new(0.0, 0.0, 0.0, 0.35));
    rounded(x, y, w, h, 12.0, fill);
    rounded(x + 2.0, y + 2.0, w - 4.0, (h * 0.35).min(40.0), 10.0, Color::new(1.0, 1.0, 1.0, 0.018));
    rounded_outline(x, y, w, h, 12.0, 1.5, edge);
}

/// Panel title strip with a small gold diamond.
pub fn panel_title(s: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_poly(x + 6.0, y - size * 0.32, 4, 5.0, 45.0, GOLD);
    text_shadow(s, x + 18.0, y, size, color);
}

/// Horizontal bar with rounded ends; `frac` in 0..=1.
pub fn bar(x: f32, y: f32, w: f32, h: f32, frac: f32, fill: Color, back: Color) {
    rounded(x, y, w, h, h / 2.0, back);
    let fw = (w * frac.clamp(0.0, 1.0)).max(if frac > 0.0 { h } else { 0.0 });
    if fw > 0.0 {
        rounded(x, y, fw, h, h / 2.0, fill);
        rounded(x + 2.0, y + 1.0, (fw - 4.0).max(0.0), h * 0.35, h * 0.2, Color::new(1.0, 1.0, 1.0, 0.25));
    }
}

// ───────────────────────── Widgets ─────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Primary,
    Secondary,
    Danger,
}

/// A rounded button. Returns true when clicked. Lifts and glows on hover.
pub fn button(r: Rect, label: &str, enabled: bool, style: Style) -> bool {
    let hover = enabled && r.contains(mouse_position().into());
    let down = hover && is_mouse_button_down(MouseButton::Left);
    let lift = if down { 1.0 } else if hover { -2.0 } else { 0.0 };
    let base = match style {
        Style::Primary => Color::new(0.80, 0.24, 0.30, 1.0),
        Style::Secondary => Color::new(0.16, 0.12, 0.27, 1.0),
        Style::Danger => Color::new(0.35, 0.10, 0.14, 1.0),
    };
    let fill = if !enabled {
        Color::new(0.12, 0.10, 0.18, 0.8)
    } else if hover {
        Color::new((base.r * 1.25).min(1.0), (base.g * 1.25).min(1.0), (base.b * 1.25).min(1.0), 1.0)
    } else {
        base
    };
    let y = r.y + lift;
    rounded(r.x + 2.0, r.y + 4.0, r.w, r.h, 10.0, Color::new(0.0, 0.0, 0.0, 0.35));
    rounded(r.x, y, r.w, r.h, 10.0, fill);
    rounded(r.x + 2.0, y + 2.0, r.w - 4.0, r.h * 0.42, 8.0, Color::new(1.0, 1.0, 1.0, if enabled { 0.08 } else { 0.02 }));
    let edge = if !enabled {
        Color::new(1.0, 1.0, 1.0, 0.08)
    } else if hover {
        GOLD
    } else {
        Color::new(1.0, 0.82, 0.42, 0.4)
    };
    rounded_outline(r.x, y, r.w, r.h, 10.0, if hover { 2.0 } else { 1.5 }, edge);
    let size = fit(label, r.w - 20.0, (r.h * 0.42).clamp(16.0, 26.0));
    let d = measure(label, size as u16);
    let c = if enabled { INK } else { Color::new(1.0, 1.0, 1.0, 0.3) };
    text(label, r.x + (r.w - d.width) / 2.0, y + r.h / 2.0 + d.offset_y / 2.0 - 1.0, size, c);
    hover && is_mouse_button_pressed(MouseButton::Left)
}

/// Gap between UI and the screen edge, in virtual pixels.
pub const MARGIN: f32 = 24.0;

/// Keyboard intent for menu focus, read once per frame.
pub struct NavInput {
    pub prev: bool,
    pub next: bool,
    pub confirm: bool,
    pub digit: Option<usize>,
}

impl NavInput {
    pub fn read() -> NavInput {
        let any = |keys: &[KeyCode]| keys.iter().any(|&k| is_key_pressed(k));
        const DIGITS: [KeyCode; 9] = [
            KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4, KeyCode::Key5,
            KeyCode::Key6, KeyCode::Key7, KeyCode::Key8, KeyCode::Key9,
        ];
        NavInput {
            prev: any(&[KeyCode::Left, KeyCode::A, KeyCode::Up, KeyCode::W]),
            next: any(&[KeyCode::Right, KeyCode::D, KeyCode::Down, KeyCode::S]),
            confirm: any(&[KeyCode::Enter, KeyCode::KpEnter, KeyCode::Space]),
            digit: DIGITS.iter().position(|&k| is_key_pressed(k)),
        }
    }
}

/// Moves focus among `n` items (wrapping) and reports the item picked this frame, if any.
pub fn nav(focus: usize, n: usize, input: &NavInput) -> (usize, Option<usize>) {
    if n == 0 {
        return (0, None);
    }
    let mut focus = focus.min(n - 1);
    if let Some(d) = input.digit.filter(|&d| d < n) {
        return (d, Some(d));
    }
    if input.next {
        focus = (focus + 1) % n;
    } else if input.prev {
        focus = (focus + n - 1) % n;
    }
    (focus, input.confirm.then_some(focus))
}

/// Pulsing outline marking the keyboard-focused item.
pub fn focus_ring(x: f32, y: f32, w: f32, h: f32, accent: Color) {
    let pulse = 0.65 + 0.35 * (get_time() as f32 * 6.0).sin();
    rounded_outline(x, y, w, h, 16.0, 3.0, Color::new(accent.r, accent.g, accent.b, pulse));
}

/// A square icon slot with optional level pips underneath (VS-style inventory).
pub fn icon_slot(x: f32, y: f32, s: f32, icon: Option<&Texture2D>, level: u8, max: u8, gold: bool) {
    let edge = if gold { GOLD } else { Color::new(1.0, 1.0, 1.0, 0.18) };
    rounded(x, y, s, s, 6.0, if gold { Color::new(0.28, 0.20, 0.06, 0.95) } else { Color::new(0.06, 0.05, 0.11, 0.9) });
    rounded_outline(x, y, s, s, 6.0, if gold { 2.0 } else { 1.0 }, edge);
    if let Some(t) = icon {
        let pad = s * 0.12;
        draw_texture_ex(t, x + pad, y + pad, WHITE, DrawTextureParams { dest_size: Some(vec2(s - 2.0 * pad, s - 2.0 * pad)), ..Default::default() });
    }
    if max > 0 && icon.is_some() {
        let pip = ((s - 4.0) / max as f32).min(6.0);
        let total = pip * max as f32;
        let px = x + (s - total) / 2.0;
        for k in 0..max {
            let c = if gold {
                GOLD
            } else if k < level {
                SKY
            } else {
                Color::new(1.0, 1.0, 1.0, 0.15)
            };
            draw_rectangle(px + k as f32 * pip + 0.5, y + s + 2.0, pip - 1.0, 3.0, c);
        }
    }
}

/// Small label/value chip, e.g. a stat in a row.
pub fn chip(x: f32, y: f32, label: &str, value: &str, accent: Color) -> f32 {
    let lw = measure(label, 14).width;
    let vw = measure(value, 20).width.min(260.0);
    let w = lw + vw + 28.0;
    rounded(x, y, w, 30.0, 15.0, Color::new(0.06, 0.05, 0.11, 0.85));
    rounded_outline(x, y, w, 30.0, 15.0, 1.0, Color::new(accent.r, accent.g, accent.b, 0.5));
    text(label, x + 12.0, y + 20.0, 14.0, DIM);
    text(value, x + 18.0 + lw, y + 21.0, 20.0, accent);
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    fn i(prev: bool, next: bool, confirm: bool, digit: Option<usize>) -> NavInput {
        NavInput { prev, next, confirm, digit }
    }

    #[test]
    fn nav_wraps_and_confirms() {
        assert_eq!(nav(0, 3, &i(false, true, false, None)), (1, None));
        assert_eq!(nav(2, 3, &i(false, true, false, None)), (0, None));
        assert_eq!(nav(0, 3, &i(true, false, false, None)), (2, None));
        assert_eq!(nav(1, 3, &i(false, false, true, None)), (1, Some(1)));
    }

    #[test]
    fn nav_digits_and_bounds() {
        assert_eq!(nav(0, 2, &i(false, false, false, Some(1))), (1, Some(1)));
        assert_eq!(nav(0, 2, &i(false, false, false, Some(2))), (0, None));
        assert_eq!(nav(5, 2, &i(false, false, false, None)), (1, None));
        assert_eq!(nav(0, 0, &i(false, true, true, None)), (0, None));
    }
}
