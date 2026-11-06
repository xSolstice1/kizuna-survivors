//! Out-of-run screens: main menu, character select and the Shrine Shop.

use macroquad::prelude::*;

use crate::kit;
use crate::kit::{mouse_position, screen_height, screen_width};
use sim::content::{awakened_name, awakened_text, complement, hero_def, EnemyKind, Upgrade, ALL_ENEMIES, ALL_HEROES, ALL_PASSIVES, ALL_SHRINE_EVENTS, ALL_WEAPONS};
use sim::campaign::{Difficulty, StageId, ALL_DIFFICULTIES, ALL_RELICS, ALL_STAGES};
use kit::SAKURA;
use sim::meta::{collection_progress, enemy_found, unlock_text, PowerUp, Profile, ALL_POWERUPS};

use crate::render::Assets;

const PANEL: Color = Color::new(0.06, 0.05, 0.10, 0.88);
pub const ACCENT: Color = Color::new(1.0, 0.43, 0.63, 1.0);
const GOLD: Color = Color::new(1.0, 0.85, 0.4, 1.0);
const DIM: Color = Color::new(1.0, 1.0, 1.0, 0.55);

pub fn centered(text: &str, y: f32, size: f32, color: Color) {
    let d = kit::measure(text, size as u16);
    kit::text(text, (screen_width() - d.width) / 2.0, y, size, color);
}

/// Draws a button; returns true when clicked. Main actions get the vermilion style.
fn button(r: Rect, label: &str, enabled: bool) -> bool {
    let primary = matches!(label, "Start" | "Begin the night" | "Buy" | "Choose hero");
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
    let x = screen_width() - w - 20.0;
    kit::chip(x, 18.0, "SEN", &v, GOLD);
    draw_texture_ex(&assets.coin, x - 26.0, 22.0, WHITE, DrawTextureParams { dest_size: Some(vec2(22.0, 22.0)), ..Default::default() });
}

pub fn backdrop(time: f64) {
    clear_background(Color::from_rgba(18, 14, 30, 255));
    let (w, h) = (screen_width(), screen_height());
    for k in 0..8 {
        let y = h * (0.55 + k as f32 * 0.06);
        draw_rectangle(0.0, y, w, h * 0.06, Color::new(0.35, 0.12, 0.25, 0.025 * k as f32));
    }
    draw_circle(w * 0.82, h * 0.2, 70.0, Color::new(1.0, 0.9, 0.7, 0.07));
    draw_circle(w * 0.82, h * 0.2, 46.0, Color::new(1.0, 0.92, 0.75, 0.10));
    // Drifting lantern lights.
    for k in 0..24 {
        let fx = (k as f32 * 0.618).fract();
        let x = fx * screen_width();
        let y = screen_height() - ((time as f32 * (12.0 + k as f32 * 1.7) + k as f32 * 97.0) % (screen_height() + 60.0));
        draw_circle(x, y, 3.0 + (k % 3) as f32, Color::new(1.0, 0.55 + 0.1 * (k % 3) as f32, 0.3, 0.25));
    }
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
fn menu_card(r: Rect, label: &str, sub: &str, accent: Color, art: impl FnOnce(Rect, bool)) -> bool {
    let mouse: Vec2 = mouse_position().into();
    let hover = r.contains(mouse);
    let lift = if hover { -6.0 } else { 0.0 };
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
    }
    kit::text_fit_centered(label, r.x + r.w / 2.0, r.y + r.h - 38.0, r.w - 16.0, 24.0, if hover { WHITE } else { Color::new(0.92, 0.9, 0.98, 1.0) });
    kit::text_fit_centered(sub, r.x + r.w / 2.0, r.y + r.h - 16.0, r.w - 16.0, 13.0, Color::new(accent.r, accent.g, accent.b, 0.9));
    kit::rounded_outline(r.x, r.y, r.w, r.h, 12.0, if hover { 2.5 } else { 1.5 }, if hover { accent } else { Color::new(1.0, 1.0, 1.0, 0.16) });
    hover && is_mouse_button_pressed(MouseButton::Left)
}

