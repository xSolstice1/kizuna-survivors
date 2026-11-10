//! Out-of-run screens: main menu, character select and the Shrine Shop.

use macroquad::prelude::*;
#[allow(unused_imports)]
use crate::kit::{draw_circle, draw_circle_lines};

use crate::kit;
use crate::kit::{mouse_position, screen_height, screen_width};
use sim::content::{awakened_name, awakened_text, complement, hero_def, Element, EnemyKind, Upgrade, ALL_ENEMIES, ALL_HEROES, ALL_PASSIVES, ALL_SHRINE_EVENTS, ALL_WEAPONS};
use sim::campaign::{Difficulty, StageId, ALL_DIFFICULTIES, ALL_RELICS, ALL_STAGES};
use kit::SAKURA;
use sim::meta::{collection_progress, enemy_found, unlock_text, PowerUp, Profile, ALL_POWERUPS};

use crate::render::Assets;

const PANEL: Color = Color::new(0.06, 0.05, 0.10, 0.88);
pub const ACCENT: Color = Color::new(1.0, 0.43, 0.63, 1.0);
const GOLD: Color = Color::new(1.0, 0.85, 0.4, 1.0);
const DIM: Color = Color::new(1.0, 1.0, 1.0, 0.55);

/// Unlike kit::centered, no drop shadow.
pub fn centered_at(text: &str, cx: f32, y: f32, size: f32, color: Color) {
    let d = kit::measure(text, size as u16);
    kit::text(text, cx - d.width / 2.0, y, size, color);
}

pub fn centered(text: &str, y: f32, size: f32, color: Color) {
    let d = kit::measure(text, size as u16);
    kit::text(text, (screen_width() - d.width) / 2.0, y, size, color);
}

/// Draws a button; returns true when clicked. Main actions get the vermilion style.
fn button(r: Rect, label: &str, enabled: bool) -> bool {
    let primary = matches!(label, "Start" | "Begin the night" | "Buy");
    kit::button(r, label, enabled, if primary { kit::Style::Primary } else { kit::Style::Secondary })
}

/// A fullscreen toggle (on iPhone Safari it explains Add to Home Screen instead).
pub fn fullscreen_button(r: Rect) {
    let ok = crate::platform::fullscreen_supported();
    if kit::button(r, crate::platform::fullscreen_label(), ok, kit::Style::Secondary) {
        crate::platform::toggle_fullscreen();
    }
}

fn sen_badge(assets: &Assets, sen: u32) {
    let v = format!("{sen}");
    let w = kit::measure(&v, 20).width + kit::measure("SEN", 14).width + 28.0;
    let x = screen_width() - w - kit::MARGIN;
    kit::chip(x, kit::MARGIN, "SEN", &v, GOLD);
    draw_texture_ex(&assets.coin, x - 26.0, kit::MARGIN + 4.0, WHITE, DrawTextureParams { dest_size: Some(vec2(22.0, 22.0)), ..Default::default() });
}

pub fn backdrop(time: f64) {
    scenery(time, Color::new(1.0, 0.45, 0.65, 1.0));
}

fn lerp_c(a: Color, b: Color, t: f32) -> Color {
    Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t)
}

/// Vertical gradient band.
pub fn gradient(x: f32, y: f32, w: f32, h: f32, top: Color, bottom: Color) {
    let n = ((h / 6.0) as usize).clamp(2, 80);
    let step = h / n as f32;
    for k in 0..n {
        let t = (k as f32 + 0.5) / n as f32;
        draw_rectangle(x, y + k as f32 * step, w, step + 0.6, lerp_c(top, bottom, t));
    }
}

/// A soft ridge line filled down to `bottom`: sum of sines, drawn as a strip of quads.
fn ridge(x0: f32, x1: f32, base: f32, amp: f32, freq: f32, seed: f32, drift: f32, bottom: f32, c: Color) {
    let step = 12.0;
    let mut x = x0;
    let hgt = |x: f32| {
        let u = x * freq + seed + drift;
        base - amp * (0.55 * (u).sin() + 0.3 * (u * 2.3 + 1.7).sin() + 0.15 * (u * 5.1 + 0.4).sin()).abs().powf(0.9)
    };
    while x < x1 {
        let (ya, yb) = (hgt(x), hgt(x + step));
        draw_triangle(vec2(x, ya), vec2(x + step, yb), vec2(x, bottom), c);
        draw_triangle(vec2(x + step, yb), vec2(x + step, bottom), vec2(x, bottom), c);
        x += step;
    }
}

/// The painted night used behind every menu: sky gradient, stars, a moon with shafts of
/// light, layered mountains, mist, a distant torii and lanterns, petals and fireflies.
pub fn scenery(time: f64, acc: Color) {
    let (w, h) = (screen_width(), screen_height());
    let t = time as f32;
    clear_background(Color::new(0.04, 0.03, 0.08, 1.0));
    let top = Color::new(0.05, 0.04, 0.12, 1.0);
    let mid = lerp_c(Color::new(0.16, 0.08, 0.22, 1.0), Color::new(acc.r * 0.3, acc.g * 0.2, acc.b * 0.35, 1.0), 0.35);
    gradient(0.0, 0.0, w, h * 0.62, top, mid);
    gradient(0.0, h * 0.62, w, h * 0.38, mid, Color::new(0.03, 0.02, 0.06, 1.0));
    // Stars.
    for k in 0..90 {
        let sx = ((k as f32 * 0.7548).fract()) * w;
        let sy = ((k as f32 * 0.5698).fract()) * h * 0.55;
        let tw = 0.35 + 0.65 * (t * (0.8 + (k % 5) as f32 * 0.3) + k as f32).sin().abs();
        let r = if k % 9 == 0 { 1.6 } else { 0.9 };
        draw_circle(sx, sy, r, Color::new(1.0, 0.95, 0.9, 0.55 * tw));
    }
    // Moon, halo and shafts.
    let (mx, my) = (w * 0.62, 70.0);
    for (r, a) in [(190.0, 0.025), (130.0, 0.04), (90.0, 0.07), (64.0, 0.1)] {
        draw_circle(mx, my, r + (t * 0.8).sin() * 4.0, Color::new(1.0, 0.92, 0.82, a));
    }
    draw_circle(mx, my, 46.0, Color::new(1.0, 0.96, 0.88, 0.95));
    for (cx, cy, r) in [(-14.0, -10.0, 9.0), (12.0, 8.0, 12.0), (-6.0, 18.0, 6.0), (18.0, -16.0, 5.0)] {
        draw_circle(mx + cx, my + cy, r, Color::new(0.88, 0.84, 0.78, 0.55));
    }
    draw_circle(mx + 10.0, my + 6.0, 46.0, Color::new(0.7, 0.62, 0.7, 0.12));
    for k in 0..6 {
        let ang = 2.0 + k as f32 * 0.17 + (t * 0.15 + k as f32).sin() * 0.03;
        let (dx, dy) = (ang.cos(), ang.sin());
        let len = h * 1.5;
        let side = vec2(-dy, dx) * (16.0 + k as f32 * 7.0);
        let p0 = vec2(mx, my);
        let p1 = p0 + vec2(dx, dy) * len;
        draw_triangle(p0, p1 + side, p1 - side, Color::new(1.0, 0.95, 0.85, 0.016 + 0.009 * ((t * 0.7 + k as f32).sin() + 1.0)));
    }
    // Mountains, far to near, slowly drifting.
    let far = lerp_c(Color::new(0.14, 0.1, 0.24, 1.0), acc, 0.08);
    ridge(0.0, w + 12.0, h * 0.58, h * 0.2, 0.004, 1.0, t * 0.01, h, far);
    // Mist between the ranges.
    for k in 0..3 {
        let y = h * (0.56 + 0.06 * k as f32) + (t * 0.3 + k as f32).sin() * 6.0;
        gradient(0.0, y - 20.0, w, 40.0, Color::new(0.7, 0.65, 0.95, 0.0), Color::new(0.7, 0.65, 0.95, 0.05));
    }
    let midc = Color::new(0.08, 0.06, 0.15, 1.0);
    ridge(0.0, w + 12.0, h * 0.7, h * 0.16, 0.007, 4.0, t * 0.02, h, midc);
    // A distant torii on the middle range, with a lantern glow.
    let (tx, ty) = (w * 0.24, h * 0.66);
    let tc = Color::new(0.05, 0.035, 0.09, 1.0);
    draw_rectangle(tx - 34.0, ty - 46.0, 6.0, 52.0, tc);
    draw_rectangle(tx + 28.0, ty - 46.0, 6.0, 52.0, tc);
    draw_rectangle(tx - 44.0, ty - 50.0, 88.0, 6.0, tc);
    draw_rectangle(tx - 38.0, ty - 38.0, 76.0, 4.0, tc);
    let glow = 0.5 + 0.2 * (t * 2.0).sin();
    draw_circle(tx, ty - 10.0, 26.0, Color::new(1.0, 0.6, 0.3, 0.08 * glow));
    draw_circle(tx, ty - 10.0, 4.0, Color::new(1.0, 0.75, 0.4, 0.8 * glow));
    let near = Color::new(0.03, 0.02, 0.06, 1.0);
    ridge(0.0, w + 12.0, h * 0.86, h * 0.08, 0.012, 9.0, t * 0.035, h, near);
    // Falling petals.
    for k in 0..46 {
        let seed = (k as f32 * 0.618).fract();
        let speed = 22.0 + (k % 7) as f32 * 6.0;
        let y = ((t * speed + k as f32 * 131.0) % (h + 40.0)) - 20.0;
        let x = (seed * (w + 200.0) - 100.0 + y * 0.35 + (t * 0.9 + k as f32).sin() * 26.0) % (w + 100.0);
        let rot = t * (1.5 + (k % 5) as f32 * 0.4) + k as f32;
        let sz = 3.0 + (k % 3) as f32;
        let a = vec2(rot.cos(), rot.sin()) * sz;
        let b = vec2(-rot.sin(), rot.cos()) * sz * 0.55;
        let c = vec2(x, y);
        let col = Color::new(1.0, 0.64 + 0.1 * (k % 2) as f32, 0.8, 0.6);
        draw_triangle(c + a, c + b, c - a, col);
        draw_triangle(c + a, c - b, c - a, col);
    }
    // Fireflies.
    for k in 0..24 {
        let fx = (k as f32 * 0.381).fract();
        let x = fx * w + (t * 0.4 + k as f32 * 2.0).sin() * 40.0;
        let y = h - ((t * (8.0 + k as f32 * 1.1) + k as f32 * 97.0) % (h + 60.0));
        let flick = 0.5 + 0.5 * (t * 3.0 + k as f32 * 1.7).sin();
        draw_circle(x, y, 8.0, Color::new(acc.r, acc.g, acc.b, 0.07 * flick));
        draw_circle(x, y, 2.2, Color::new(acc.r, acc.g, acc.b, 0.5 * flick + 0.2));
    }
    // Vignette.
    gradient(0.0, 0.0, w, 90.0, Color::new(0.0, 0.0, 0.0, 0.35), Color::new(0.0, 0.0, 0.0, 0.0));
    gradient(0.0, h - 120.0, w, 120.0, Color::new(0.0, 0.0, 0.0, 0.0), Color::new(0.0, 0.0, 0.0, 0.45));
}

pub enum MenuAction {
    Start,
    Armory,
    Draw,
    Shop,
    Collection,
    Stats,
    Settings,
    Dev,
}

/// Where the logo sits (tapping it 7 times opens the dev menu).
pub fn title_rect() -> Rect {
    Rect::new(30.0, 26.0, 520.0, 66.0)
}

