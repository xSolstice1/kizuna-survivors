//! Screen-space UI: HUD, level-up cards, title / pause / results screens.

use macroquad::prelude::*;
use sim::content::{awakened_name, complement, hero_def, ShrineEvent, Upgrade, RUN_SECONDS};
use sim::World;

use crate::render::Assets;

const PANEL: Color = Color::new(0.06, 0.05, 0.10, 0.85);
const ACCENT: Color = Color::new(1.0, 0.43, 0.63, 1.0);
const XP_BLUE: Color = Color::new(0.31, 0.82, 1.0, 1.0);

fn centered(text: &str, y: f32, size: f32, color: Color) {
    let d = measure_text(text, None, size as u16, 1.0);
    draw_text(text, (screen_width() - d.width) / 2.0, y, size, color);
}

fn fmt_time(t: f32) -> String {
    let s = t.max(0.0) as u32;
    format!("{:02}:{:02}", s / 60, s % 60)
}

pub fn hud(world: &World, assets: &Assets) {
    let w = screen_width();
    // XP bar across the top.
    let frac = world.player.xp as f32 / world.xp_to_next() as f32;
    draw_rectangle(0.0, 0.0, w, 14.0, Color::new(0.05, 0.05, 0.12, 0.9));
    draw_rectangle(0.0, 0.0, w * frac.min(1.0), 14.0, XP_BLUE);
    draw_text(&format!("LV {}", world.player.level), w - 70.0, 12.0, 18.0, WHITE);

    // Timer (counts toward dawn) + kills.
    let remaining = RUN_SECONDS - world.time;
    centered(&fmt_time(world.time), 44.0, 34.0, WHITE);
    centered(&format!("dawn in {}", fmt_time(remaining)), 64.0, 18.0, Color::new(1.0, 1.0, 1.0, 0.6));
    if let Some((_, frac)) = world.boss() {
        let (bw, bx) = (420.0, (w - 420.0) / 2.0);
        centered("GREAT ONI", 88.0, 18.0, Color::new(1.0, 0.8, 0.8, 1.0));
        draw_rectangle(bx, 94.0, bw, 10.0, Color::from_rgba(40, 10, 20, 230));
        draw_rectangle(bx, 94.0, bw * frac, 10.0, Color::from_rgba(230, 60, 70, 255));
        draw_rectangle_lines(bx, 94.0, bw, 10.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.4));
    }
    let kills = format!("{} kills", world.kills);
    let d = measure_text(&kills, None, 20, 1.0);
    draw_text(&kills, w - d.width - 16.0, 40.0, 20.0, WHITE);

    // HP + loadout, top-left.
    let p = &world.player;
    draw_rectangle(16.0, 24.0, 200.0, 14.0, Color::from_rgba(40, 20, 30, 230));
    draw_rectangle(16.0, 24.0, 200.0 * (p.hp / p.max_hp), 14.0, Color::from_rgba(255, 80, 110, 255));
    draw_text(&format!("{:.0} / {:.0}", p.hp, p.max_hp), 22.0, 36.0, 16.0, WHITE);
    // Special meter.
    let full = p.special >= 1.0;
    let glow = if full { 0.75 + 0.25 * (get_time() * 6.0).sin() as f32 } else { 1.0 };
    draw_rectangle(16.0, 42.0, 200.0, 8.0, Color::from_rgba(30, 30, 50, 230));
    draw_rectangle(16.0, 42.0, 200.0 * p.special, 8.0, Color::new(1.0 * glow, 0.85 * glow, 0.45 * glow, 1.0));
    if full {
        draw_text("PURIFY  [Space]", 222.0, 51.0, 18.0, Color::new(1.0, 0.9, 0.5, glow));
    }
    let hero = hero_def(world.cfg.hero);
    let dim = Color::new(1.0, 1.0, 1.0, 0.5);
    let mut y = 70.0;
    draw_text(&format!("WEAPONS {}/{}", world.weapons.len(), hero.weapon_slots), 16.0, y, 14.0, dim);
    y += 6.0;
    for wpn in &world.weapons {
        let label = if wpn.awakened {
            awakened_name(wpn.kind).to_string()
        } else {
            format!("{}  Lv{}", Upgrade::Weapon(wpn.kind).name(), wpn.level)
        };
        let c = if wpn.awakened { Color::new(1.0, 0.85, 0.4, 1.0) } else { Color::new(1.0, 0.85, 0.9, 1.0) };
        draw_texture(assets.icon(Upgrade::Weapon(wpn.kind)), 16.0, y, WHITE);
        draw_text(&label, 38.0, y + 13.0, 18.0, c);
        y += 19.0;
    }
    y += 12.0;
    draw_text(&format!("PASSIVES {}/{}", world.passives.len(), hero.passive_slots), 16.0, y, 14.0, dim);
    y += 6.0;
    for (k, l) in &world.passives {
        draw_texture(assets.icon(Upgrade::Passive(*k)), 16.0, y, WHITE);
        draw_text(&format!("{}  Lv{}", Upgrade::Passive(*k).name(), l), 38.0, y + 13.0, 16.0, Color::new(0.8, 0.85, 1.0, 1.0));
        y += 18.0;
    }
}