fn tex_in(t: &Texture2D, cx: f32, bottom: f32, h: f32, tint: Color) {
    let s = h / t.height();
    draw_texture_ex(t, (cx - t.width() * s / 2.0).round(), (bottom - h).round(), tint, DrawTextureParams { dest_size: Some(vec2(t.width() * s, h)), ..Default::default() });
}

pub fn main_menu(profile: &Profile, assets: &Assets, art: &crate::stage::StageArt, time: f64, dev: bool) -> Option<MenuAction> {
    let (w, h) = (screen_width(), screen_height());
    let t = time as f32;
    // Backdrop: the chosen world's ground, slowly drifting, darkened.
    let drift = (t * 6.0) % 96.0;
    art.draw_patch(profile.stage, -drift, -drift * 0.5, w + 96.0, h + 96.0, 3.0);
    draw_rectangle(0.0, 0.0, w, h, Color::new(0.03, 0.02, 0.07, 0.72));
    for k in 0..10 {
        let y = h * (0.5 + k as f32 * 0.05);
        draw_rectangle(0.0, y, w, h * 0.05, Color::new(0.0, 0.0, 0.02, 0.05 * k as f32));
    }
    for k in 0..28 {
        let fx = (k as f32 * 0.618).fract();
        let x = fx * w;
        let y = h - ((t * (10.0 + k as f32 * 1.3) + k as f32 * 97.0) % (h + 60.0));
        let c = crate::stage::accent(profile.stage);
        draw_circle(x, y, 2.0 + (k % 3) as f32, Color::new(c.r, c.g, c.b, 0.22));
    }

    // Logo.
    let tr = title_rect();
    kit::text_shadow("KIZUNA SURVIVORS", tr.x + 6.0, tr.y + 46.0, 50.0, ACCENT);
    kit::text("Heroes of many stories. Five worlds. One night each.", tr.x + 8.0, tr.y + 70.0, 15.0, DIM);

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
    let ch = 320.0;
    let x0 = (w - (n * cw + (n - 1.0) * gap)) / 2.0;
    let cy = 138.0;
    let card = |i: usize| Rect::new(x0 + i as f32 * (cw + gap), cy, cw, ch);
    let stage = profile.stage;
    let sdef = stage.def();
    if menu_card(card(0), "START", &format!("{}  ·  {}", sdef.chapter, sdef.name), crate::stage::accent(stage), |r, hover| {
        art.draw_patch(stage, r.x, r.y, r.w, r.h, 2.0);
        let c = crate::stage::accent(stage);
        draw_circle(r.x + r.w / 2.0, r.y + r.h * 0.45, r.w * 0.42, Color::new(c.r, c.g, c.b, if hover { 0.22 } else { 0.14 }));
        let bob = (t * 2.0).sin() * 3.0;
        tex_in(assets.enemy_tex(sdef.final_boss), r.x + r.w / 2.0, r.y + r.h * 0.78 + bob, r.h * 0.62, WHITE);
        tex_in(&assets.hero(profile.selected)[0], r.x + r.w * 0.24, r.y + r.h - 8.0, 64.0, WHITE);
    }) {
        action = Some(MenuAction::Start);
    }
    if menu_card(card(1), "ARMORY", &format!("{} items", profile.items.len()), kit::SKY, |r, _| {
        draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.08, 0.1, 0.18, 1.0));
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
    if menu_card(card(2), "SHRINE DRAW", "Test your fortune", GOLD, |r, hover| {
        draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.16, 0.06, 0.1, 1.0));
        let glow = if hover { 0.3 } else { 0.18 };
        draw_circle(r.x + r.w / 2.0, r.y + r.h * 0.5, r.w * 0.45, Color::new(1.0, 0.75, 0.35, glow));
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
    if menu_card(card(3), "SHRINE SHOP", "Permanent blessings", kit::JADE, |r, _| {
        draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.06, 0.13, 0.1, 1.0));
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
    if menu_card(card(4), "COLLECTIONS", &format!("{found} Otherworlders met"), SAKURA, |r, _| {
        draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.12, 0.06, 0.16, 1.0));
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

    // Campaign progress.
    let py = cy + ch + 34.0;
    centered("CAMPAIGN PROGRESS", py, 20.0, Color::new(1.0, 1.0, 1.0, 0.85));
    let (lx, rx) = (x0 + 30.0, x0 + n * cw + (n - 1.0) * gap - 30.0);
    let ty = py + 56.0;
    let count = ALL_STAGES.len();
    let at = |i: usize| lx + (rx - lx) * i as f32 / (count - 1) as f32;
    draw_line(lx, ty, rx, ty, 4.0, Color::new(1.0, 1.0, 1.0, 0.18));
    let reached = ALL_STAGES.iter().rposition(|s| profile.cleared[s.index()][0]).map_or(0, |i| i + 1).min(count - 1);
    if reached > 0 {
        draw_line(lx, ty, at(reached), ty, 4.0, ACCENT);
    }
    for (i, sid) in ALL_STAGES.iter().enumerate() {
        let x = at(i);
        let d = sid.def();
        let open = profile.stage_open(*sid);
        let cleared = profile.cleared[sid.index()];
        let c = crate::stage::accent(*sid);
        let label_c = if open { Color::new(1.0, 1.0, 1.0, 0.75) } else { Color::new(1.0, 1.0, 1.0, 0.3) };
        kit::text_fit_centered(d.chapter.to_uppercase().as_str(), x, ty - 22.0, 150.0, 12.0, label_c);
        if *sid == profile.stage {
            draw_circle_lines(x, ty, 17.0 + (t * 4.0).sin().abs() * 2.0, 2.0, c);
        }
        draw_circle(x, ty, 12.0, if cleared[0] { c } else if open { Color::new(0.25, 0.22, 0.35, 1.0) } else { Color::new(0.12, 0.11, 0.16, 1.0) });
        draw_circle_lines(x, ty, 12.0, 2.0, if open { Color::new(1.0, 1.0, 1.0, 0.7) } else { Color::new(1.0, 1.0, 1.0, 0.2) });
        // Hard / Nightmare clears as small diamonds.
        for (k, done) in [cleared[1], cleared[2]].iter().enumerate() {
            let dx = x - 7.0 + k as f32 * 14.0;
            if *done {
                draw_poly(dx, ty + 22.0, 4, 4.5, 45.0, if k == 0 { GOLD } else { Color::new(1.0, 0.35, 0.45, 1.0) });
            } else {
                draw_poly_lines(dx, ty + 22.0, 4, 4.5, 45.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.2));
            }
        }
        kit::text_fit_centered(d.name, x, ty + 46.0, 170.0, 13.0, label_c);
        let r = Rect::new(x - 40.0, ty - 30.0, 80.0, 60.0);
        if open && r.contains(mouse_position().into()) && is_mouse_button_pressed(MouseButton::Left) {
            action = Some(MenuAction::Start);
        }
    }

    if dev && kit::button(Rect::new(20.0, h - 52.0, 100.0, 36.0), "Dev", true, kit::Style::Danger) {
        action = Some(MenuAction::Dev);
    }
    let hint = "WASD / arrows to move   ·   Space: special   ·   Esc to pause";
    kit::text_fit_centered(hint, w / 2.0, h - 14.0, w - 300.0, 13.0, Color::new(1.0, 1.0, 1.0, 0.4));
    action
}