/// One illustrated card on the main menu.
fn menu_card(r: Rect, label: &str, sub: &str, pip: Option<Element>, accent: Color, art: impl FnOnce(Rect, bool)) -> bool {
    let mouse: Vec2 = mouse_position().into();
    let hover = r.contains(mouse);
    let t = get_time() as f32;
    // Idle float (each card on its own phase), and a lift on hover.
    let lift = if hover { -10.0 } else { (t * 1.3 + r.x * 0.013).sin() * 3.0 };
    let r = Rect::new(r.x, r.y + lift, r.w, r.h);
    kit::rounded(r.x + 4.0, r.y + 8.0, r.w, r.h, 12.0, Color::new(0.0, 0.0, 0.0, 0.45));
    kit::rounded(r.x, r.y, r.w, r.h, 12.0, Color::new(0.05, 0.04, 0.09, 0.96));
    let art_r = Rect::new(r.x + 6.0, r.y + 6.0, r.w - 12.0, r.h - 76.0);
    art(art_r, hover);
    // Fade the art into the label band.
    for k in 0..12 {
        let t = k as f32 / 12.0;
        draw_rectangle(art_r.x, art_r.y + art_r.h - 36.0 + t * 36.0, art_r.w, 3.0, Color::new(0.05, 0.04, 0.09, t * 0.9));
    }
    if !hover {
        draw_rectangle(art_r.x, art_r.y, art_r.w, art_r.h, Color::new(0.0, 0.0, 0.0, 0.22));
    } else {
        // A light sweep across the art.
        let k = (t * 0.9).fract();
        let bx = art_r.x - 60.0 + (art_r.w + 120.0) * k;
        for (dw, a) in [(46.0, 0.05), (22.0, 0.08), (8.0, 0.12)] {
            let x0 = (bx - dw / 2.0).max(art_r.x);
            let x1 = (bx + dw / 2.0).min(art_r.x + art_r.w);
            if x1 > x0 {
                draw_rectangle(x0, art_r.y, x1 - x0, art_r.h, Color::new(1.0, 1.0, 1.0, a));
            }
        }
        // Sparks rising along the frame.
        for k in 0..14 {
            let f = ((t * 0.6 + k as f32 * 0.37) % 1.0).abs();
            let side = if k % 2 == 0 { r.x + 4.0 } else { r.x + r.w - 4.0 };
            let x = side + ((k * 13) % 7) as f32 - 3.0 + (t * 3.0 + k as f32).sin() * 3.0;
            let y = r.y + r.h - f * r.h;
            draw_circle(x, y, 1.8, Color::new(accent.r, accent.g, accent.b, 0.9 * (1.0 - f)));
        }
        draw_rectangle(r.x, r.y + r.h - 3.0, r.w, 3.0, Color::new(accent.r, accent.g, accent.b, 0.8));
    }
    kit::text_fit_centered(label, r.x + r.w / 2.0, r.y + r.h - 38.0, r.w - 16.0, 24.0, if hover { WHITE } else { Color::new(0.92, 0.9, 0.98, 1.0) });
    let sub_color = Color::new(accent.r, accent.g, accent.b, 0.9);
    match pip {
        Some(e) => {
            let size = kit::fit(sub, r.w - 16.0 - 14.0, 13.0);
            let left = r.x + r.w / 2.0 - (kit::measure(sub, size as u16).width + 14.0) / 2.0;
            crate::ui::element_pip(e, left + 5.0, r.y + r.h - 16.0 - size * 0.32, 5.0);
            kit::text(sub, left + 14.0, r.y + r.h - 16.0, size, sub_color);
        }
        None => kit::text_fit_centered(sub, r.x + r.w / 2.0, r.y + r.h - 16.0, r.w - 16.0, 13.0, sub_color),
    }
    kit::rounded_outline(r.x, r.y, r.w, r.h, 12.0, if hover { 2.5 } else { 1.5 }, if hover { accent } else { Color::new(1.0, 1.0, 1.0, 0.16) });
    hover && is_mouse_button_pressed(MouseButton::Left)
}

/// Draws a sprite standing at `bottom`, `h` tall (capped at 3.5x its art size so pixel art
/// never balloons into blocks).
fn tex_in(t: &Texture2D, cx: f32, bottom: f32, h: f32, tint: Color) {
    let n = crate::sprites::nat(t);
    let h = h.min(n.y * 3.5);
    let s = h / t.height();
    draw_texture_ex(t, (cx - t.width() * s / 2.0).round(), (bottom - h).round(), tint, DrawTextureParams { dest_size: Some(vec2(t.width() * s, h)), ..Default::default() });
}

pub fn main_menu(profile: &Profile, assets: &Assets, art: &crate::stage::StageArt, time: f64, dev: bool) -> Option<MenuAction> {
    let (w, h) = (screen_width(), screen_height());
    let t = time as f32;
    scenery(time, crate::stage::accent(profile.stage));

    // Logo: soft glow, gently bobbing letters and a light sweep.
    let tr = title_rect();
    let title = "KIZUNA SURVIVORS";
    let size = 50.0;
    let sweep = ((t * 0.35) % 1.6) - 0.3;
    let total = kit::measure(title, size as u16).width;
    let mut x = tr.x + 6.0;
    for (i, ch) in title.chars().enumerate() {
        let st = ch.to_string();
        let cw = kit::measure(&st, size as u16).width.max(if ch == ' ' { 14.0 } else { 0.0 });
        let rel = (x - tr.x) / total;
        let shine = (1.0 - ((rel - sweep).abs() * 8.0)).clamp(0.0, 1.0);
        let bob = (t * 2.2 + i as f32 * 0.45).sin() * 2.0;
        let y = tr.y + 46.0 + bob;
        kit::text(&st, x + 3.0, y + 3.0, size, Color::new(0.0, 0.0, 0.0, 0.5));
        for (o, a) in [(2.0, 0.12), (1.0, 0.18)] {
            kit::text(&st, x - o, y, size, Color::new(ACCENT.r, ACCENT.g, ACCENT.b, a));
            kit::text(&st, x + o, y, size, Color::new(ACCENT.r, ACCENT.g, ACCENT.b, a));
        }
        let c = Color::new(ACCENT.r + (1.0 - ACCENT.r) * shine, ACCENT.g + (1.0 - ACCENT.g) * shine, ACCENT.b + (1.0 - ACCENT.b) * shine, 1.0);
        kit::text(&st, x, y, size, c);
        x += cw;
    }
    kit::text("Heroes of many stories. Five worlds. One night each.", tr.x + 8.0, tr.y + 72.0, 15.0, DIM);

    // Currencies and small buttons, top-right.
    let sen = format!("{}", profile.sen);
    let sw = kit::measure(&sen, 20).width + kit::measure("SEN", 14).width + 28.0;
    kit::chip(w - 24.0 - sw, 26.0, "SEN", &sen, GOLD);
    draw_texture_ex(&assets.coin, w - 50.0 - sw, 30.0, WHITE, DrawTextureParams { dest_size: Some(vec2(22.0, 22.0)), ..Default::default() });
    let dust = format!("{}", profile.dust);
    let dw = kit::measure(&dust, 20).width + kit::measure("DUST", 14).width + 28.0;
    kit::chip(w - 70.0 - sw - dw, 26.0, "DUST", &dust, kit::SKY);
    let mut action = None;
    let (bw, bh) = (118.0, 34.0);
    let bx = w - 24.0 - 3.0 * bw - 16.0;
    if kit::button(Rect::new(bx, 68.0, bw, bh), "Stats", true, kit::Style::Secondary) {
        action = Some(MenuAction::Stats);
    }
    if kit::button(Rect::new(bx + bw + 8.0, 68.0, bw, bh), "Settings", true, kit::Style::Secondary) {
        action = Some(MenuAction::Settings);
    }
    fullscreen_button(Rect::new(bx + 2.0 * (bw + 8.0), 68.0, bw, bh));

    // The cards.
    let n = 5.0;
    let gap = 18.0;
    let cw = ((w - 100.0 - gap * (n - 1.0)) / n).min(220.0);
    let ch = (h - 220.0).clamp(320.0, 430.0);
    let x0 = (w - (n * cw + (n - 1.0) * gap)) / 2.0;
    let cy = 140.0 + (h - 220.0 - ch) / 2.0 + 10.0;
    let card = |i: usize| Rect::new(x0 + i as f32 * (cw + gap), cy, cw, ch);
    let stage = profile.stage;
    let sdef = stage.def();
    if menu_card(card(0), "START", &format!("{}  ·  {}", sdef.chapter, sdef.name), Some(stage.element()), crate::stage::accent(stage), |r, hover| {
        let c = crate::stage::accent(stage);
        gradient(r.x, r.y, r.w, r.h, Color::new(c.r * 0.25, c.g * 0.2, c.b * 0.35, 1.0), Color::new(0.04, 0.03, 0.08, 1.0));
        art.draw_patch(stage, r.x, r.y + r.h * 0.72, r.w, r.h * 0.28, 2.0);
        gradient(r.x, r.y + r.h * 0.72, r.w, r.h * 0.12, Color::new(0.04, 0.03, 0.08, 0.9), Color::new(0.04, 0.03, 0.08, 0.0));
        draw_circle(r.x + r.w / 2.0, r.y + r.h * 0.45, r.w * 0.42, Color::new(c.r, c.g, c.b, if hover { 0.22 } else { 0.14 }));
        let bob = (t * 2.0).sin() * 3.0;
        tex_in(assets.enemy_tex(sdef.final_boss), r.x + r.w / 2.0, r.y + r.h * 0.78 + bob, r.h * 0.62, WHITE);
        tex_in(&assets.hero(profile.selected)[0], r.x + r.w * 0.24, r.y + r.h - 8.0, 64.0, WHITE);
    }) {
        action = Some(MenuAction::Start);
    }
    if menu_card(card(1), "ARMORY", &format!("{} items", profile.items.len()), None, kit::SKY, |r, _| {
        gradient(r.x, r.y, r.w, r.h, Color::new(0.12, 0.18, 0.32, 1.0), Color::new(0.04, 0.03, 0.08, 1.0));
        draw_circle(r.x + r.w / 2.0, r.y + r.h * 0.48, r.w * 0.36, Color::new(0.35, 0.6, 1.0, 0.12));
        tex_in(&assets.hero(profile.selected)[0], r.x + r.w / 2.0, r.y + r.h * 0.8, r.h * 0.5, WHITE);
        let slots = [sim::gear::Slot::Hat, sim::gear::Slot::Ring, sim::gear::Slot::Charm, sim::gear::Slot::Shoes, sim::gear::Slot::Arsenal, sim::gear::Slot::Omamori];
        for (k, sl) in slots.iter().enumerate() {
            let a = t * 0.4 + k as f32 / 6.0 * std::f32::consts::TAU;
            let (ix, iy) = (r.x + r.w / 2.0 + a.cos() * r.w * 0.38 - 14.0, r.y + r.h * 0.45 + a.sin() * r.h * 0.3 - 14.0);
            kit::rounded(ix - 3.0, iy - 3.0, 34.0, 34.0, 7.0, Color::new(0.05, 0.05, 0.12, 0.85));
            draw_texture_ex(assets.gear_icon(*sl), ix, iy, WHITE, DrawTextureParams { dest_size: Some(vec2(28.0, 28.0)), ..Default::default() });
        }
    }) {
        action = Some(MenuAction::Armory);
    }
    if menu_card(card(2), "SHRINE DRAW", "Test your fortune", None, GOLD, |r, hover| {
        gradient(r.x, r.y, r.w, r.h, Color::new(0.32, 0.12, 0.14, 1.0), Color::new(0.04, 0.03, 0.08, 1.0));
        let glow = if hover { 0.3 } else { 0.18 };
        draw_circle(r.x + r.w / 2.0, r.y + r.h * 0.62, r.w * 0.3, Color::new(1.0, 0.75, 0.35, glow));
        tex_in(&assets.shrine, r.x + r.w / 2.0, r.y + r.h * 0.82, r.h * 0.55, WHITE);
        for k in 0..10 {
            let a = t * 0.8 + k as f32 * 0.628;
            let rr = r.w * (0.25 + 0.2 * ((k * 37 % 10) as f32 / 10.0));
            let (sx, sy) = (r.x + r.w / 2.0 + a.cos() * rr, r.y + r.h * 0.42 + a.sin() * rr * 0.8);
            draw_poly(sx, sy, 4, 2.5 + (t * 3.0 + k as f32).sin().abs() * 2.0, 45.0, Color::new(1.0, 0.9, 0.6, 0.8));
        }
        draw_texture_ex(assets.gear_icon_named("draw"), r.x + r.w - 52.0, r.y + 10.0, WHITE, DrawTextureParams { dest_size: Some(vec2(40.0, 40.0)), ..Default::default() });
    }) {
        action = Some(MenuAction::Draw);
    }
    if menu_card(card(3), "SHRINE SHOP", "Permanent blessings", None, kit::JADE, |r, _| {
        gradient(r.x, r.y, r.w, r.h, Color::new(0.1, 0.24, 0.18, 1.0), Color::new(0.04, 0.03, 0.08, 1.0));
        draw_circle(r.x + r.w / 2.0, r.y + r.h * 0.45, r.w * 0.4, Color::new(0.4, 1.0, 0.6, 0.1));
        for (k, name) in ["might", "guard", "haste", "vitality", "magnetism", "recovery"].iter().enumerate() {
            let (c, row) = (k % 2, k / 2);
            let ix = r.x + r.w * (0.3 + 0.4 * c as f32) - 22.0;
            let iy = r.y + 24.0 + row as f32 * (r.h * 0.25);
            kit::rounded(ix - 4.0, iy - 4.0, 52.0, 52.0, 9.0, Color::new(0.02, 0.06, 0.04, 0.8));
            draw_texture_ex(assets.icon_named(name), ix, iy, WHITE, DrawTextureParams { dest_size: Some(vec2(44.0, 44.0)), ..Default::default() });
        }
        for k in 0..3 {
            let bob = (t * 3.0 + k as f32).sin() * 3.0;
            draw_texture_ex(&assets.coin, r.x + r.w * 0.3 + k as f32 * 26.0, r.y + r.h - 70.0 + bob, WHITE, DrawTextureParams { dest_size: Some(vec2(22.0, 22.0)), ..Default::default() });
        }
    }) {
        action = Some(MenuAction::Shop);
    }
    let found = profile.enemy_kills.iter().filter(|k| **k > 0).count();
    if menu_card(card(4), "COLLECTIONS", &format!("{found} Otherworlders met"), None, SAKURA, |r, _| {
        gradient(r.x, r.y, r.w, r.h, Color::new(0.24, 0.1, 0.28, 1.0), Color::new(0.04, 0.03, 0.08, 1.0));
        let show = [EnemyKind::Wisp, EnemyKind::Kodama, EnemyKind::LeafShinobi, EnemyKind::LostSoul, EnemyKind::RainGhoul, EnemyKind::CursedSpawn, EnemyKind::Chochin, EnemyKind::LanternFairy, EnemyKind::CurseEye];
        for (k, e) in show.iter().enumerate() {
            let (c, row) = (k % 3, k / 3);
            let cx = r.x + r.w * (0.2 + 0.3 * c as f32);
            let by = r.y + 70.0 + row as f32 * (r.h * 0.26) + (t * 2.0 + k as f32).sin() * 2.0;
            let tint = if enemy_found(profile, *e) { WHITE } else { Color::new(0.0, 0.0, 0.0, 0.8) };
            tex_in(assets.enemy_tex(*e), cx, by, 46.0, tint);
        }
    }) {
        action = Some(MenuAction::Collection);
    }
    if is_key_pressed(KeyCode::Enter) {
        action = Some(MenuAction::Start);
    }

    // Account chip: who's signed in; opens Settings → Account.
    let acct = crate::cloud::account();
    if crate::cloud::status() != crate::cloud::Status::Off && !(acct.username.is_empty() && acct.email.is_empty()) {
        let name = if acct.username.is_empty() { acct.email.clone() } else { acct.username.clone() };
        let nw = kit::measure(&name, 15).width.min(260.0);
        let r = Rect::new(w - kit::MARGIN - nw - 62.0, h - kit::MARGIN - 34.0 - 22.0, nw + 62.0, 34.0);
        let hover = r.contains(mouse_position().into());
        kit::rounded(r.x, r.y, r.w, r.h, 17.0, Color::new(0.06, 0.05, 0.11, if hover { 0.95 } else { 0.8 }));
        kit::rounded_outline(r.x, r.y, r.w, r.h, 17.0, 1.2, if hover { GOLD } else { Color::new(1.0, 1.0, 1.0, 0.18) });
        draw_circle(r.x + 17.0, r.y + 17.0, 12.0, ACCENT);
        let initial: String = name.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default();
        kit::text_fit_centered(&initial, r.x + 17.0, r.y + 22.0, 20.0, 14.0, WHITE);
        kit::text_fit(&name, r.x + 36.0, r.y + 22.0, nw + 4.0, 15.0, kit::INK);
        draw_triangle(vec2(r.x + r.w - 18.0, r.y + 14.0), vec2(r.x + r.w - 10.0, r.y + 14.0), vec2(r.x + r.w - 14.0, r.y + 20.0), DIM);
        if hover && is_mouse_button_pressed(MouseButton::Left) {
            action = Some(MenuAction::Settings);
        }
    }
    if dev && kit::button(Rect::new(kit::MARGIN, h - kit::MARGIN - 36.0, 100.0, 36.0), "Dev", true, kit::Style::Danger) {
        action = Some(MenuAction::Dev);
    }
    // Copyright (the font has no © glyph, so the circle is drawn).
    let owner = "2026 xSolstice1";
    let ow = kit::measure(owner, 13).width;
    let (cx2, cy2) = (w - kit::MARGIN - ow - 14.0, h - kit::MARGIN - 4.0);
    draw_circle_lines(cx2, cy2, 6.5, 1.2, Color::new(1.0, 1.0, 1.0, 0.45));
    kit::text("c", cx2 - 3.2, cy2 + 3.6, 11.0, Color::new(1.0, 1.0, 1.0, 0.45));
    kit::text(owner, w - kit::MARGIN - ow, h - kit::MARGIN, 13.0, Color::new(1.0, 1.0, 1.0, 0.45));
    let hint = "WASD / arrows to move   ·   Space: special   ·   Esc to pause";
    kit::text_fit_centered(hint, w / 2.0, h - kit::MARGIN, w - 300.0, 13.0, Color::new(1.0, 1.0, 1.0, 0.4));
    action
}