/// Draws the level-up cards; returns the index of a card that was clicked or picked with 1/2/3.
pub fn level_up(world: &World, offer: &[Upgrade], assets: &Assets) -> Option<usize> {
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.55));
    centered("LEVEL UP!", screen_height() * 0.22, 48.0, ACCENT);
    centered("Choose an upgrade  (click or press 1-3)", screen_height() * 0.22 + 30.0, 20.0, WHITE);

    let (cw, ch, gap) = (270.0, 200.0, 24.0);
    let total = offer.len() as f32 * cw + (offer.len() as f32 - 1.0) * gap;
    let x0 = (screen_width() - total) / 2.0;
    let y0 = screen_height() * 0.36;
    let (mx, my) = mouse_position();
    let mut picked = None;
    for (i, &u) in offer.iter().enumerate() {
        let x = x0 + i as f32 * (cw + gap);
        let r = Rect::new(x, y0, cw, ch);
        let hover = r.contains(vec2(mx, my));
        draw_rectangle(x, y0, cw, ch, if hover { Color::new(0.16, 0.10, 0.22, 0.97) } else { PANEL });
        draw_rectangle_lines(x, y0, cw, ch, 2.0, if hover { ACCENT } else { Color::new(1.0, 1.0, 1.0, 0.25) });
        let lvl = world.next_level(u);
        let tag = match u {
            Upgrade::Weapon(_) if lvl == 1 => "NEW WEAPON".to_string(),
            Upgrade::Passive(_) if lvl == 1 => "NEW PASSIVE".to_string(),
            Upgrade::Heal => "FOOD".to_string(),
            _ => format!("LEVEL {lvl}"),
        };
        draw_text(&format!("{}", i + 1), x + cw - 22.0, y0 + 24.0, 20.0, Color::new(1.0, 1.0, 1.0, 0.4));
        draw_rectangle(x + 12.0, y0 + 12.0, 52.0, 52.0, Color::new(1.0, 1.0, 1.0, 0.06));
        draw_texture_ex(assets.icon(u), x + 14.0, y0 + 14.0, WHITE, DrawTextureParams { dest_size: Some(vec2(48.0, 48.0)), ..Default::default() });
        draw_text(u.name(), x + 76.0, y0 + 36.0, 24.0, WHITE);
        draw_text(&tag, x + 76.0, y0 + 56.0, 16.0, ACCENT);
        if let Upgrade::Weapon(k) = u {
            let (p, need) = complement(k);
            let owned = world.passive_level(p) >= need;
            let txt = format!("Awaken with: {}{}", Upgrade::Passive(p).name(), if owned { " (owned)" } else { "" });
            let c = if owned { Color::new(0.45, 1.0, 0.6, 1.0) } else { Color::new(1.0, 1.0, 1.0, 0.45) };
            draw_text(&txt, x + 14.0, y0 + ch - 14.0, 16.0, c);
        }
        // Naive word wrap for the description.
        let mut line = String::new();
        let mut ly = y0 + 92.0;
        for word in u.describe(lvl).split_whitespace() {
            if measure_text(&format!("{line} {word}"), None, 18, 1.0).width > cw - 28.0 {
                draw_text(&line, x + 14.0, ly, 18.0, Color::new(0.9, 0.9, 0.95, 1.0));
                line.clear();
                ly += 20.0;
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        draw_text(&line, x + 14.0, ly, 18.0, Color::new(0.9, 0.9, 0.95, 1.0));
        if hover && is_mouse_button_pressed(MouseButton::Left) {
            picked = Some(i);
        }
    }
    for (i, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3].iter().enumerate() {
        if i < offer.len() && is_key_pressed(*key) {
            picked = Some(i);
        }
    }
    picked
}

/// Shrine choice. Returns Some(Some(i)) for a pick, Some(None) to walk away, None while undecided.
pub fn shrine(offer: &[ShrineEvent]) -> Option<Option<usize>> {
    let (w, h) = (screen_width(), screen_height());
    draw_rectangle(0.0, 0.0, w, h, Color::new(0.02, 0.01, 0.05, 0.75));
    centered("A ROADSIDE SHRINE", h * 0.22, 44.0, Color::new(1.0, 0.85, 0.45, 1.0));
    centered("Every blessing has a price.  (click, or press 1 / 2)", h * 0.22 + 30.0, 20.0, WHITE);
    let (cw, ch, gap) = (330.0, 190.0, 30.0);
    let x0 = (w - (offer.len() as f32 * cw + (offer.len() as f32 - 1.0) * gap)) / 2.0;
    let y0 = h * 0.36;
    let (mx, my) = mouse_position();
    let mut picked = None;
    for (i, ev) in offer.iter().enumerate() {
        let x = x0 + i as f32 * (cw + gap);
        let hover = Rect::new(x, y0, cw, ch).contains(vec2(mx, my));
        draw_rectangle(x, y0, cw, ch, if hover { Color::new(0.18, 0.12, 0.08, 0.97) } else { PANEL });
        draw_rectangle_lines(x, y0, cw, ch, 2.0, if hover { Color::new(1.0, 0.85, 0.45, 1.0) } else { Color::new(1.0, 1.0, 1.0, 0.25) });
        draw_text(&format!("{}", i + 1), x + cw - 24.0, y0 + 26.0, 20.0, Color::new(1.0, 1.0, 1.0, 0.4));
        draw_text(ev.name(), x + 18.0, y0 + 40.0, 30.0, WHITE);
        let (bless, price) = ev.text();
        draw_text("BLESSING", x + 18.0, y0 + 82.0, 16.0, Color::new(0.45, 1.0, 0.6, 0.8));
        draw_text(bless, x + 18.0, y0 + 104.0, 20.0, Color::new(0.6, 1.0, 0.7, 1.0));
        draw_text("PRICE", x + 18.0, y0 + 140.0, 16.0, Color::new(1.0, 0.5, 0.5, 0.8));
        draw_text(price, x + 18.0, y0 + 162.0, 20.0, Color::new(1.0, 0.6, 0.6, 1.0));
        if hover && is_mouse_button_pressed(MouseButton::Left) {
            picked = Some(Some(i));
        }
    }
    let leave = "Walk away  [Esc]";
    let d = measure_text(leave, None, 22, 1.0);
    let r = Rect::new((w - d.width) / 2.0 - 16.0, y0 + ch + 34.0, d.width + 32.0, 36.0);
    let hover = r.contains(vec2(mx, my));
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, Color::new(1.0, 1.0, 1.0, if hover { 0.8 } else { 0.3 }));
    draw_text(leave, r.x + 16.0, r.y + 25.0, 22.0, Color::new(1.0, 1.0, 1.0, if hover { 1.0 } else { 0.7 }));
    if (hover && is_mouse_button_pressed(MouseButton::Left)) || is_key_pressed(KeyCode::Escape) {
        picked = Some(None);
    }
    for (i, key) in [KeyCode::Key1, KeyCode::Key2].iter().enumerate() {
        if i < offer.len() && is_key_pressed(*key) {
            picked = Some(Some(i));
        }
    }
    picked
}