pub enum SelectAction {
    Begin,
    Back,
}

/// Character select. `cursor` is the highlighted hero index (kept between frames).
pub fn char_select(profile: &mut Profile, assets: &Assets, cursor: &mut usize, time: f64) -> Option<SelectAction> {
    backdrop(time);
    let w = screen_width();
    centered("CHOOSE YOUR HERO", 70.0, 44.0, ACCENT);
    sen_badge(assets, profile.sen);

    // Hero cards in a row.
    let n = ALL_HEROES.len();
    let gap = 12.0;
    let cw = ((w - 40.0 - gap * (n as f32 - 1.0)) / n as f32).min(150.0);
    let ch = 190.0;
    let x0 = (w - (n as f32 * cw + (n as f32 - 1.0) * gap)) / 2.0;
    let y0 = 110.0;
    let mouse: Vec2 = mouse_position().into();
    for (i, hero) in ALL_HEROES.iter().enumerate() {
        let r = Rect::new(x0 + i as f32 * (cw + gap), y0, cw, ch);
        let open = profile.is_unlocked(*hero);
        let sel = *cursor == i;
        let hover = r.contains(mouse);
        kit::panel_ex(r.x, r.y, r.w, r.h, if sel { Color::new(0.20, 0.12, 0.26, 0.97) } else { PANEL }, if sel { ACCENT } else if hover { Color::new(1.0, 1.0, 1.0, 0.5) } else { Color::new(1.0, 1.0, 1.0, 0.2) });
        let tint = if open { WHITE } else { Color::new(0.0, 0.0, 0.0, 0.85) };
        let bob = if sel && open { ((time * 4.0).sin() * 2.0) as f32 } else { 0.0 };
        draw_texture_ex(&assets.hero(*hero)[0], r.x + (cw - 72.0) / 2.0, r.y + 18.0 + bob, tint, DrawTextureParams { dest_size: Some(vec2(72.0, 96.0)), ..Default::default() });
        let def = hero_def(*hero);
        let name = if open { def.name.split(' ').next().unwrap_or(def.name) } else { "???" };
        kit::text_fit_centered(name, r.x + cw / 2.0, r.y + 140.0, cw - 12.0, 24.0, if open { WHITE } else { DIM });
        kit::text_fit_centered(def.archetype, r.x + cw / 2.0, r.y + 162.0, cw - 10.0, 16.0, DIM);
        if hover && is_mouse_button_pressed(MouseButton::Left) {
            *cursor = i;
        }
    }
    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
        *cursor = (*cursor + n - 1) % n;
    }
    if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
        *cursor = (*cursor + 1) % n;
    }

    // Detail panel.
    let hero = ALL_HEROES[*cursor];
    let def = hero_def(hero);
    let open = profile.is_unlocked(hero);
    // The panel grows to fit its (wrapped) lines.
    let pw = 760.0f32.min(w - 40.0);
    let inner = pw - 40.0;
    let innate = kit::wrap(&format!("Innate: {}", def.innate_text), inner, 20);
    let px = (w - pw) / 2.0;
    let py = y0 + ch + 20.0;
    let special_text = if !def.finished {
        "Placeholder: own sprite and special move coming later.".to_string()
    } else {
        let what = match def.special {
            sim::content::SpecialKind::Barrier => "a purifying ring that erases fireballs",
            sim::content::SpecialKind::NineTails => "8s transformation: nine tails, aura, golden orb volleys",
            sim::content::SpecialKind::Hannya => "8s berserk: beams, crescent waves, shadow steps, finale blast",
            sim::content::SpecialKind::Mukade => "8s predator form: tendril barrage, eruptions, regeneration",
            sim::content::SpecialKind::Firmament => "8s domain: enemies suspended and crushed, ends in a giant void lance",
            sim::content::SpecialKind::Divine => "8s invulnerable: ofuda storm, yin-yang crashes, seal rings, final purification",
        };
        format!("Special: {} ({what})", def.special_name)
    };
    let special = kit::wrap(&special_text, inner, 17);
    let ph = if open { 120.0 + innate.len() as f32 * 24.0 + special.len() as f32 * 21.0 + 14.0 } else { 150.0 };
    kit::panel_ex(px, py, pw, ph, PANEL, Color::new(1.0, 1.0, 1.0, 0.2));
    if open {
        kit::text_fit(def.name, px + 20.0, py + 40.0, inner, 32.0, WHITE);
        kit::text_fit(def.archetype, px + 20.0, py + 64.0, inner, 18.0, ACCENT);
        draw_texture_ex(assets.icon(Upgrade::Weapon(def.signature)), px + 20.0, py + 78.0, WHITE, DrawTextureParams { dest_size: Some(vec2(28.0, 28.0)), ..Default::default() });
        kit::text_fit(&format!("Starts with {}", Upgrade::Weapon(def.signature).name()), px + 58.0, py + 99.0, inner - 40.0, 20.0, WHITE);
        let mut ly = py + 132.0;
        for l in &innate {
            kit::text(l, px + 20.0, ly, 20.0, Color::new(0.6, 1.0, 0.7, 1.0));
            ly += 24.0;
        }
        if !def.finished {
            kit::text(&special_text, px + 20.0, ly, 17.0, DIM);
        } else {
            for l in &special {
                kit::text(l, px + 20.0, ly + 4.0, 17.0, Color::new(1.0, 0.8, 0.45, 1.0));
                ly += 21.0;
            }
        }
    } else {
        kit::text("LOCKED", px + 20.0, py + 44.0, 32.0, DIM);
        kit::text_fit(&format!("{} (unlock to reveal)", def.archetype), px + 20.0, py + 72.0, inner, 20.0, DIM);
        kit::text_fit(&format!("Unlock: {}", unlock_text(hero)), px + 20.0, py + 116.0, inner, 24.0, GOLD);
    }

    let mut action = None;
    let by = py + ph + 24.0;
    if button(Rect::new(w / 2.0 - 330.0, by, 200.0, 52.0), "Back", true) || is_key_pressed(KeyCode::Escape) {
        action = Some(SelectAction::Back);
    }
    let begin = button(Rect::new(w / 2.0 - 100.0, by, 430.0, 52.0), if open { "Begin the night" } else { "Locked" }, open);
    if open && (begin || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space)) {
        profile.selected = hero;
        action = Some(SelectAction::Begin);
    }
    action
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