/// One-line description of a hero's special move.
pub fn special_text(def: &sim::content::HeroDef) -> String {
    if !def.finished {
        return "Placeholder: own sprite and special move coming later.".to_string();
    }
    let what = match def.special {
        sim::content::SpecialKind::Barrier => "a purifying ring that erases fireballs",
        sim::content::SpecialKind::NineTails => "8s transformation: nine tails, aura, golden orb volleys",
        sim::content::SpecialKind::Hannya => "8s berserk: beams, crescent waves, shadow steps, finale blast",
        sim::content::SpecialKind::Mukade => "8s predator form: tendril barrage, eruptions, regeneration",
        sim::content::SpecialKind::Firmament => "8s domain: enemies suspended and crushed, ends in a giant void lance",
        sim::content::SpecialKind::Divine => "8s invulnerable: ofuda storm, yin-yang crashes, seal rings, final purification",
    };
    format!("Special: {} ({what})", def.special_name)
}

fn powerup_icon(assets: &Assets, p: PowerUp) -> &Texture2D {
    match p {
        PowerUp::Might => assets.icon_named("might"),
        PowerUp::Armor => assets.icon_named("guard"),
        PowerUp::MaxHp => assets.icon_named("vitality"),
        PowerUp::Recovery => assets.icon_named("recovery"),
        PowerUp::Cooldown => assets.icon_named("haste"),
        PowerUp::Area => assets.icon_named("expanse"),
        PowerUp::MoveSpeed => assets.icon_named("swiftness"),
        PowerUp::Magnet => assets.icon_named("magnetism"),
        PowerUp::Growth => &assets.gem,
        PowerUp::Greed => &assets.coin,
        PowerUp::Revival => &assets.shrine,
        PowerUp::AutoSpecial => assets.icon_named("persistence"),
    }
}

pub enum ShopAction {
    Back,
    /// Something was bought or refunded: save the profile.
    Changed,
}

pub fn shop(profile: &mut Profile, assets: &Assets, cursor: &mut usize, time: f64) -> Option<ShopAction> {
    backdrop(time);
    let w = screen_width();
    centered("SHRINE SHOP", 70.0, 44.0, GOLD);
    centered("Offer Sen for blessings that last every night.", 98.0, 20.0, DIM);
    sen_badge(assets, profile.sen);

    let cols = 6;
    let (cw, ch, gap) = (140.0, 120.0, 14.0);
    let x0 = (w - (cols as f32 * cw + (cols as f32 - 1.0) * gap)) / 2.0;
    let y0 = 126.0;
    let mouse: Vec2 = mouse_position().into();
    for (i, p) in ALL_POWERUPS.iter().enumerate() {
        let (col, row) = (i % cols, i / cols);
        let r = Rect::new(x0 + col as f32 * (cw + gap), y0 + row as f32 * (ch + gap), cw, ch);
        let sel = *cursor == i;
        let hover = r.contains(mouse);
        kit::panel_ex(r.x, r.y, r.w, r.h, if sel { Color::new(0.22, 0.16, 0.08, 0.97) } else { PANEL }, if sel { GOLD } else if hover { Color::new(1.0, 1.0, 1.0, 0.5) } else { Color::new(1.0, 1.0, 1.0, 0.2) });
        draw_texture_ex(powerup_icon(assets, *p), r.x + (cw - 40.0) / 2.0, r.y + 12.0, WHITE, DrawTextureParams { dest_size: Some(vec2(40.0, 40.0)), ..Default::default() });
        kit::text_fit_centered(p.name(), r.x + cw / 2.0, r.y + 74.0, cw - 12.0, 20.0, WHITE);
        // Rank pips.
        let max = p.max_rank() as f32;
        let pip = 12.0;
        let px0 = r.x + (cw - (max * pip + (max - 1.0) * 4.0)) / 2.0;
        for k in 0..p.max_rank() {
            let c = if k < profile.rank(*p) { GOLD } else { Color::new(1.0, 1.0, 1.0, 0.15) };
            draw_rectangle(px0 + k as f32 * (pip + 4.0), r.y + 92.0, pip, 8.0, c);
        }
        if hover && is_mouse_button_pressed(MouseButton::Left) {
            *cursor = i;
        }
    }
    let n = ALL_POWERUPS.len();
    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
        *cursor = (*cursor + n - 1) % n;
    }
    if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
        *cursor = (*cursor + 1) % n;
    }

    let p = ALL_POWERUPS[*cursor];
    let rows = n.div_ceil(cols) as f32;
    let (px, py, pw, ph) = ((w - 640.0) / 2.0, y0 + rows * (ch + gap) + 10.0, 640.0, 130.0);
    kit::panel_ex(px, py, pw, ph, PANEL, Color::new(1.0, 1.0, 1.0, 0.2));
    kit::text_fit(p.name(), px + 20.0, py + 38.0, pw - 180.0, 30.0, WHITE);
    kit::text(&format!("Rank {}/{}", profile.rank(p), p.max_rank()), px + pw - 140.0, py + 38.0, 22.0, DIM);
    kit::text_fit(p.describe(), px + 20.0, py + 72.0, pw - 40.0, 22.0, Color::new(0.9, 0.9, 0.95, 1.0));
    let maxed = profile.rank(p) >= p.max_rank();
    let price = if maxed { "Fully blessed".to_string() } else { format!("Price: {} Sen", profile.cost(p)) };
    kit::text(&price, px + 20.0, py + 108.0, 22.0, if maxed || profile.can_buy(p) { GOLD } else { Color::new(1.0, 0.5, 0.5, 1.0) });

    let mut action = None;
    let by = py + ph + 20.0;
    if button(Rect::new(w / 2.0 - 330.0, by, 200.0, 52.0), "Back", true) || is_key_pressed(KeyCode::Escape) {
        action = Some(ShopAction::Back);
    }
    let buy = button(Rect::new(w / 2.0 - 110.0, by, 220.0, 52.0), "Buy", profile.can_buy(p));
    if (buy || is_key_pressed(KeyCode::Enter)) && profile.buy(p) {
        action = Some(ShopAction::Changed);
    }
    if button(Rect::new(w / 2.0 + 130.0, by, 200.0, 52.0), "Refund all", profile.spent > 0) {
        profile.refund();
        action = Some(ShopAction::Changed);
    }
    if profile.spent > 0 {
        centered(&format!("Refund returns all {} Sen spent.", profile.spent), by + 80.0, 18.0, DIM);
    }
    action
}