pub fn banner(text: &str, left: f32) {
    let a = (left / 0.5).min(1.0);
    let h = screen_height();
    // Sits under the timer and boss bar, clear of the chest/level-up titles.
    draw_rectangle(0.0, h * 0.20 - 40.0, screen_width(), 58.0, Color::new(0.05, 0.02, 0.08, 0.6 * a));
    centered(text, h * 0.20, 40.0, Color::new(1.0, 0.85, 0.45, a));
}

pub fn title(time: f64) {
    clear_background(Color::from_rgba(20, 16, 32, 255));
    let h = screen_height();
    centered("KIZUNA SURVIVORS", h * 0.36, 72.0, ACCENT);
    centered("Heroes of many stories. One night. Survive until dawn.", h * 0.36 + 30.0, 20.0, Color::new(1.0, 1.0, 1.0, 0.6));
    centered("Survive until dawn. Your talismans fire on their own.", h * 0.52, 22.0, WHITE);
    centered("WASD / arrows to move   ·   Space: special   ·   Esc to pause", h * 0.52 + 28.0, 20.0, Color::new(1.0, 1.0, 1.0, 0.7));
    let a = 0.55 + 0.45 * (time * 3.0).sin() as f32;
    centered("Click or press Enter to start", h * 0.72, 30.0, Color::new(1.0, 1.0, 1.0, a));
}