/// One entry: (found, icon, name, text lines).
struct Entry<'a> {
    found: bool,
    icon: &'a Texture2D,
    tint: Color,
    name: String,
    lines: Vec<String>,
}

fn enemy_icon(assets: &Assets, k: EnemyKind) -> (&Texture2D, Color) {
    (assets.enemy_tex(k), if k == EnemyKind::GreatOni { Color::new(0.85, 0.7, 1.0, 1.0) } else { WHITE })
}

fn entry<'a>(profile: &Profile, assets: &'a Assets, tab: Tab, i: usize) -> Entry<'a> {
    match tab {
        Tab::Otherworlders => {
            let k = ALL_ENEMIES[i];
            let (icon, tint) = enemy_icon(assets, k);
            let kills = profile.enemy_kills[i];
            Entry {
                found: enemy_found(profile, k),
                icon,
                tint,
                name: k.name().into(),
                lines: vec![
                    k.lore().into(),
                    format!("{}  ·  Defeated: {kills}{}", sim::campaign::home_stage(k).def().name, if k.is_boss() { "   (boss)" } else { "" }),
                ],
            }
        }
        Tab::Weapons => {
            let k = ALL_WEAPONS[i];
            let u = Upgrade::Weapon(k);
            let (p, _) = complement(k);
            let users: Vec<&str> = ALL_HEROES.iter().filter(|h| hero_def(**h).weapon_pool().contains(&k)).map(|h| hero_def(*h).name.split(' ').next().unwrap_or("")).collect();
            Entry {
                found: profile.weapons_found.contains(&k),
                icon: assets.icon(u),
                tint: WHITE,
                name: u.name().into(),
                lines: vec![u.describe(1), format!("Awakens with: {}", Upgrade::Passive(p).name()), format!("Used by: {}", users.join(", "))],
            }
        }
        Tab::Awakenings => {
            let k = ALL_WEAPONS[i];
            let (p, _) = complement(k);
            Entry {
                found: profile.awakenings_found.contains(&k),
                icon: assets.icon(Upgrade::Weapon(k)),
                tint: GOLD,
                name: awakened_name(k).into(),
                lines: vec![
                    awakened_text(k).into(),
                    format!("Recipe: {} at max level + {}, from a chest after 10:00.", Upgrade::Weapon(k).name(), Upgrade::Passive(p).name()),
                ],
            }
        }
        Tab::Passives => {
            let k = ALL_PASSIVES[i];
            let u = Upgrade::Passive(k);
            Entry { found: profile.passives_found.contains(&k), icon: assets.icon(u), tint: WHITE, name: u.name().into(), lines: vec![u.describe(1)] }
        }
        Tab::Shrines => {
            let ev = ALL_SHRINE_EVENTS[i];
            let (bless, price) = ev.text();
            Entry {
                found: profile.shrines_found.contains(&ev),
                icon: &assets.shrine,
                tint: WHITE,
                name: ev.name().into(),
                lines: vec![format!("Blessing: {bless}"), format!("Price: {price}")],
            }
        }
        Tab::Heroes => {
            let h = ALL_HEROES[i];
            let d = hero_def(h);
            Entry {
                found: profile.is_unlocked(h),
                icon: &assets.hero(h)[0],
                tint: WHITE,
                name: d.name.into(),
                lines: vec![
                    format!("{}. Starts with {}.", d.archetype, Upgrade::Weapon(d.signature).name()),
                    format!("Innate: {}", d.innate_text),
                    format!("Unlock: {}", unlock_text(h)),
                ],
            }
        }
        Tab::Relics => {
            let r = ALL_RELICS[i];
            let home = ALL_STAGES.iter().find(|s| s.def().relic == Some(r)).map_or("", |s| s.def().name);
            Entry {
                found: profile.relics.contains(&r),
                icon: assets.icon_named(r.icon()),
                tint: WHITE,
                name: r.name().into(),
                lines: vec![r.describe().into(), format!("Always active. Earned by clearing {home} on Nightmare.")],
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
    let (found, total) = collection_progress(profile);
    centered("COLLECTION", 64.0, 44.0, ACCENT);
    centered(&format!("{found} / {total} discovered"), 92.0, 20.0, DIM);

    // Tabs.
    let (tw, th, tg) = (156.0, 36.0, 8.0);
    let tx0 = (w - (TABS.len() as f32 * tw + (TABS.len() as f32 - 1.0) * tg)) / 2.0;
    let mouse: Vec2 = mouse_position().into();
    for (i, t) in TABS.iter().enumerate() {
        let r = Rect::new(tx0 + i as f32 * (tw + tg), 108.0, tw, th);
        let sel = st.tab == *t;
        let n = (0..t.len()).filter(|k| entry(profile, assets, *t, *k).found).count();
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
        let e = entry(profile, assets, st.tab, i);
        let r = Rect::new(x0 + (i % cols) as f32 * (cw + gap), y0 + (i / cols) as f32 * (cw + gap), cw, cw);
        let sel = st.cursor == i;
        kit::panel_ex(r.x, r.y, r.w, r.h, if sel { Color::new(0.20, 0.12, 0.26, 0.97) } else { PANEL }, if sel { ACCENT } else if r.contains(mouse) { Color::new(1.0, 1.0, 1.0, 0.5) } else { Color::new(1.0, 1.0, 1.0, 0.2) });
        // Fit the icon into the cell keeping its aspect ratio.
        let (iw, ih) = (e.icon.width(), e.icon.height());
        let k = ((cw - 22.0) / iw.max(ih)).floor().max(1.0).min(4.0);
        let (dw, dh) = (iw * k, ih * k);
        let tint = if e.found { e.tint } else { Color::new(0.0, 0.0, 0.0, 0.85) };
        draw_texture_ex(e.icon, (r.x + (cw - dw) / 2.0).round(), (r.y + (cw - dh) / 2.0).round(), tint, DrawTextureParams { dest_size: Some(vec2(dw, dh)), ..Default::default() });
        if !e.found {
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
    let e = entry(profile, assets, st.tab, st.cursor);
    let (px, py, pw, ph) = ((w - 760.0) / 2.0, y0 + rows * (cw + gap) + 14.0, 760.0, 170.0);
    kit::panel_ex(px, py, pw, ph, PANEL, Color::new(1.0, 1.0, 1.0, 0.2));
    if e.found {
        kit::text_fit(&e.name, px + 20.0, py + 40.0, pw - 40.0, 30.0, if st.tab == Tab::Awakenings { GOLD } else { WHITE });
        let mut y = py + 72.0;
        for line in &e.lines {
            for l in wrap(line, pw - 40.0, 20) {
                kit::text(&l, px + 20.0, y, 20.0, Color::new(0.9, 0.9, 0.95, 1.0));
                y += 24.0;
            }
        }
    } else {
        kit::text("???", px + 20.0, py + 40.0, 30.0, DIM);
        kit::text(hint(st.tab), px + 20.0, py + 74.0, 20.0, DIM);
        if st.tab == Tab::Heroes {
            kit::text(&format!("Unlock: {}", unlock_text(ALL_HEROES[st.cursor])), px + 20.0, py + 104.0, 20.0, GOLD);
        }
    }
    button(Rect::new((w - 200.0) / 2.0, py + ph + 18.0, 200.0, 48.0), "Back", true) || is_key_pressed(KeyCode::Escape)
}

fn wrap(text: &str, width: f32, size: u16) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if kit::measure(&candidate, size).width > width && !line.is_empty() {
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
    centered("All music and sound effects are generated in code.", h * 0.36 + 220.0, 16.0, DIM);
    if button(Rect::new((screen_width() - 200.0) / 2.0, h * 0.36 + 250.0, 200.0, 48.0), "Back", true) || is_key_pressed(KeyCode::Escape) {
        return Some(SettingsAction::Back);
    }
    changed.then_some(SettingsAction::Changed)
}

// ───────────────────────── Stats ─────────────────────────

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
pub fn stats(profile: &Profile, assets: &Assets, time: f64) -> bool {
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
        let (iw, ih) = (icon.width(), icon.height());
        let sc = (28.0 / iw.max(ih)).min(2.0);
        draw_texture_ex(icon, x + pad, y + (30.0 - ih * sc) / 2.0, tint, DrawTextureParams { dest_size: Some(vec2(iw * sc, ih * sc)), ..Default::default() });
        kit::text(k.name(), x + pad + 40.0, y + 14.0, 15.0, kit::INK);
        let frac = profile.enemy_kills[i] as f32 / top as f32;
        kit::bar(x + pad + 40.0, y + 20.0, cw - 2.0 * pad - 110.0, 7.0, frac, kit::SAKURA, Color::new(1.0, 1.0, 1.0, 0.08));
        let v = profile.enemy_kills[i].to_string();
        let d = kit::measure(&v, 15);
        kit::text(&v, x + cw - pad - d.width, y + 27.0, 15.0, kit::DIM);
    }

    // Heroes strip.
    let hy = cy + ch + 20.0;
    let heroes: Vec<_> = ALL_HEROES.iter().copied().filter(|h| profile.is_unlocked(*h)).collect();
    let hw = 150.0;
    let hx0 = (w - (heroes.len() as f32 * hw + (heroes.len() as f32 - 1.0) * 12.0)) / 2.0;
    for (i, h) in heroes.iter().enumerate() {
        let x = hx0 + i as f32 * (hw + 12.0);
        kit::panel(x, hy, hw, 96.0);
        let tex = &assets.hero(*h)[0];
        draw_texture_ex(tex, x + 10.0, hy + 14.0, WHITE, DrawTextureParams { source: Some(Rect::new(0.0, 0.0, 24.0, 18.0)), dest_size: Some(vec2(48.0, 36.0)), ..Default::default() });
        let name = hero_def(*h).name.split(' ').next().unwrap_or("");
        kit::text(name, x + 64.0, hy + 30.0, 17.0, kit::INK);
        let hi = h.index();
        let plural = |n: u32, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
        kit::text(&plural(profile.hero_runs[hi], "night", "nights"), x + 64.0, hy + 50.0, 13.0, kit::DIM);
        kit::text(&plural(profile.hero_wins[hi], "win", "wins"), x + 12.0, hy + 74.0, 13.0, GOLD);
        let best = format!("best {}", fmt_time(profile.hero_best[hi]));
        let d = kit::measure(&best, 13);
        kit::text(&best, x + hw - 12.0 - d.width, hy + 74.0, 13.0, kit::SKY);
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
            draw_texture_ex(prop, ax + 18.0, ay + ah - prop.height() * s - 4.0, if open { WHITE } else { Color::new(0.0, 0.0, 0.0, 0.8) }, DrawTextureParams { dest_size: Some(vec2(prop.width() * s, prop.height() * s)), ..Default::default() });
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
            let kinds: Vec<(EnemyKind, u8)> = [Some((def.mini, 1u8)), def.boss.map(|(b, _)| (b, 2u8)), Some((def.final_boss, 4u8))].into_iter().flatten().collect();
            let mut bx = r.x + r.w - 14.0 - kinds.len() as f32 * 24.0;
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
    kit::panel_ex(px, py, pw, 720.0 - py - 12.0, PANEL, Color::new(1.0, 1.0, 1.0, 0.18));
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
    let tw = px + pw - 210.0 - tx;
    kit::text_fit(&mods, tx, py + 30.0, tw, 15.0, kit::INK);
    let reward = match (d, def.relic) {
        (Difficulty::Nightmare, Some(r)) => format!("First clear: +{} Sen and the relic {} ({})", d.first_clear_sen(), r.name(), r.describe()),
        _ => format!("First clear: +{} Sen{}", d.first_clear_sen(), if d == Difficulty::Normal && sid != StageId::Shotengai { ", opens the next world" } else { "" }),
    };
    kit::text_fit(&reward, tx, py + 50.0, tw, 14.0, GOLD);
    let twist = if def.mechanic == sim::campaign::Mechanic::None {
        def.blurb.to_string()
    } else {
        format!("{}  ·  {}: {}", def.blurb, def.mechanic.name(), def.mechanic.describe())
    };
    let lines = kit::wrap(&twist, pw - 230.0, 15);
    let ty = if profile.difficulty == Difficulty::Nightmare && open_asc > 0 { py + 106.0 } else { py + 80.0 };
    for (k, l) in lines.iter().take(2).enumerate() {
        kit::text(l, px + 16.0, ty + k as f32 * 19.0, 15.0, DIM);
    }
    let mut action = None;
    if button(Rect::new(px + pw - 196.0, py + 14.0, 180.0, 48.0), "Choose hero", profile.stage_open(sid)) || is_key_pressed(KeyCode::Enter) {
        action = Some(StageAction::Go);
    }
    if button(Rect::new(px + pw - 196.0, py + 70.0, 180.0, 36.0), "Back", true) || is_key_pressed(KeyCode::Escape) {
        action = Some(StageAction::Back);
    }
    action
}