// ───────────────────────── Collection ─────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Otherworlders,
    Weapons,
    Awakenings,
    Passives,
    Shrines,
    Heroes,
    Relics,
}

const TABS: [Tab; 7] = [Tab::Otherworlders, Tab::Weapons, Tab::Awakenings, Tab::Passives, Tab::Shrines, Tab::Heroes, Tab::Relics];

impl Tab {
    fn name(self) -> &'static str {
        match self {
            Tab::Otherworlders => "Otherworlders",
            Tab::Weapons => "Weapons",
            Tab::Awakenings => "Awakenings",
            Tab::Passives => "Passives",
            Tab::Shrines => "Shrine Events",
            Tab::Heroes => "Heroes",
            Tab::Relics => "Relics",
        }
    }
    fn len(self) -> usize {
        match self {
            Tab::Otherworlders => ALL_ENEMIES.len(),
            Tab::Weapons | Tab::Awakenings => ALL_WEAPONS.len(),
            Tab::Passives => ALL_PASSIVES.len(),
            Tab::Shrines => ALL_SHRINE_EVENTS.len(),
            Tab::Heroes => ALL_HEROES.len(),
            Tab::Relics => ALL_RELICS.len(),
        }
    }
}

pub struct CollectionState {
    pub tab: Tab,
    pub cursor: usize,
}

impl Default for CollectionState {
    fn default() -> Self {
        CollectionState { tab: Tab::Otherworlders, cursor: 0 }
    }
}

/// One entry: found flag, name, text lines.
struct Entry {
    found: bool,
    name: String,
    lines: Vec<String>,
    elem: Option<Element>,
}

fn enemy_icon(assets: &Assets, k: EnemyKind) -> (&Texture2D, Color) {
    (assets.enemy_tex(k), if k == EnemyKind::GreatOni { Color::new(0.85, 0.7, 1.0, 1.0) } else { WHITE })
}

/// Cheap discovery check: no strings, used for counts and grid cells.
fn found(profile: &Profile, tab: Tab, i: usize) -> bool {
    match tab {
        Tab::Otherworlders => enemy_found(profile, ALL_ENEMIES[i]),
        Tab::Weapons => profile.weapons_found.contains(&ALL_WEAPONS[i]),
        Tab::Awakenings => profile.awakenings_found.contains(&ALL_WEAPONS[i]),
        Tab::Passives => profile.passives_found.contains(&ALL_PASSIVES[i]),
        Tab::Shrines => profile.shrines_found.contains(&ALL_SHRINE_EVENTS[i]),
        Tab::Heroes => hero_def(ALL_HEROES[i]).finished && profile.is_unlocked(ALL_HEROES[i]),
        Tab::Relics => profile.relics.contains(&ALL_RELICS[i]),
    }
}

/// Grid-cell icon and tint, matching `entry` without building its text.
fn cell_icon<'a>(assets: &'a Assets, tab: Tab, i: usize) -> (&'a Texture2D, Color) {
    match tab {
        Tab::Otherworlders => enemy_icon(assets, ALL_ENEMIES[i]),
        Tab::Weapons => (assets.icon(Upgrade::Weapon(ALL_WEAPONS[i])), WHITE),
        Tab::Awakenings => (assets.icon(Upgrade::Weapon(ALL_WEAPONS[i])), GOLD),
        Tab::Passives => (assets.icon(Upgrade::Passive(ALL_PASSIVES[i])), WHITE),
        Tab::Shrines => (&assets.shrine, WHITE),
        Tab::Heroes => (&assets.hero(ALL_HEROES[i])[0], WHITE),
        Tab::Relics => (assets.icon_named(ALL_RELICS[i].icon()), WHITE),
    }
}

fn entry(profile: &Profile, tab: Tab, i: usize) -> Entry {
    match tab {
        Tab::Otherworlders => {
            let k = ALL_ENEMIES[i];
            let kills = profile.enemy_kills[i];
            Entry {
                found: found(profile, tab, i),
                name: k.name().into(),
                lines: vec![
                    k.lore().into(),
                    format!("{}  ·  Defeated: {kills}{}", sim::campaign::home_stage(k).def().name, if k.is_boss() { "   (boss)" } else { "" }),
                ],
                elem: Some(k.element()),
            }
        }
        Tab::Weapons => {
            let k = ALL_WEAPONS[i];
            let u = Upgrade::Weapon(k);
            let (p, _) = complement(k);
            let users: Vec<&str> = ALL_HEROES.iter().filter(|h| hero_def(**h).weapon_pool().contains(&k)).map(|h| hero_def(*h).name.split(' ').next().unwrap_or("")).collect();
            Entry {
                found: found(profile, tab, i),
                name: u.name().into(),
                lines: vec![u.describe(1), format!("Awakens with: {}", Upgrade::Passive(p).name()), format!("Used by: {}", users.join(", "))],
                elem: Some(k.element()),
            }
        }
        Tab::Awakenings => {
            let k = ALL_WEAPONS[i];
            let (p, _) = complement(k);
            Entry {
                found: found(profile, tab, i),
                name: awakened_name(k).into(),
                lines: vec![
                    awakened_text(k).into(),
                    recipe_text(Upgrade::Weapon(k).name(), Upgrade::Passive(p).name()),
                ],
                elem: Some(k.element()),
            }
        }
        Tab::Passives => {
            let k = ALL_PASSIVES[i];
            let u = Upgrade::Passive(k);
            Entry { found: found(profile, tab, i), name: u.name().into(), lines: vec![u.describe(1)], elem: None }
        }
        Tab::Shrines => {
            let ev = ALL_SHRINE_EVENTS[i];
            let (bless, price) = ev.text();
            Entry {
                found: found(profile, tab, i),
                name: ev.name().into(),
                lines: vec![format!("Blessing: {bless}"), format!("Price: {price}")],
                elem: None,
            }
        }
        Tab::Heroes if !hero_def(ALL_HEROES[i]).finished => Entry { found: false, name: "???".into(), lines: vec!["Coming soon.".into()], elem: None },
        Tab::Heroes => {
            let h = ALL_HEROES[i];
            let d = hero_def(h);
            Entry {
                found: found(profile, tab, i),
                name: d.name.into(),
                lines: vec![
                    format!("{}. Starts with {}.", d.archetype, Upgrade::Weapon(d.signature).name()),
                    format!("Innate: {}", d.innate_text),
                    format!("Affinity: {}", crate::ui::affinity_bonus_text()),
                    format!("Unlock: {}", unlock_text(h)),
                ],
                elem: Some(d.element),
            }
        }
        Tab::Relics => {
            let r = ALL_RELICS[i];
            let home = ALL_STAGES.iter().find(|s| s.def().relic == Some(r)).map_or("", |s| s.def().name);
            Entry {
                found: found(profile, tab, i),
                name: r.name().into(),
                lines: vec![r.describe().into(), format!("Always active. Earned by clearing {home} on Nightmare.")],
                elem: None,
            }
        }
    }
}

fn hint(tab: Tab) -> &'static str {
    match tab {
        Tab::Otherworlders => "Not yet defeated.",
        Tab::Weapons => "Not yet found. Pick it on a level-up to record it.",
        Tab::Awakenings => "Not yet awakened.",
        Tab::Passives => "Not yet found.",
        Tab::Shrines => "Not yet chosen at a roadside shrine.",
        Tab::Heroes => "Locked.",
        Tab::Relics => "Clear its world on Nightmare to earn it.",
    }
}

/// Returns true to go back.
pub fn collection(profile: &Profile, assets: &Assets, st: &mut CollectionState, time: f64) -> bool {
    backdrop(time);
    let w = screen_width();
    let (done, total) = collection_progress(profile);
    centered("COLLECTION", 64.0, 44.0, ACCENT);
    centered(&format!("{done} / {total} discovered"), 92.0, 20.0, DIM);

    // Tabs.
    let (tw, th, tg) = (156.0, 36.0, 8.0);
    let tx0 = (w - (TABS.len() as f32 * tw + (TABS.len() as f32 - 1.0) * tg)) / 2.0;
    let mouse: Vec2 = mouse_position().into();
    for (i, t) in TABS.iter().enumerate() {
        let r = Rect::new(tx0 + i as f32 * (tw + tg), 108.0, tw, th);
        let sel = st.tab == *t;
        let n = (0..t.len()).filter(|k| found(profile, *t, *k)).count();
        kit::panel_ex(r.x, r.y, r.w, r.h, if sel { Color::new(0.24, 0.12, 0.22, 0.97) } else { PANEL }, if sel { ACCENT } else { Color::new(1.0, 1.0, 1.0, 0.2) });
        let label = format!("{} {}/{}", t.name(), n, t.len());
        kit::text_fit_centered(&label, r.x + tw / 2.0, r.y + 24.0, tw - 12.0, 16.0, if sel { WHITE } else { DIM });
        if r.contains(mouse) && is_mouse_button_pressed(MouseButton::Left) {
            st.tab = *t;
            st.cursor = 0;
        }
    }
    if is_key_pressed(KeyCode::Tab) || is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::Q) {
        let i = TABS.iter().position(|t| *t == st.tab).unwrap();
        let j = if is_key_pressed(KeyCode::Q) { (i + TABS.len() - 1) % TABS.len() } else { (i + 1) % TABS.len() };
        st.tab = TABS[j];
        st.cursor = 0;
    }

    // Grid.
    let n = st.tab.len();
    // Big tabs (the bestiary) use smaller cells so the detail panel still fits.
    let (cols, cw, gap) = if n > 30 { (16.min(n), 62.0, 8.0) } else { (10.min(n), 84.0, 10.0) };
    let x0 = (w - (cols as f32 * cw + (cols as f32 - 1.0) * gap)) / 2.0;
    let y0 = 160.0;
    for i in 0..n {
        let (icon, icon_tint) = cell_icon(assets, st.tab, i);
        let is_found = found(profile, st.tab, i);
        let r = Rect::new(x0 + (i % cols) as f32 * (cw + gap), y0 + (i / cols) as f32 * (cw + gap), cw, cw);
        let sel = st.cursor == i;
        kit::panel_ex(r.x, r.y, r.w, r.h, if sel { Color::new(0.20, 0.12, 0.26, 0.97) } else { PANEL }, if sel { ACCENT } else if r.contains(mouse) { Color::new(1.0, 1.0, 1.0, 0.5) } else { Color::new(1.0, 1.0, 1.0, 0.2) });
        // Fit the icon into the cell keeping its aspect ratio.
        let (iw, ih) = { let n = crate::sprites::nat(icon); (n.x, n.y) };
        let k = ((cw - 22.0) / iw.max(ih)).floor().max(1.0).min(4.0);
        let (dw, dh) = (iw * k, ih * k);
        let tint = if is_found { icon_tint } else { Color::new(0.0, 0.0, 0.0, 0.85) };
        draw_texture_ex(icon, (r.x + (cw - dw) / 2.0).round(), (r.y + (cw - dh) / 2.0).round(), tint, DrawTextureParams { dest_size: Some(vec2(dw, dh)), ..Default::default() });
        if !is_found {
            let d = kit::measure("?", 30);
            kit::text("?", r.x + (cw - d.width) / 2.0, r.y + cw / 2.0 + 10.0, 30.0, DIM);
        }
        if r.contains(mouse) && is_mouse_button_pressed(MouseButton::Left) {
            st.cursor = i;
        }
    }
    if st.tab == Tab::Relics && st.cursor >= n {
        st.cursor = 0;
    }
    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
        st.cursor = (st.cursor + n - 1) % n;
    }
    if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
        st.cursor = (st.cursor + 1) % n;
    }
    st.cursor = st.cursor.min(n - 1);

    // Detail panel.
    let rows = n.div_ceil(cols) as f32;
    let e = entry(profile, st.tab, st.cursor);
    let (px, py, pw, ph) = ((w - 760.0) / 2.0, y0 + rows * (cw + gap) + 14.0, 760.0, 170.0);
    kit::panel_ex(px, py, pw, ph, PANEL, Color::new(1.0, 1.0, 1.0, 0.2));
    if e.found {
        let tag_w = e.elem.map_or(0.0, |el| crate::ui::element_tag_width(el, 20.0) + 16.0);
        kit::text_fit(&e.name, px + 20.0, py + 40.0, pw - 40.0 - tag_w, 30.0, if st.tab == Tab::Awakenings { GOLD } else { WHITE });
        if let Some(el) = e.elem {
            crate::ui::element_tag(el, px + pw - 20.0 - crate::ui::element_tag_width(el, 20.0), py + 40.0, 20.0);
        }
        let mut y = py + 72.0;
        for line in &e.lines {
            for l in kit::wrap(line, pw - 40.0, 20) {
                kit::text(&l, px + 20.0, y, 20.0, Color::new(0.9, 0.9, 0.95, 1.0));
                y += 24.0;
            }
        }
    } else {
        kit::text("???", px + 20.0, py + 40.0, 30.0, DIM);
        kit::text(hint(st.tab), px + 20.0, py + 74.0, 20.0, DIM);
        if st.tab == Tab::Heroes {
            let h = ALL_HEROES[st.cursor];
            let t = if hero_def(h).finished { format!("Unlock: {}", unlock_text(h)) } else { "Coming soon.".to_string() };
            kit::text(&t, px + 20.0, py + 104.0, 20.0, GOLD);
        }
    }
    button(Rect::new((w - 200.0) / 2.0, py + ph + 18.0, 200.0, 48.0), "Back", true) || is_key_pressed(KeyCode::Escape)
}