pub fn paused() {
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.6));
    centered("PAUSED", screen_height() * 0.42, 56.0, WHITE);
    centered("Esc to resume   ·   R to restart   ·   N toggle damage numbers", screen_height() * 0.42 + 40.0, 20.0, Color::new(1.0, 1.0, 1.0, 0.8));
}

pub fn results(world: &World, won: bool) {
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.7));
    let h = screen_height();
    if won {
        centered("DAWN BREAKS", h * 0.32, 64.0, Color::new(1.0, 0.85, 0.5, 1.0));
        centered("Ren: \"Entry closed. Sunrise logged.\"", h * 0.32 + 34.0, 22.0, WHITE);
    } else {
        centered("FALLEN", h * 0.32, 64.0, ACCENT);
        centered("Ren: \"...to be continued.\"", h * 0.32 + 34.0, 22.0, WHITE);
    }
    let lines = [
        format!("Survived  {}", fmt_time(world.time)),
        format!("Level  {}", world.player.level),
        format!("Otherworlders defeated  {}", world.kills),
    ];
    for (i, l) in lines.iter().enumerate() {
        centered(l, h * 0.50 + i as f32 * 30.0, 26.0, WHITE);
    }
    centered("Click or press R to try again", h * 0.75, 26.0, Color::new(1.0, 1.0, 1.0, 0.8));
}

pub fn debug_overlay(fps: i32, sim_ms: f32, world: &World, stress: bool) {
    let txt = format!(
        "{fps} fps · sim {sim_ms:.2} ms · {} enemies · {} proj{}",
        world.enemies.len(),
        world.projectiles.len(),
        if stress { " · STRESS (F1)  [ ] enemies  - = projectiles" } else { "" }
    );
    draw_text(&txt, 16.0, screen_height() - 12.0, 16.0, Color::new(1.0, 1.0, 1.0, 0.55));
}