// ───────────────────────── Settings ─────────────────────────

pub enum SettingsAction {
    Back,
    /// A volume changed: apply it and save.
    Changed,
}

/// A 0..=1 slider with -/+ buttons. Returns true if the value changed.
fn slider(label: &str, y: f32, v: &mut f32) -> bool {
    let w = screen_width();
    let x = w / 2.0 - 300.0;
    kit::text(label, x, y + 30.0, 26.0, WHITE);
    let before = *v;
    if button(Rect::new(x + 200.0, y, 48.0, 44.0), "-", true) {
        *v -= 0.1;
    }
    let (bx, bw) = (x + 260.0, 280.0);
    draw_rectangle(bx, y + 16.0, bw, 12.0, Color::new(1.0, 1.0, 1.0, 0.12));
    draw_rectangle(bx, y + 16.0, bw * *v, 12.0, ACCENT);
    let r = Rect::new(bx, y + 4.0, bw, 36.0);
    if r.contains(mouse_position().into()) && is_mouse_button_down(MouseButton::Left) {
        *v = (mouse_position().0 - bx) / bw;
    }
    if button(Rect::new(bx + bw + 12.0, y, 48.0, 44.0), "+", true) {
        *v += 0.1;
    }
    *v = (*v * 20.0).round() / 20.0;
    *v = v.clamp(0.0, 1.0);
    kit::text(&format!("{:.0}%", *v * 100.0), bx + bw + 72.0, y + 30.0, 24.0, DIM);
    (*v - before).abs() > 1e-6
}

pub fn settings(profile: &mut Profile, time: f64) -> Option<SettingsAction> {
    backdrop(time);
    let h = screen_height();
    centered("SETTINGS", h * 0.22, 44.0, ACCENT);
    let mut changed = slider("Music", h * 0.36, &mut profile.music_volume);
    changed |= slider("Sound effects", h * 0.36 + 70.0, &mut profile.sfx_volume);
    fullscreen_button(Rect::new((screen_width() - 300.0) / 2.0, h * 0.36 + 140.0, 300.0, 46.0));
    centered("All music and sound effects are generated in code.", h * 0.36 + 210.0, 16.0, DIM);
    account_panel(h * 0.36 + 222.0);
    if button(Rect::new((screen_width() - 200.0) / 2.0, h * 0.36 + 380.0, 200.0, 46.0), "Back", true) || is_key_pressed(KeyCode::Escape) {
        return Some(SettingsAction::Back);
    }
    changed.then_some(SettingsAction::Changed)
}

/// Which sign-in form is showing.
#[derive(Default)]
pub struct LoginState {
    /// false = sign in, true = create account.
    pub create: bool,
    /// The "Forgot password?" form is open.
    pub forgot: bool,
    /// The reset link was sent (the form shows "Check your inbox").
    pub forgot_sent: bool,
    /// Where the reset link went.
    sent_to: String,
    /// Copies the sign-in box's email into the reset form once it exists.
    prefill: Option<String>,
    /// A problem found before asking the server (e.g. passwords that don't match).
    error: String,
}

/// A clickable text link; returns true when clicked.
fn link(label: &str, x: f32, y: f32, size: f32, right_align: bool) -> bool {
    let d = kit::measure(label, size as u16);
    let x = if right_align { x - d.width } else { x };
    let r = Rect::new(x - 2.0, y - size, d.width + 4.0, size + 6.0);
    let hover = r.contains(mouse_position().into());
    let c = if hover { GOLD } else { Color::new(1.0, 0.82, 0.55, 0.85) };
    kit::text(label, x, y, size, c);
    if hover {
        draw_line(x, y + 3.0, x + d.width, y + 3.0, 1.0, c);
    }
    hover && is_mouse_button_pressed(MouseButton::Left)
}

/// The four-colour "G" for the Google button.
fn google_mark(cx: f32, cy: f32, r: f32) {
    let segs = [(Color::from_rgba(234, 67, 53, 255), 200.0, 110.0), (Color::from_rgba(251, 188, 5, 255), 130.0, 70.0), (Color::from_rgba(52, 168, 83, 255), 60.0, 90.0), (Color::from_rgba(66, 133, 244, 255), -30.0, 90.0)];
    for (c, start, arc) in segs {
        draw_arc(cx, cy, 24, r, start, r * 0.42, arc, c);
    }
    draw_rectangle(cx, cy - r * 0.21, r * 1.0, r * 0.42, Color::from_rgba(66, 133, 244, 255));
}

fn google_button(r: Rect, enabled: bool) -> bool {
    let hover = enabled && r.contains(mouse_position().into());
    kit::rounded(r.x + 2.0, r.y + 3.0, r.w, r.h, 10.0, Color::new(0.0, 0.0, 0.0, 0.3));
    kit::rounded(r.x, r.y, r.w, r.h, 10.0, if hover { Color::new(1.0, 1.0, 1.0, 1.0) } else { Color::new(0.95, 0.95, 0.97, 1.0) });
    let label = "Continue with Google";
    let d = kit::measure(label, 17);
    let total = d.width + 30.0;
    let x0 = r.x + (r.w - total) / 2.0;
    google_mark(x0 + 9.0, r.y + r.h / 2.0, 9.0);
    kit::text(label, x0 + 30.0, r.y + r.h / 2.0 + 6.0, 17.0, Color::new(0.17, 0.17, 0.2, if enabled { 1.0 } else { 0.5 }));
    hover && is_mouse_button_pressed(MouseButton::Left)
}

/// The sign-in / sign-up screen (accounts are mandatory). Returns true once the player is
/// signed in and their save is loaded.
pub fn login(st: &mut LoginState, assets: &Assets, time: f64) -> bool {
    use crate::cloud::{self, Status};
    use crate::textinput::{self as ti, Key, Kind};
    let status = cloud::status();
    let recovery = cloud::recovery();
    if matches!(status, Status::Off | Status::Synced | Status::Syncing | Status::Error) && !recovery {
        // Leaving the login screen: hide its text fields; next time it opens on Sign in.
        st.forgot = false;
        st.forgot_sent = false;
        st.error.clear();
        ti::begin();
        ti::end();
        return true;
    }
    scenery(time, Color::new(1.0, 0.45, 0.65, 1.0));
    let (w, h) = (screen_width(), screen_height());
    let t = time as f32;
    let (cw, ch) = (900.0f32.min(w - 40.0), 520.0f32.min(h - 40.0));
    let (cx, cy) = ((w - cw) / 2.0, (h - ch) / 2.0);
    kit::rounded(cx + 6.0, cy + 10.0, cw, ch, 18.0, Color::new(0.0, 0.0, 0.0, 0.45));
    kit::rounded(cx, cy, cw, ch, 18.0, Color::new(0.06, 0.05, 0.11, 0.97));

    // ── Left: the game's art ──
    let aw = 380.0f32.min(cw * 0.42);
    let (ax, ay, ah) = (cx + 8.0, cy + 8.0, ch - 16.0);
    gradient(ax, ay, aw, ah, Color::new(0.42, 0.14, 0.32, 1.0), Color::new(0.09, 0.06, 0.18, 1.0));
    draw_circle(ax + aw * 0.72, ay + 92.0, 44.0, Color::new(1.0, 0.95, 0.85, 0.85));
    draw_circle(ax + aw * 0.72, ay + 92.0, 70.0, Color::new(1.0, 0.92, 0.8, 0.08));
    for k in 0..18 {
        let fx = (k as f32 * 0.618).fract();
        let y = ay + ah - ((t * (14.0 + k as f32) + k as f32 * 53.0) % ah);
        draw_circle(ax + fx * aw, y, 2.0, Color::new(1.0, 0.7, 0.85, 0.5));
    }
    kit::text_shadow("KIZUNA", ax + 30.0, ay + 190.0, 52.0, ACCENT);
    kit::text_shadow("SURVIVORS", ax + 30.0, ay + 240.0, 52.0, ACCENT);
    for (k, l) in kit::wrap("Heroes of many stories. Five worlds. One night each.", aw - 60.0, 16).iter().enumerate() {
        kit::text(l, ax + 32.0, ay + 272.0 + k as f32 * 20.0, 16.0, Color::new(1.0, 0.92, 0.96, 0.85));
    }
    draw_rectangle(ax, ay + ah - 70.0, aw, 70.0, Color::new(0.04, 0.03, 0.08, 0.55));
    let heroes = [sim::content::HeroId::Ren, sim::content::HeroId::Riku, sim::content::HeroId::Shun, sim::content::HeroId::Toya, sim::content::HeroId::Kuon];
    let step = (aw - 40.0) / heroes.len() as f32;
    for (k, hid) in heroes.iter().enumerate() {
        let bob = (t * 2.0 + k as f32).sin() * 2.0;
        let x = ax + 20.0 + step * (k as f32 + 0.5) - 30.0;
        draw_texture_ex(&assets.hero(*hid)[0], x, ay + ah - 104.0 + bob, WHITE, DrawTextureParams { dest_size: Some(vec2(60.0, 80.0)), ..Default::default() });
    }

    // ── Right: the form ──
    let fx = ax + aw + 44.0;
    let fw = cx + cw - 44.0 - fx;
    let mut y = cy + 56.0;
    ti::begin();
    let pending = cloud::pending();
    let (msg, kind) = if st.error.is_empty() { cloud::message() } else { (st.error.clone(), 1) };
    let show_msg = |y: f32| {
        if !msg.is_empty() {
            let c = match kind {
                1 => Color::new(1.0, 0.45, 0.45, 1.0),
                2 => kit::JADE,
                _ => DIM,
            };
            for (k, l) in kit::wrap(&msg, fw, 14).iter().take(2).enumerate() {
                kit::text(l, fx, y + k as f32 * 18.0, 14.0, c);
            }
        }
    };
    match (status, recovery) {
        (_, true) => {
            kit::text("Set a new password", fx, y + 6.0, 28.0, kit::INK);
            kit::text("Your reset link worked. Choose a new password for", fx, y + 36.0, 15.0, DIM);
            kit::text_fit(&format!("{}.", cloud::account().email), fx, y + 56.0, fw, 15.0, kit::INK);
            y += 104.0;
            ti::field(6, Kind::NewPassword, Rect::new(fx, y, fw, 44.0), "New password", "At least 8 characters");
            y += 44.0 + 30.0;
            ti::field(8, Kind::NewPassword, Rect::new(fx, y, fw, 44.0), "Confirm new password", "Type it again");
            show_msg(y + 64.0);
            y += 44.0 + 48.0;
            let key = ti::key();
            if let Key::Tab(6) = key {
                ti::focus(8);
            }
            let go = kit::button(Rect::new(fx, y, fw, 48.0), if pending { "Saving..." } else { "Save password" }, !pending, kit::Style::Primary);
            if (go || matches!(key, Key::Enter(6) | Key::Enter(8))) && !pending {
                let (a, b) = (ti::value(6), ti::value(8));
                st.error.clear();
                if a.chars().count() < 8 {
                    st.error = "Choose a longer password (at least 8 characters).".into();
                } else if a != b {
                    st.error = "The two passwords don't match.".into();
                } else {
                    cloud::set_password(&a);
                }
            }
        }
        (Status::Connecting, _) => {
            kit::text("Signing in", fx, y + 120.0, 30.0, kit::INK);
            kit::text("Loading your save...", fx, y + 150.0, 16.0, DIM);
            for k in 0..8 {
                let a = t * 4.0 + k as f32 * std::f32::consts::TAU / 8.0;
                let alpha = 0.2 + 0.8 * ((k as f32 / 8.0 + t * 0.6).fract());
                draw_circle(fx + 24.0 + a.cos() * 16.0, y + 200.0 + a.sin() * 16.0, 3.5, Color::new(1.0, 0.82, 0.42, alpha));
            }
        }
        (Status::Conflict, _) => {
            kit::text("Two saves found", fx, y + 40.0, 28.0, GOLD);
            for (k, l) in kit::wrap("This device and your account both have progress that the other doesn't. Which one do you want to keep?", fw, 15).iter().enumerate() {
                kit::text(l, fx, y + 76.0 + k as f32 * 20.0, 15.0, DIM);
            }
            if kit::button(Rect::new(fx, y + 150.0, fw, 46.0), "Keep account save", true, kit::Style::Primary) {
                cloud::resolve(true);
            }
            if kit::button(Rect::new(fx, y + 206.0, fw, 46.0), "Keep this device's save", true, kit::Style::Secondary) {
                cloud::resolve(false);
            }
        }
        _ if st.forgot => {
            if link("< Back to sign in", fx, y - 14.0, 14.0, false) {
                st.forgot = false;
                st.forgot_sent = false;
                cloud::clear_message();
            }
            y += 30.0;
            if kind == 2 && !msg.is_empty() {
                st.forgot_sent = true;
            }
            if !st.forgot_sent {
                kit::text("Reset your password", fx, y + 6.0, 28.0, kit::INK);
                for (k, l) in kit::wrap("Enter the email on your account and we'll send you a link to choose a new password.", fw, 15).iter().enumerate() {
                    kit::text(l, fx, y + 38.0 + k as f32 * 20.0, 15.0, DIM);
                }
                y += 112.0;
                ti::field(7, Kind::Email, Rect::new(fx, y, fw, 44.0), "Email", "you@example.com");
                if let Some(v) = st.prefill.take() {
                    ti::set(7, &v);
                    ti::focus(7);
                }
                show_msg(y + 64.0);
                y += 44.0 + 48.0;
                let go = kit::button(Rect::new(fx, y, fw, 48.0), if pending { "Sending..." } else { "Send reset link" }, !pending, kit::Style::Primary);
                if (go || matches!(ti::key(), Key::Enter(7))) && !pending {
                    st.sent_to = ti::value(7).trim().to_string();
                    cloud::reset_password(&st.sent_to);
                }
            } else {
                // Sent: an envelope, what happens next, and a way to send it again.
                let (ex, ey) = (fx + 26.0, y + 30.0);
                kit::rounded(ex - 26.0, ey - 26.0, 52.0, 52.0, 26.0, Color::new(0.2, 0.75, 0.55, 0.18));
                kit::rounded_outline(ex - 15.0, ey - 10.0, 30.0, 21.0, 3.0, 2.0, kit::JADE);
                draw_line(ex - 14.0, ey - 9.0, ex, ey + 2.0, 2.0, kit::JADE);
                draw_line(ex + 14.0, ey - 9.0, ex, ey + 2.0, 2.0, kit::JADE);
                kit::text("Check your inbox", fx + 68.0, ey + 9.0, 28.0, kit::INK);
                y += 86.0;
                let email = st.sent_to.clone();
                let body = format!(
                    "If {} has an account, a reset link is on its way. Open it on this device, then come back to this tab to choose a new password. Check spam if it hasn't arrived in a few minutes.",
                    email
                );
                for (k, l) in kit::wrap(&body, fw, 15).iter().enumerate() {
                    kit::text(l, fx, y + k as f32 * 21.0, 15.0, DIM);
                }
                y += 112.0;
                if kit::button(Rect::new(fx, y, fw, 46.0), if pending { "Sending..." } else { "Resend link" }, !pending, kit::Style::Secondary) {
                    cloud::reset_password(&email);
                }
                if kit::button(Rect::new(fx, y + 58.0, fw, 46.0), "Back to sign in", true, kit::Style::Primary) {
                    st.forgot = false;
                    st.forgot_sent = false;
                    cloud::clear_message();
                }
            }
        }
        _ => {
            // Segmented tabs.
            let tw = fw / 2.0;
            kit::rounded(fx, y - 30.0, fw, 38.0, 10.0, Color::new(0.03, 0.02, 0.06, 0.9));
            for (k, label) in ["Sign in", "Create account"].iter().enumerate() {
                let r = Rect::new(fx + 3.0 + k as f32 * tw, y - 27.0, tw - 6.0, 32.0);
                let on = st.create == (k == 1);
                if on {
                    kit::rounded(r.x, r.y, r.w, r.h, 8.0, Color::new(0.2, 0.13, 0.3, 1.0));
                }
                kit::text_fit_centered(label, r.x + r.w / 2.0, r.y + 22.0, r.w - 10.0, 15.0, if on { kit::INK } else { DIM });
                if r.contains(mouse_position().into()) && is_mouse_button_pressed(MouseButton::Left) && !on {
                    st.create = k == 1;
                    st.error.clear();
                    cloud::clear_message();
                }
            }
            y += 40.0;
            let field_h = 44.0;
            if !st.create {
                kit::text("Welcome back", fx, y + 6.0, 26.0, kit::INK);
                y += 48.0;
                ti::field(1, Kind::Text, Rect::new(fx, y, fw, field_h), "Username or email", "KuonFan or you@example.com");
                y += field_h + 30.0;
                ti::field(2, Kind::Password, Rect::new(fx, y, fw, field_h), "Password", "Your password");
                let forgot = link("Forgot password?", fx + fw, y + field_h + 20.0, 13.0, true);
                if forgot {
                    st.forgot = true;
                    st.forgot_sent = false;
                    let v = ti::value(1);
                    st.prefill = Some(if v.contains('@') { v } else { String::new() });
                    st.error.clear();
                    cloud::clear_message();
                }
                show_msg(y + field_h + 20.0);
                y += field_h + 44.0;
                let key = ti::key();
                if let Key::Tab(1) = key {
                    ti::focus(2);
                }
                let go = kit::button(Rect::new(fx, y, fw, 46.0), if pending { "Signing in..." } else { "Sign in" }, !pending, kit::Style::Primary);
                if (go || matches!(key, Key::Enter(1) | Key::Enter(2))) && !pending {
                    cloud::password_sign_in(&ti::value(1), &ti::value(2));
                }
                y += 62.0;
            } else {
                kit::text("Create your account", fx, y + 6.0, 26.0, kit::INK);
                y += 44.0;
                ti::field(3, Kind::Text, Rect::new(fx, y, fw, field_h), "Username", "3-16 letters, numbers or _");
                y += field_h + 28.0;
                ti::field(4, Kind::Email, Rect::new(fx, y, fw, field_h), "Email", "you@example.com");
                y += field_h + 28.0;
                ti::field(5, Kind::NewPassword, Rect::new(fx, y, fw, field_h), "Password", "At least 8 characters");
                show_msg(y + field_h + 20.0);
                y += field_h + 40.0;
                let key = ti::key();
                match key {
                    Key::Tab(3) => ti::focus(4),
                    Key::Tab(4) => ti::focus(5),
                    _ => {}
                }
                let go = kit::button(Rect::new(fx, y, fw, 46.0), if pending { "Creating..." } else { "Create account" }, !pending, kit::Style::Primary);
                if (go || matches!(key, Key::Enter(3) | Key::Enter(4) | Key::Enter(5))) && !pending {
                    cloud::sign_up(&ti::value(3), &ti::value(4), &ti::value(5));
                }
                y += 60.0;
            }
            // Divider and the other ways in.
            let (g, _, e) = cloud::providers();
            if g {
                draw_line(fx, y, fx + fw / 2.0 - 20.0, y, 1.0, Color::new(1.0, 1.0, 1.0, 0.15));
                draw_line(fx + fw / 2.0 + 20.0, y, fx + fw, y, 1.0, Color::new(1.0, 1.0, 1.0, 0.15));
                kit::text_fit_centered("or", fx + fw / 2.0, y + 5.0, 30.0, 13.0, kit::FAINT);
                if google_button(Rect::new(fx, y + 16.0, fw, 44.0), !pending) {
                    cloud::sign_in("google");
                }
                y += 76.0;
            }
            if e && !st.create && link("Email me a sign-in link instead", fx + fw / 2.0 - kit::measure("Email me a sign-in link instead", 13).width / 2.0, y + 4.0, 13.0, false) {
                cloud::magic_link(&ti::value(1));
            }
        }
    }
    ti::end();
    kit::text_fit_centered("(c) 2026 xSolstice1", w / 2.0, h - 10.0, 300.0, 12.0, Color::new(1.0, 1.0, 1.0, 0.35));
    false
}

/// The signed-in player's account: username, email, sign-in method, member since, cloud sync
/// state, and Sign out.
fn account_panel(y: f32) {
    use crate::cloud::{self, Status};
    let w = screen_width();
    let (pw, ph) = (600.0f32.min(w - 40.0), 144.0);
    let px = (w - pw) / 2.0;
    kit::panel(px, y, pw, ph);
    kit::panel_title("ACCOUNT", px + 16.0, y + 28.0, 16.0, GOLD);
    let st = cloud::status();
    if st == Status::Off {
        kit::text_fit("Cloud saves aren't set up on this build. Progress is saved in this browser.", px + 18.0, y + 62.0, pw - 36.0, 15.0, DIM);
        return;
    }
    let a = cloud::account();
    let (sync, c) = match st {
        Status::Synced => ("Saved to the cloud", kit::JADE),
        Status::Syncing | Status::Connecting => ("Syncing...", GOLD),
        Status::Conflict => ("Waiting for you to pick a save", GOLD),
        Status::SignedOut => ("Signed out", DIM),
        _ => ("Offline: will sync when you're back online", DIM),
    };
    let dash = |v: &str| if v.is_empty() { "-".to_string() } else { v.to_string() };
    let rows = [("Username", dash(&a.username)), ("Email", dash(&a.email)), ("Signed in with", dash(&a.method)), ("Member since", dash(&a.since))];
    let colw = (pw - 190.0) / 2.0;
    for (i, (label, value)) in rows.iter().enumerate() {
        let (cx, cy) = (px + 18.0 + (i % 2) as f32 * colw, y + 54.0 + (i / 2) as f32 * 36.0);
        kit::text(label, cx, cy, 12.0, kit::FAINT);
        kit::text_fit(value, cx, cy + 17.0, colw - 14.0, 15.0, kit::INK);
    }
    draw_circle(px + 22.0, y + ph - 16.0, 4.0, c);
    kit::text_fit(sync, px + 32.0, y + ph - 11.0, pw - 200.0, 13.0, c);
    if kit::button(Rect::new(px + pw - 150.0, y + 50.0, 132.0, 38.0), "Sign out", true, kit::Style::Danger) {
        cloud::sign_out();
    }
}

// ───────────────────────── Stats ─────────────────────────

fn recipe_text(weapon: &str, passive: &str) -> String {
    format!("Recipe: {weapon} at max level + {passive}, from a chest after {}.", fmt_time(sim::content::AWAKEN_FROM))
}

fn fmt_time(t: f32) -> String {
    let s = t.max(0.0) as u32;
    if s >= 3600 {
        format!("{}h {:02}m", s / 3600, (s / 60) % 60)
    } else {
        format!("{:02}:{:02}", s / 60, s % 60)
    }
}

/// A big number with a label underneath.
fn tile(x: f32, y: f32, w: f32, value: &str, label: &str, accent: Color) {
    kit::panel(x, y, w, 78.0);
    let d = kit::measure(value, 30);
    kit::text_shadow(value, x + (w - d.width) / 2.0, y + 40.0, 30.0, accent);
    let d = kit::measure(label, 14);
    kit::text(label, x + (w - d.width) / 2.0, y + 64.0, 14.0, kit::DIM);
}

/// Label on the left, value on the right.
fn row(x: f32, y: f32, w: f32, label: &str, value: &str, accent: Color) {
    kit::text(label, x, y, 17.0, kit::DIM);
    let d = kit::measure(value, 18);
    kit::text(value, x + w - d.width, y, 18.0, accent);
    draw_line(x, y + 9.0, x + w, y + 9.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.05));
}

/// Returns true to go back.
/// Horizontal scroll of the Stats heroes strip.
#[derive(Default)]
pub struct StripScroll {
    pub offset: f32,
    drag: Option<f32>,
}

pub fn stats(profile: &Profile, assets: &Assets, scroll: &mut StripScroll, time: f64) -> bool {
    backdrop(time);
    let w = screen_width();
    kit::centered("STATS", w / 2.0, 62.0, 44.0, ACCENT);

    // Headline tiles.
    let tiles = [
        (profile.runs.to_string(), "Nights played", kit::INK),
        (profile.wins.to_string(), "Night Lords defeated", GOLD),
        (profile.deaths.to_string(), "Fallen", kit::SAKURA),
        (profile.dawn_outs.to_string(), "Dawn without victory", Color::new(1.0, 0.6, 0.4, 1.0)),
        (fmt_time(profile.time_played), "Time in the night", kit::SKY),
        (profile.total_kills.to_string(), "Otherworlders defeated", kit::INK),
    ];
    let (tw, gap) = (180.0, 12.0);
    let x0 = (w - (tiles.len() as f32 * tw + (tiles.len() as f32 - 1.0) * gap)) / 2.0;
    for (i, (v, l, c)) in tiles.iter().enumerate() {
        tile(x0 + i as f32 * (tw + gap), 86.0, tw, v, l, *c);
    }

    // Three columns.
    let (cw, cy, ch) = (360.0, 182.0, 250.0);
    let total = 3.0 * cw + 2.0 * 20.0;
    let cx0 = (w - total) / 2.0;
    let pad = 22.0;
    // Records.
    let x = cx0;
    kit::panel(x, cy, cw, ch);
    kit::panel_title("RECORDS", x + 16.0, cy + 30.0, 16.0, GOLD);
    let winrate = if profile.runs > 0 { format!("{:.0}%", profile.wins as f32 / profile.runs as f32 * 100.0) } else { "-".into() };
    for (i, (l, v)) in [
        ("Longest night", fmt_time(profile.best_time)),
        ("Highest level", profile.best_level.to_string()),
        ("Most defeated in a night", profile.best_kills.to_string()),
        ("Great Oni defeated", profile.great_onis.to_string()),
        ("Win rate", winrate),
    ]
    .iter()
    .enumerate()
    {
        row(x + pad, cy + 70.0 + i as f32 * 34.0, cw - 2.0 * pad, l, v, kit::INK);
    }
    // Economy & feats.
    let x = cx0 + cw + 20.0;
    kit::panel(x, cy, cw, ch);
    kit::panel_title("SEN & FEATS", x + 16.0, cy + 30.0, 16.0, GOLD);
    let (found, total_c) = collection_progress(profile);
    for (i, (l, v, c)) in [
        ("Sen earned", profile.sen_earned.to_string(), GOLD),
        ("Sen offered at the shrine", profile.spent.to_string(), GOLD),
        ("Chests opened", profile.chests_opened.to_string(), kit::INK),
        ("Shrine blessings taken", profile.shrines_taken.to_string(), kit::INK),
        ("Awakenings achieved", profile.awakenings.to_string(), GOLD),
        ("Specials unleashed", profile.specials_used.to_string(), kit::INK),
        ("Collection", format!("{found}/{total_c}"), kit::SKY),
    ]
    .iter()
    .enumerate()
    {
        row(x + pad, cy + 62.0 + i as f32 * 26.0, cw - 2.0 * pad, l, v, *c);
    }
    // Most defeated Otherworlders.
    let x = cx0 + 2.0 * (cw + 20.0);
    kit::panel(x, cy, cw, ch);
    kit::panel_title("MOST DEFEATED", x + 16.0, cy + 30.0, 16.0, GOLD);
    let mut order: Vec<usize> = (0..ALL_ENEMIES.len()).filter(|&i| profile.enemy_kills[i] > 0).collect();
    order.sort_by(|a, b| profile.enemy_kills[*b].cmp(&profile.enemy_kills[*a]));
    let top = order.first().map_or(1, |&i| profile.enemy_kills[i].max(1));
    if order.is_empty() {
        kit::text("Nothing yet. The night awaits.", x + pad, cy + 80.0, 16.0, kit::FAINT);
    }
    for (row_i, &i) in order.iter().take(5).enumerate() {
        let k = ALL_ENEMIES[i];
        let y = cy + 50.0 + row_i as f32 * 38.0;
        let (icon, tint) = enemy_icon(assets, k);
        let (iw, ih) = { let n = crate::sprites::nat(icon); (n.x, n.y) };
        let sc = (28.0 / iw.max(ih)).min(2.0);
        draw_texture_ex(icon, x + pad, y + (30.0 - ih * sc) / 2.0, tint, DrawTextureParams { dest_size: Some(vec2(iw * sc, ih * sc)), ..Default::default() });
        kit::text(k.name(), x + pad + 40.0, y + 14.0, 15.0, kit::INK);
        let frac = profile.enemy_kills[i] as f32 / top as f32;
        kit::bar(x + pad + 40.0, y + 20.0, cw - 2.0 * pad - 110.0, 7.0, frac, kit::SAKURA, Color::new(1.0, 1.0, 1.0, 0.08));
        let v = profile.enemy_kills[i].to_string();
        let d = kit::measure(&v, 15);
        kit::text(&v, x + cw - pad - d.width, y + 27.0, 15.0, kit::DIM);
    }

    // Heroes strip: scrolls sideways (wheel, drag or the arrows) when it doesn't fit.
    let hy = cy + ch + 20.0;
    let heroes: Vec<_> = ALL_HEROES.iter().copied().filter(|h| profile.is_unlocked(*h)).collect();
    let (hw, hgap) = (150.0, 12.0);
    let content = heroes.len() as f32 * (hw + hgap) - hgap;
    let (vx, vw) = (60.0, w - 120.0);
    let view = Rect::new(vx, hy - 4.0, vw, 104.0);
    let max_scroll = (content - vw).max(0.0);
    let mouse: Vec2 = mouse_position().into();
    if max_scroll > 0.0 {
        let (wx, wy) = mouse_wheel();
        if view.contains(mouse) {
            scroll.offset -= (if wx.abs() > wy.abs() { wx } else { wy }) * 0.6;
        }
        if is_mouse_button_pressed(MouseButton::Left) && view.contains(mouse) {
            scroll.drag = Some(mouse.x);
        }
        if let Some(last) = scroll.drag {
            if is_mouse_button_down(MouseButton::Left) {
                scroll.offset -= mouse.x - last;
                scroll.drag = Some(mouse.x);
            } else {
                scroll.drag = None;
            }
        }
        if kit::button(Rect::new(14.0, hy + 28.0, 38.0, 40.0), "<", scroll.offset > 0.0, kit::Style::Secondary) {
            scroll.offset -= hw + hgap;
        }
        if kit::button(Rect::new(w - 52.0, hy + 28.0, 38.0, 40.0), ">", scroll.offset < max_scroll, kit::Style::Secondary) {
            scroll.offset += hw + hgap;
        }
        if is_key_pressed(KeyCode::Left) {
            scroll.offset -= hw + hgap;
        }
        if is_key_pressed(KeyCode::Right) {
            scroll.offset += hw + hgap;
        }
    }
    scroll.offset = scroll.offset.clamp(0.0, max_scroll);
    let hx0 = if max_scroll > 0.0 { vx - scroll.offset } else { (w - content) / 2.0 };
    kit::clip(Some(view));
    for (i, h) in heroes.iter().enumerate() {
        let x = hx0 + i as f32 * (hw + hgap);
        if x + hw < vx - 10.0 || x > vx + vw + 10.0 {
            continue;
        }
        kit::panel(x, hy, hw, 96.0);
        let tex = &assets.hero(*h)[0];
        draw_texture_ex(tex, x + 10.0, hy + 14.0, WHITE, DrawTextureParams { source: Some(Rect::new(0.0, 0.0, 24.0 * crate::sprites::UP, 18.0 * crate::sprites::UP)), dest_size: Some(vec2(48.0, 36.0)), ..Default::default() });
        let name = hero_def(*h).name.split(' ').next().unwrap_or("");
        kit::text(name, x + 64.0, hy + 30.0, 17.0, kit::INK);
        crate::ui::element_pip(hero_def(*h).element, x + 64.0 + kit::measure(name, 17).width + 10.0, hy + 24.0, 5.0);
        let hi = h.index();
        let plural = |n: u32, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
        kit::text(&plural(profile.hero_runs[hi], "night", "nights"), x + 64.0, hy + 50.0, 13.0, kit::DIM);
        kit::text(&plural(profile.hero_wins[hi], "win", "wins"), x + 12.0, hy + 74.0, 13.0, GOLD);
        let best = format!("best {}", fmt_time(profile.hero_best[hi]));
        let d = kit::measure(&best, 13);
        kit::text(&best, x + hw - 12.0 - d.width, hy + 74.0, 13.0, kit::SKY);
    }
    kit::clip(None);
    if max_scroll > 0.0 {
        // Scroll position bar.
        let frac = vw / content;
        let bw = vw * frac;
        let bx = vx + (vw - bw) * (scroll.offset / max_scroll);
        kit::rounded(vx, hy + 101.0, vw, 4.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.08));
        kit::rounded(bx, hy + 101.0, bw, 4.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.35));
    }

    button(Rect::new((w - 200.0) / 2.0, hy + 112.0, 200.0, 46.0), "Back", true) || is_key_pressed(KeyCode::Escape)
}

// ───────────────────────── Map select ─────────────────────────

pub enum StageAction {
    Go,
    Back,
}

fn requirement(s: StageId) -> String {
    match s.requires() {
        Some(r) => format!("Clear {} on Normal", r.def().name),
        None => String::new(),
    }
}

/// Map select: one card per world (art, name, lock, difficulty clears and best times,
/// boss progress, relic preview), a difficulty picker and the world's twist.
pub fn stage_select(profile: &mut Profile, assets: &Assets, art: &crate::stage::StageArt, time: f64) -> Option<StageAction> {
    backdrop(time);
    let w = screen_width();
    centered("CHOOSE YOUR BATTLEFIELD", 62.0, 40.0, ACCENT);
    sen_badge(assets, profile.sen);
    let mouse: Vec2 = mouse_position().into();
    let (cols, gap) = (3usize, 16.0);
    let cw = ((w - 60.0 - gap * 2.0) / 3.0).min(390.0);
    let ch = 222.0;
    let x0 = (w - (cols as f32 * cw + 2.0 * gap)) / 2.0;
    let y0 = 84.0;
    let t = time as f32;
    for (i, sid) in ALL_STAGES.iter().enumerate() {
        let def = sid.def();
        let r = Rect::new(x0 + (i % cols) as f32 * (cw + gap), y0 + (i / cols) as f32 * (ch + gap), cw, ch);
        let open = profile.stage_open(*sid);
        let sel = profile.stage == *sid;
        let hover = r.contains(mouse);
        let acc = crate::stage::accent(*sid);
        kit::panel_ex(r.x, r.y, r.w, r.h, if sel { Color::new(0.18, 0.11, 0.24, 0.97) } else { PANEL }, if sel { acc } else if hover { Color::new(1.0, 1.0, 1.0, 0.5) } else { Color::new(1.0, 1.0, 1.0, 0.18) });
        // Art strip: the world's ground with its final boss standing on it.
        let (ax, ay, aw, ah) = (r.x + 8.0, r.y + 8.0, r.w - 16.0, 104.0);
        art.draw_patch(*sid, ax, ay, aw, ah, 1.5);
        draw_rectangle(ax, ay, aw, ah, Color::new(0.0, 0.0, 0.0, 0.25));
        if let Some(prop) = art.world_tex(*sid, 4) {
            let s = 1.6;
            let n = crate::sprites::nat(prop);
            draw_texture_ex(prop, ax + 18.0, ay + ah - n.y * s - 4.0, if open { WHITE } else { Color::new(0.0, 0.0, 0.0, 0.8) }, DrawTextureParams { dest_size: Some(n * s), ..Default::default() });
        }
        let boss = assets.enemy_tex(def.final_boss);
        let bs = (ah - 6.0) / boss.height();
        let bob = if sel && open { (t * 2.5).sin() * 2.0 } else { 0.0 };
        if open {
            draw_circle(ax + aw - 60.0, ay + ah * 0.55, 46.0, Color::new(acc.r, acc.g, acc.b, 0.16));
        }
        draw_texture_ex(boss, ax + aw - 60.0 - boss.width() * bs / 2.0, ay + 3.0 + bob, if open { WHITE } else { Color::new(0.0, 0.0, 0.0, 0.85) }, DrawTextureParams { dest_size: Some(vec2(boss.width() * bs, boss.height() * bs)), ..Default::default() });
        // Name.
        kit::text(def.chapter, r.x + 14.0, r.y + 132.0, 14.0, if open { acc } else { DIM });
        kit::text_fit(def.name, r.x + 14.0, r.y + 156.0, r.w - 28.0, 22.0, if open { WHITE } else { DIM });
        if !open {
            draw_rectangle(r.x + 2.0, r.y + 2.0, r.w - 4.0, r.h - 4.0, Color::new(0.0, 0.0, 0.0, 0.45));
            kit::text_fit_centered("LOCKED", r.x + r.w / 2.0, r.y + 60.0, r.w - 30.0, 26.0, Color::new(1.0, 1.0, 1.0, 0.8));
            kit::text_fit_centered(&requirement(*sid), r.x + r.w / 2.0, r.y + 88.0, r.w - 30.0, 15.0, GOLD);
        } else {
            // Difficulty clears / best times.
            let si = sid.index();
            let dw = (r.w - 28.0) / 3.0;
            for d in ALL_DIFFICULTIES {
                let dx = r.x + 14.0 + d.index() as f32 * dw;
                let cleared = profile.cleared[si][d.index()];
                let avail = profile.difficulty_open(*sid, d);
                let col = if cleared { kit::JADE } else if avail { kit::INK } else { Color::new(1.0, 1.0, 1.0, 0.3) };
                if cleared {
                    draw_poly(dx + 6.0, r.y + 175.0, 4, 5.0, 45.0, kit::JADE);
                } else {
                    draw_poly_lines(dx + 6.0, r.y + 175.0, 4, 5.0, 45.0, 1.5, col);
                }
                kit::text_fit(d.name(), dx + 16.0, r.y + 180.0, dw - 18.0, 14.0, col);
                let best = profile.stage_best[si][d.index()];
                if best > 0.0 {
                    kit::text(&fmt_time(best), dx + 16.0, r.y + 197.0, 12.0, kit::DIM);
                }
            }
            // Boss progress: mini / boss / final (bits ever defeated).
            let bits = profile.stage_bosses[si];
            let kinds: Vec<(EnemyKind, u8)> = [Some((def.mini, 1u8)), def.boss.map(|b| (b, 2u8)), Some((def.final_boss, 4u8))].into_iter().flatten().collect();
            let mut bx = r.x + r.w - 14.0 - kinds.len() as f32 * 24.0;
            let el = sid.element();
            crate::ui::element_tag(el, bx - 12.0 - crate::ui::element_tag_width(el, 14.0), r.y + 132.0, 14.0);
            for (k, bit) in kinds {
                let tex = assets.enemy_tex(k);
                let s = 20.0 / tex.width().max(tex.height());
                let tint = if bits & bit != 0 { WHITE } else { Color::new(0.0, 0.0, 0.0, 0.7) };
                draw_texture_ex(tex, bx, r.y + 126.0, tint, DrawTextureParams { dest_size: Some(vec2(tex.width() * s, tex.height() * s)), ..Default::default() });
                bx += 24.0;
            }
            // Relic preview.
            if let Some(rel) = def.relic {
                let owned = profile.relics.contains(&rel);
                draw_texture_ex(assets.icon_named(rel.icon()), r.x + r.w - 34.0, r.y + 186.0, if owned { WHITE } else { Color::new(0.3, 0.3, 0.4, 0.9) }, DrawTextureParams { dest_size: Some(vec2(22.0, 22.0)), ..Default::default() });
            }
        }
        if open && hover && is_mouse_button_pressed(MouseButton::Left) {
            if profile.stage != *sid {
                profile.stage = *sid;
                if !profile.difficulty_open(*sid, profile.difficulty) {
                    profile.difficulty = Difficulty::Normal;
                }
            }
        }
    }

    // Selected world: difficulty picker, twist, reward.
    let sid = profile.stage;
    let def = sid.def();
    let py = y0 + 2.0 * (ch + gap) + 4.0;
    let pw = (w - 60.0).min(1180.0);
    let px = (w - pw) / 2.0;
    kit::panel_ex(px, py, pw, screen_height() - kit::MARGIN - py, PANEL, Color::new(1.0, 1.0, 1.0, 0.18));
    let bw = 150.0;
    for d in ALL_DIFFICULTIES {
        let rr = Rect::new(px + 16.0 + d.index() as f32 * (bw + 10.0), py + 14.0, bw, 40.0);
        let avail = profile.difficulty_open(sid, d);
        let chosen = profile.difficulty == d;
        let style = if chosen { kit::Style::Primary } else { kit::Style::Secondary };
        if kit::button(rr, d.name(), avail, style) {
            profile.difficulty = d;
        }
    }
    // Ascension (Nightmare+N) once Nightmare is cleared.
    let open_asc = profile.ascension_open(sid);
    if profile.difficulty == Difficulty::Nightmare && open_asc > 0 {
        profile.ascension = profile.ascension.min(open_asc);
        let ax = px + 16.0;
        let ay = py + 62.0;
        if kit::button(Rect::new(ax, ay, 30.0, 26.0), "-", profile.ascension > 0, kit::Style::Secondary) {
            profile.ascension -= 1;
        }
        let label = if profile.ascension == 0 { "Nightmare".to_string() } else { format!("Nightmare+{}", profile.ascension) };
        kit::text_fit_centered(&label, ax + 90.0, ay + 19.0, 110.0, 16.0, GOLD);
        if kit::button(Rect::new(ax + 150.0, ay, 30.0, 26.0), "+", profile.ascension < open_asc, kit::Style::Secondary) {
            profile.ascension += 1;
        }
        kit::text(&format!("Ascension open to +{open_asc}"), ax + 190.0, ay + 18.0, 13.0, DIM);
    }
    let asc = if profile.difficulty == Difficulty::Nightmare { profile.ascension } else { 0 };
    let d = profile.difficulty;
    let mods = format!(
        "Enemies x{:.1}  ·  Damage x{:.1}  ·  Health x{:.1}  ·  Elites {}  ·  Boss phases +{}  ·  Gear iLvl {}",
        d.density() * (1.0 + 0.04 * asc as f32).min(1.6),
        d.damage() * (1.0 + 0.12 * asc as f32),
        d.hp() * (1.0 + 0.4 * asc as f32),
        ["occasional", "frequent", "very frequent"][d.index()],
        d.extra_phases(),
        sim::gear::drop_ilvl(sid, d.index(), asc)
    );
    let tx = px + 16.0 + 3.0 * (bw + 10.0) + 6.0;
    let hero_x = px + pw - 286.0;
    let tw = hero_x - 14.0 - tx;
    for (k, l) in kit::wrap(&mods, tw, 13).iter().take(2).enumerate() {
        kit::text(l, tx, py + 28.0 + k as f32 * 15.0, 13.0, kit::INK);
    }
    let reward = match (d, def.relic) {
        (Difficulty::Nightmare, Some(r)) => format!("First clear: +{} Sen and the relic {} ({})", d.first_clear_sen(), r.name(), r.describe()),
        _ => format!("First clear: +{} Sen{}", d.first_clear_sen(), if d == Difficulty::Normal && sid != StageId::Shotengai { ", opens the next world" } else { "" }),
    };
    kit::text_fit(&reward, tx, py + 62.0, tw, 14.0, GOLD);
    let twist = if def.mechanic == sim::campaign::Mechanic::None {
        def.blurb.to_string()
    } else {
        format!("{}  ·  {}: {}", def.blurb, def.mechanic.name(), def.mechanic.describe())
    };
    let lines = kit::wrap(&twist, hero_x - 14.0 - (px + 16.0), 15);
    let ty = if profile.difficulty == Difficulty::Nightmare && open_asc > 0 { py + 106.0 } else { py + 80.0 };
    for (k, l) in lines.iter().take(2).enumerate() {
        kit::text(l, px + 16.0, ty + k as f32 * 19.0, 15.0, DIM);
    }
    let mut action = None;
    // The active hero (chosen in the Armory).
    let hdef = hero_def(profile.selected);
    draw_texture_ex(&assets.hero(profile.selected)[0], hero_x, py + 14.0, WHITE, DrawTextureParams { dest_size: Some(vec2(36.0, 48.0)), ..Default::default() });
    kit::text_fit(hdef.name.split(' ').next().unwrap_or(""), hero_x, py + 78.0, 80.0, 14.0, DIM);
    crate::ui::element_tag(hdef.element, hero_x, py + 98.0, 14.0);
    if button(Rect::new(px + pw - 196.0, py + 14.0, 180.0, 48.0), "Begin the night", profile.stage_open(sid)) || is_key_pressed(KeyCode::Enter) {
        action = Some(StageAction::Go);
    }
    if button(Rect::new(px + pw - 196.0, py + 70.0, 180.0, 36.0), "Back", true) || is_key_pressed(KeyCode::Escape) {
        action = Some(StageAction::Back);
    }
    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipe_text_uses_awaken_time() {
        let txt = recipe_text("Star Fall", "Might");
        assert!(txt.contains(&fmt_time(sim::content::AWAKEN_FROM)), "{txt}");
    }

    #[test]
    fn found_matches_each_tab_rule() {
        let mut p = Profile::default();
        p.enemy_kills[1] = 3;
        p.weapons_found.push(ALL_WEAPONS[2]);
        p.awakenings_found.push(ALL_WEAPONS[0]);
        p.passives_found.push(ALL_PASSIVES[1]);
        p.shrines_found.push(ALL_SHRINE_EVENTS[0]);
        p.relics.push(ALL_RELICS[0]);
        for t in TABS {
            let mut hits = 0;
            for i in 0..t.len() {
                let want = match t {
                    Tab::Otherworlders => enemy_found(&p, ALL_ENEMIES[i]),
                    Tab::Weapons => p.weapons_found.contains(&ALL_WEAPONS[i]),
                    Tab::Awakenings => p.awakenings_found.contains(&ALL_WEAPONS[i]),
                    Tab::Passives => p.passives_found.contains(&ALL_PASSIVES[i]),
                    Tab::Shrines => p.shrines_found.contains(&ALL_SHRINE_EVENTS[i]),
                    Tab::Heroes => hero_def(ALL_HEROES[i]).finished && p.is_unlocked(ALL_HEROES[i]),
                    Tab::Relics => p.relics.contains(&ALL_RELICS[i]),
                };
                assert_eq!(found(&p, t, i), want, "{} {i}", t.name());
                hits += want as usize;
            }
            assert!(hits > 0, "{} sample profile finds nothing", t.name());
        }
    }
}
