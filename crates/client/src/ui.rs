//! Screen-space UI: HUD, level-up cards, shrine, pause menu, title and results screens.

use macroquad::prelude::*;
use sim::content::{
    awakened_name, complement, hero_def, passive_max, EnemyKind, ShrineEvent, SpecialKind, Upgrade, AWAKEN_FROM, DAWN_LIMIT, MAX_LEVEL,
    MIDBOSS_AT, RUN_SECONDS, SHRINE_TIMES,
};
use sim::World;

use crate::kit::{self, Style, DIM, FAINT, GOLD, INK, JADE, SAKURA, SKY, VERMILION};
use crate::render::Assets;

fn fmt_time(t: f32) -> String {
    let s = t.max(0.0) as u32;
    format!("{:02}:{:02}", s / 60, s % 60)
}

fn centered(text: &str, y: f32, size: f32, color: Color) {
    kit::centered(text, screen_width() / 2.0, y, size, color);
}

fn dim_screen(a: f32) {
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.02, 0.01, 0.05, a));
}

/// Word-wraps `text` into lines that fit `width`.
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

/// Hero head in a round frame (the top of the idle sprite).
fn portrait(assets: &Assets, world: &World, cx: f32, cy: f32, r: f32) {
    let form = world.form_kind().is_some();
    draw_circle(cx, cy, r, Color::new(0.10, 0.08, 0.18, 1.0));
    let tex = assets.portrait(world.cfg.hero, form);
    let s = r * 2.0 / 18.0; // show the top 18 rows (the head)
    draw_texture_ex(
        tex,
        cx - 12.0 * s,
        cy - r + 1.0,
        WHITE,
        DrawTextureParams { source: Some(Rect::new(0.0, 0.0, 24.0, 18.0)), dest_size: Some(vec2(24.0 * s, 18.0 * s)), ..Default::default() },
    );
}

// ───────────────────────── HUD ─────────────────────────

pub fn hud(world: &World, assets: &Assets) {
    let w = screen_width();
    let p = &world.player;
    let hero = hero_def(world.cfg.hero);
    let t = get_time() as f32;

    // XP bar across the top with a level badge.
    let frac = world.player.xp as f32 / world.xp_to_next() as f32;
    kit::bar(8.0, 6.0, w - 16.0, 10.0, frac, SKY, Color::new(0.05, 0.04, 0.10, 0.9));

    // Hero card, top-left: portrait, HP, special meter.
    kit::panel(12.0, 24.0, 300.0, 78.0);
    portrait(assets, world, 50.0, 63.0, 30.0);
    draw_circle(72.0, 88.0, 13.0, Color::new(0.08, 0.06, 0.14, 1.0));
    draw_circle_lines(72.0, 88.0, 13.0, 2.0, SKY);
    let lv = format!("{}", p.level);
    let d = kit::measure(&lv, 15);
    kit::text(&lv, 72.0 - d.width / 2.0, 93.0, 15.0, INK);
    kit::bar(92.0, 38.0, 206.0, 16.0, p.hp / p.max_hp, Color::new(0.95, 0.30, 0.42, 1.0), Color::new(0.18, 0.06, 0.10, 0.95));
    kit::text(&format!("{:.0} / {:.0}", p.hp, p.max_hp), 100.0, 51.0, 13.0, INK);
    match world.form_kind() {
        Some(kind) => {
            let f = p.form / kind.duration();
            let flick = 0.8 + 0.2 * (t * 14.0).sin();
            let c = match kind {
                SpecialKind::Hannya | SpecialKind::Mukade => Color::new(0.9 * flick, 0.1, 0.15, 1.0),
                _ => Color::new(1.0, 0.6 * flick, 0.15, 1.0),
            };
            kit::bar(92.0, 62.0, 206.0, 10.0, f, c, Color::new(0.1, 0.08, 0.16, 0.95));
            kit::text(&format!("{}  {:.0}s", hero.special_name.to_uppercase(), p.form.ceil()), 94.0, 90.0, 14.0, c);
        }
        None => {
            let full = p.special >= 1.0;
            let glow = if full { 0.75 + 0.25 * (t * 6.0).sin() } else { 1.0 };
            kit::bar(92.0, 62.0, 206.0, 10.0, p.special, Color::new(glow, 0.85 * glow, 0.45 * glow, 1.0), Color::new(0.1, 0.08, 0.16, 0.95));
            let label = if full { format!("{}  READY  [Space]", hero.special_name.to_uppercase()) } else { hero.special_name.to_uppercase() };
            kit::text(&label, 94.0, 90.0, 14.0, if full { Color::new(1.0, 0.9, 0.5, glow) } else { FAINT });
        }
    }

    // Loadout: weapon and passive slots (VS-style), under the hero card.
    let s = 34.0;
    for k in 0..hero.weapon_slots {
        let x = 14.0 + k as f32 * (s + 6.0);
        match world.weapons.get(k) {
            Some(wp) => kit::icon_slot(x, 110.0, s, Some(assets.icon(Upgrade::Weapon(wp.kind))), wp.level, MAX_LEVEL, wp.awakened),
            None => kit::icon_slot(x, 110.0, s, None, 0, 0, false),
        }
    }
    let ps = 26.0;
    for k in 0..hero.passive_slots {
        let x = 14.0 + k as f32 * (ps + 5.0);
        match world.passives.get(k) {
            Some((pk, l)) => kit::icon_slot(x, 156.0, ps, Some(assets.icon(Upgrade::Passive(*pk))), *l, passive_max(*pk), false),
            None => kit::icon_slot(x, 156.0, ps, None, 0, 0, false),
        }
    }

    // Timer card, top-centre.
    let (tw, tx) = (220.0, (w - 220.0) / 2.0);
    kit::panel(tx, 22.0, tw, 62.0);
    let night = world.time >= RUN_SECONDS;
    centered(&fmt_time(world.time), 60.0, 36.0, if night { Color::new(1.0, 0.55, 0.45, 1.0) } else { INK });
    let sub = if !night { format!("Night Lord in {}", fmt_time(RUN_SECONDS - world.time)) } else { format!("dawn in {}", fmt_time(DAWN_LIMIT - world.time)) };
    centered(&sub, 77.0, 13.0, if night { Color::new(1.0, 0.6, 0.5, 0.8 + 0.2 * (t * 4.0).sin()) } else { DIM });

    // Boss bar.
    if let Some((_, frac, kind)) = world.boss() {
        let (bw, bx) = (460.0, (w - 460.0) / 2.0);
        let name = if kind == EnemyKind::NightLord { "NIGHT LORD  ·  CHOCHIN DAIMYO" } else { "GREAT ONI" };
        kit::panel_ex(bx - 10.0, 90.0, bw + 20.0, 40.0, Color::new(0.12, 0.03, 0.06, 0.92), Color::new(1.0, 0.35, 0.35, 0.5));
        centered(name, 106.0, 14.0, Color::new(1.0, 0.75, 0.75, 1.0));
        kit::bar(bx, 112.0, bw, 10.0, frac, Color::new(0.9, 0.22, 0.28, 1.0), Color::new(0.2, 0.04, 0.08, 1.0));
    }

    // Stat chips, top-right.
    let sen = format!("{}", world.sen);
    let kills = format!("{}", world.kills);
    let cw = kit::measure(&sen, 20).width + kit::measure("SEN", 14).width + 28.0;
    let kw = kit::measure(&kills, 20).width + kit::measure("KILLS", 14).width + 28.0;
    kit::chip(w - 14.0 - kw, 24.0, "KILLS", &kills, INK);
    let x = w - 14.0 - cw;
    kit::chip(x, 60.0, "SEN", &sen, GOLD);
    draw_texture_ex(&assets.coin, x - 22.0, 66.0, WHITE, DrawTextureParams { dest_size: Some(vec2(18.0, 18.0)), ..Default::default() });
}

// ───────────────────────── Level-up ─────────────────────────

/// Draws the level-up cards; returns the index of a card that was clicked or picked with 1/2/3.
pub fn level_up(world: &World, offer: &[Upgrade], assets: &Assets) -> Option<usize> {
    dim_screen(0.62);
    let (w, h) = (screen_width(), screen_height());
    let t = get_time() as f32;
    // Header ribbon.
    kit::panel_ex(w / 2.0 - 220.0, h * 0.13, 440.0, 74.0, Color::new(0.22, 0.07, 0.12, 0.95), GOLD);
    centered("LEVEL UP!", h * 0.13 + 44.0, 40.0, GOLD);
    centered(&format!("Level {}  ·  choose one", world.player.level), h * 0.13 + 66.0, 15.0, DIM);

    let (cw, ch, gap) = (290.0, 250.0, 26.0);
    let total = offer.len() as f32 * cw + (offer.len() as f32 - 1.0) * gap;
    let x0 = (w - total) / 2.0;
    let y0 = h * 0.30;
    let mouse: Vec2 = mouse_position().into();
    let mut picked = None;
    for (i, &u) in offer.iter().enumerate() {
        let x = x0 + i as f32 * (cw + gap);
        let r = Rect::new(x, y0, cw, ch);
        let hover = r.contains(mouse);
        let y = y0 + if hover { -6.0 } else { 0.0 };
        let lvl = world.next_level(u);
        let (accent, tag) = match u {
            Upgrade::Weapon(_) if lvl == 1 => (SAKURA, "NEW WEAPON".to_string()),
            Upgrade::Passive(_) if lvl == 1 => (SKY, "NEW PASSIVE".to_string()),
            Upgrade::Weapon(_) => (SAKURA, format!("LEVEL {lvl}")),
            Upgrade::Passive(_) => (SKY, format!("LEVEL {lvl}")),
            Upgrade::Heal => (JADE, "FOOD".to_string()),
        };
        if hover {
            kit::rounded(x - 6.0, y - 6.0, cw + 12.0, ch + 12.0, 16.0, Color::new(accent.r, accent.g, accent.b, 0.15 + 0.05 * (t * 6.0).sin()));
        }
        kit::panel_ex(x, y, cw, ch, if hover { kit::PANEL_HI } else { kit::PANEL }, if hover { accent } else { Color::new(accent.r, accent.g, accent.b, 0.45) });
        // Accent stripe along the top.
        kit::rounded(x + 14.0, y + 10.0, cw - 28.0, 4.0, 2.0, Color::new(accent.r, accent.g, accent.b, 0.8));
        // Key badge.
        draw_circle(x + cw - 22.0, y + 32.0, 13.0, Color::new(0.05, 0.04, 0.1, 1.0));
        draw_circle_lines(x + cw - 22.0, y + 32.0, 13.0, 1.5, FAINT);
        kit::text(&format!("{}", i + 1), x + cw - 27.0, y + 38.0, 16.0, DIM);
        // Icon + name + tag chip.
        let max = u.max_level();
        kit::icon_slot(x + 16.0, y + 24.0, 60.0, Some(assets.icon(u)), lvl.saturating_sub(1), if u == Upgrade::Heal { 0 } else { max }, false);
        kit::text(u.name(), x + 90.0, y + 50.0, 22.0, INK);
        let tw = kit::measure(&tag, 13).width + 16.0;
        kit::rounded(x + 90.0, y + 60.0, tw, 20.0, 10.0, Color::new(accent.r, accent.g, accent.b, 0.22));
        kit::text(&tag, x + 98.0, y + 75.0, 13.0, accent);
        // Description.
        let mut ly = y + 116.0;
        for line in wrap(&u.describe(lvl), cw - 32.0, 16) {
            kit::text(&line, x + 16.0, ly, 16.0, Color::new(0.9, 0.9, 0.96, 1.0));
            ly += 21.0;
        }
        // Awakening recipe footer.
        if let Upgrade::Weapon(k) = u {
            let (pk, need) = complement(k);
            let owned = world.passive_level(pk) >= need;
            draw_line(x + 16.0, y + ch - 46.0, x + cw - 16.0, y + ch - 46.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.08));
            draw_texture_ex(assets.icon(Upgrade::Passive(pk)), x + 16.0, y + ch - 36.0, if owned { WHITE } else { Color::new(1.0, 1.0, 1.0, 0.45) }, DrawTextureParams { dest_size: Some(vec2(22.0, 22.0)), ..Default::default() });
            let txt = format!("Awakens with {}{}", Upgrade::Passive(pk).name(), if owned { "  (owned)" } else { "" });
            kit::text(&txt, x + 46.0, y + ch - 19.0, 14.0, if owned { JADE } else { FAINT });
        }
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

// ───────────────────────── Shrine ─────────────────────────

/// Shrine choice. Returns Some(Some(i)) for a pick, Some(None) to walk away, None while undecided.
pub fn shrine(offer: &[ShrineEvent], assets: &Assets) -> Option<Option<usize>> {
    let (w, h) = (screen_width(), screen_height());
    dim_screen(0.72);
    kit::panel_ex(w / 2.0 - 250.0, h * 0.12, 500.0, 84.0, Color::new(0.18, 0.10, 0.04, 0.95), GOLD);
    draw_texture_ex(&assets.shrine, w / 2.0 - 232.0, h * 0.12 + 14.0, WHITE, DrawTextureParams { dest_size: Some(vec2(48.0, 54.0)), ..Default::default() });
    centered("A ROADSIDE SHRINE", h * 0.12 + 46.0, 36.0, GOLD);
    centered("Every blessing has a price.", h * 0.12 + 70.0, 15.0, DIM);
    let (cw, ch, gap) = (340.0, 210.0, 30.0);
    let x0 = (w - (offer.len() as f32 * cw + (offer.len() as f32 - 1.0) * gap)) / 2.0;
    let y0 = h * 0.32;
    let mouse: Vec2 = mouse_position().into();
    let mut picked = None;
    for (i, ev) in offer.iter().enumerate() {
        let x = x0 + i as f32 * (cw + gap);
        let hover = Rect::new(x, y0, cw, ch).contains(mouse);
        let y = y0 + if hover { -6.0 } else { 0.0 };
        kit::panel_ex(x, y, cw, ch, if hover { Color::new(0.20, 0.13, 0.08, 0.97) } else { kit::PANEL }, if hover { GOLD } else { kit::EDGE });
        kit::text(&format!("{}", i + 1), x + cw - 28.0, y + 34.0, 18.0, FAINT);
        kit::text(ev.name(), x + 20.0, y + 44.0, 28.0, INK);
        let (bless, price) = ev.text();
        kit::rounded(x + 16.0, y + 62.0, cw - 32.0, 58.0, 8.0, Color::new(0.1, 0.3, 0.18, 0.5));
        kit::text("BLESSING", x + 28.0, y + 82.0, 13.0, JADE);
        kit::text(bless, x + 28.0, y + 106.0, 17.0, Color::new(0.7, 1.0, 0.8, 1.0));
        kit::rounded(x + 16.0, y + 130.0, cw - 32.0, 58.0, 8.0, Color::new(0.35, 0.08, 0.1, 0.5));
        kit::text("PRICE", x + 28.0, y + 150.0, 13.0, VERMILION);
        kit::text(price, x + 28.0, y + 174.0, 17.0, Color::new(1.0, 0.7, 0.7, 1.0));
        if hover && is_mouse_button_pressed(MouseButton::Left) {
            picked = Some(Some(i));
        }
    }
    if kit::button(Rect::new((w - 220.0) / 2.0, y0 + ch + 30.0, 220.0, 46.0), "Walk away", true, Style::Secondary) || is_key_pressed(KeyCode::Escape) {
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
    let (w, h) = (screen_width(), screen_height());
    let d = kit::measure(text, 30);
    let bw = d.width + 80.0;
    let y = h * 0.22;
    kit::panel_ex((w - bw) / 2.0, y - 36.0, bw, 52.0, Color::new(0.10, 0.05, 0.08, 0.85 * a), Color::new(1.0, 0.82, 0.42, 0.6 * a));
    centered(text, y, 30.0, Color::new(1.0, 0.86, 0.5, a));
}

// ───────────────────────── Title / loading ─────────────────────────

pub fn loading() {
    clear_background(Color::from_rgba(20, 16, 32, 255));
    centered("Tuning the shamisen...", screen_height() * 0.5, 26.0, DIM);
}

pub fn title(time: f64) {
    clear_background(Color::from_rgba(20, 16, 32, 255));
    let (w, h) = (screen_width(), screen_height());
    // A big soft moon behind the logo.
    let t = time as f32;
    draw_circle(w / 2.0, h * 0.36, 150.0, Color::new(1.0, 0.85, 0.6, 0.06));
    draw_circle(w / 2.0, h * 0.36, 110.0, Color::new(1.0, 0.85, 0.6, 0.07));
    centered("KIZUNA SURVIVORS", h * 0.40, 80.0, SAKURA);
    centered("Heroes of many stories. One night. Defeat the Night Lord before dawn.", h * 0.40 + 36.0, 18.0, DIM);
    let a = 0.55 + 0.45 * (t * 3.0).sin();
    centered("Click or press Enter", h * 0.70, 26.0, Color::new(1.0, 1.0, 1.0, a));
}

// ───────────────────────── Pause menu ─────────────────────────

pub enum PauseAction {
    Resume,
    Restart,
    GiveUp,
    /// Volume changed (save it).
    Volume,
}

/// Small 0..=1 slider. Returns true if changed.
fn mini_slider(label: &str, x: f32, y: f32, w: f32, v: &mut f32) -> bool {
    kit::text(label, x, y + 14.0, 15.0, DIM);
    let (bx, bw) = (x + 120.0, w - 170.0);
    kit::bar(bx, y + 4.0, bw, 10.0, *v, SAKURA, Color::new(1.0, 1.0, 1.0, 0.1));
    draw_circle(bx + bw * *v, y + 9.0, 8.0, INK);
    let before = *v;
    let r = Rect::new(bx - 6.0, y - 4.0, bw + 12.0, 26.0);
    if r.contains(mouse_position().into()) && is_mouse_button_down(MouseButton::Left) {
        *v = ((mouse_position().0 - bx) / bw).clamp(0.0, 1.0);
    }
    *v = (*v * 20.0).round() / 20.0;
    kit::text(&format!("{:.0}%", *v * 100.0), bx + bw + 12.0, y + 14.0, 15.0, DIM);
    (*v - before).abs() > 1e-6
}

pub fn pause(world: &World, assets: &Assets, music: &mut f32, sfx: &mut f32) -> Option<PauseAction> {
    dim_screen(0.7);
    let (w, h) = (screen_width(), screen_height());
    let hero = hero_def(world.cfg.hero);
    centered("PAUSED", 64.0, 44.0, INK);

    let (lx, top, lw, panel_h) = (w * 0.5 - 600.0, 90.0, 560.0, h - 120.0);
    let (rx, rw) = (lx + lw + 20.0, 620.0);

    // ── Left: the build ──
    kit::panel(lx, top, lw, panel_h);
    portrait(assets, world, lx + 44.0, top + 46.0, 28.0);
    kit::text(hero.name, lx + 84.0, top + 42.0, 24.0, INK);
    kit::text(&format!("{}  ·  Level {}", hero.archetype, world.player.level), lx + 84.0, top + 64.0, 15.0, DIM);
    kit::panel_title(&format!("WEAPONS  {}/{}", world.weapons.len(), hero.weapon_slots), lx + 18.0, top + 104.0, 16.0, GOLD);
    let mut y = top + 116.0;
    for wp in &world.weapons {
        kit::icon_slot(lx + 18.0, y, 40.0, Some(assets.icon(Upgrade::Weapon(wp.kind))), wp.level, MAX_LEVEL, wp.awakened);
        let (name, c) = if wp.awakened { (awakened_name(wp.kind).to_string(), GOLD) } else { (Upgrade::Weapon(wp.kind).name().to_string(), INK) };
        kit::text(&name, lx + 70.0, y + 18.0, 18.0, c);
        let (pk, need) = complement(wp.kind);
        if wp.awakened {
            kit::text(&format!("AWAKENED  ·  {}", sim::content::signature_name(wp.kind)), lx + 70.0, y + 38.0, 13.0, GOLD);
        } else {
            let has = world.passive_level(pk) >= need;
            let lvl_ok = wp.level >= MAX_LEVEL;
            draw_texture_ex(assets.icon(Upgrade::Passive(pk)), lx + 70.0, y + 25.0, if has { WHITE } else { Color::new(1.0, 1.0, 1.0, 0.4) }, DrawTextureParams { dest_size: Some(vec2(16.0, 16.0)), ..Default::default() });
            let hint = format!(
                "Lv {}/{}  ·  Awaken: {}{}{}",
                wp.level,
                MAX_LEVEL,
                Upgrade::Passive(pk).name(),
                if has { " (owned)" } else { "" },
                if lvl_ok && has && world.time < AWAKEN_FROM { " · chest after 8:00" } else if lvl_ok && has { " · open a chest!" } else { "" }
            );
            kit::text(&hint, lx + 92.0, y + 38.0, 13.0, if has && lvl_ok { JADE } else { FAINT });
        }
        y += 54.0;
    }
    y += 10.0;
    kit::panel_title(&format!("PASSIVES  {}/{}", world.passives.len(), hero.passive_slots), lx + 18.0, y, 16.0, SKY);
    y += 12.0;
    for (k, (pk, l)) in world.passives.iter().enumerate() {
        let (col, row) = (k % 2, k / 2);
        let (px, py) = (lx + 18.0 + col as f32 * 270.0, y + row as f32 * 44.0);
        kit::icon_slot(px, py, 32.0, Some(assets.icon(Upgrade::Passive(*pk))), *l, passive_max(*pk), false);
        kit::text(Upgrade::Passive(*pk).name(), px + 42.0, py + 15.0, 16.0, INK);
        kit::text(&format!("Lv {}/{}", l, passive_max(*pk)), px + 42.0, py + 32.0, 13.0, DIM);
    }
    if !world.shrines_taken.is_empty() {
        let sy = top + panel_h - 46.0;
        kit::text("Shrine blessings:", lx + 18.0, sy + 18.0, 14.0, DIM);
        let names: Vec<&str> = world.shrines_taken.iter().map(|e| e.name()).collect();
        kit::text(&names.join(" · "), lx + 140.0, sy + 18.0, 14.0, GOLD);
    }

    // ── Right: night progress, stats, actions ──
    kit::panel(rx, top, rw, 200.0);
    kit::panel_title("THE NIGHT SO FAR", rx + 18.0, top + 32.0, 16.0, GOLD);
    // Timeline 0 -> dawn with markers.
    let (tx, tw2, ty) = (rx + 30.0, rw - 60.0, top + 70.0);
    let at = |s: f32| tx + tw2 * (s / DAWN_LIMIT).min(1.0);
    kit::bar(tx, ty, tw2, 10.0, world.time / DAWN_LIMIT, Color::new(0.55, 0.45, 0.95, 1.0), Color::new(1.0, 1.0, 1.0, 0.08));
    let mark = |x: f32, label: &str, c: Color, up: bool| {
        draw_line(x, ty - 4.0, x, ty + 14.0, 2.0, c);
        let d = kit::measure(label, 12);
        kit::text(label, x - d.width / 2.0, if up { ty - 8.0 } else { ty + 30.0 }, 12.0, c);
    };
    for st in SHRINE_TIMES {
        mark(at(st), "shrine", Color::new(1.0, 0.85, 0.5, 0.6), true);
    }
    mark(at(MIDBOSS_AT), "Great Oni", Color::new(1.0, 0.5, 0.5, 0.9), false);
    mark(at(RUN_SECONDS), "Night Lord", Color::new(1.0, 0.35, 0.35, 1.0), false);
    mark(at(DAWN_LIMIT), "dawn", Color::new(1.0, 0.9, 0.6, 1.0), false);
    draw_poly(at(world.time), ty + 5.0, 4, 8.0, 45.0, INK);
    // Stat chips.
    let mut cx = rx + 18.0;
    let cy = top + 132.0;
    for (label, value, c) in [
        ("TIME", fmt_time(world.time), INK),
        ("LEVEL", world.player.level.to_string(), SKY),
        ("KILLS", world.kills.to_string(), INK),
        ("SEN", world.sen.to_string(), GOLD),
        ("HP", format!("{:.0}/{:.0}", world.player.hp, world.player.max_hp), Color::new(1.0, 0.5, 0.6, 1.0)),
    ] {
        cx += kit::chip(cx, cy, label, &value, c) + 8.0;
    }

    // Actions.
    let by = top + 220.0;
    kit::panel(rx, by, rw, panel_h - 220.0);
    let mut action = None;
    let bw = rw - 60.0;
    if kit::button(Rect::new(rx + 30.0, by + 24.0, bw, 52.0), "Resume", true, Style::Primary) {
        action = Some(PauseAction::Resume);
    }
    if kit::button(Rect::new(rx + 30.0, by + 88.0, bw / 2.0 - 8.0, 46.0), "Restart", true, Style::Secondary) {
        action = Some(PauseAction::Restart);
    }
    if kit::button(Rect::new(rx + 30.0 + bw / 2.0 + 8.0, by + 88.0, bw / 2.0 - 8.0, 46.0), "Give up", true, Style::Danger) {
        action = Some(PauseAction::GiveUp);
    }
    kit::panel_title("SOUND", rx + 18.0, by + 172.0, 16.0, GOLD);
    let mut changed = mini_slider("Music", rx + 30.0, by + 186.0, rw - 60.0, music);
    changed |= mini_slider("Effects", rx + 30.0, by + 218.0, rw - 60.0, sfx);
    kit::text("Esc resume  ·  R restart  ·  Q give up  ·  N damage numbers", rx + 30.0, by + panel_h - 240.0, 13.0, FAINT);
    if changed && action.is_none() {
        action = Some(PauseAction::Volume);
    }
    action
}

// ───────────────────────── Results ─────────────────────────

pub enum ResultsAction {
    Retry,
    Menu,
}

pub fn results(world: &World, won: bool, report: Option<&sim::meta::RunReport>, lines: &crate::voice::Lines, assets: &Assets) -> Option<ResultsAction> {
    dim_screen(0.75);
    let (w, h) = (screen_width(), screen_height());
    let hero = hero_def(world.cfg.hero);
    let who = hero.name.split(' ').next().unwrap_or("");
    let (title, color, line) = if won {
        ("DAWN BREAKS", GOLD, lines.won)
    } else if world.dawn_out {
        ("DAWN WITHOUT VICTORY", Color::new(1.0, 0.6, 0.4, 1.0), lines.dawn)
    } else {
        ("FALLEN", SAKURA, lines.fallen)
    };
    let (pw, ph) = (760.0, 520.0);
    let (px, py) = ((w - pw) / 2.0, (h - ph) / 2.0);
    kit::panel(px, py, pw, ph);
    centered(title, py + 66.0, 52.0, color);
    centered(&format!("{who}: \"{line}\""), py + 98.0, 17.0, DIM);

    // Stat chips row.
    let stats = [
        ("SURVIVED", fmt_time(world.time), INK),
        ("LEVEL", world.player.level.to_string(), SKY),
        ("DEFEATED", world.kills.to_string(), INK),
    ];
    let widths: Vec<f32> = stats.iter().map(|(l, v, _)| kit::measure(l, 14).width + kit::measure(v, 20).width + 28.0).collect();
    let total: f32 = widths.iter().sum::<f32>() + 12.0 * (stats.len() as f32 - 1.0);
    let mut cx = (w - total) / 2.0;
    for ((l, v, c), wd) in stats.iter().zip(&widths) {
        kit::chip(cx, py + 124.0, l, v, *c);
        cx += wd + 12.0;
    }

    // Build row.
    let n = world.weapons.len() + world.passives.len();
    let s = 40.0;
    let mut bx = (w - (n as f32 * (s + 8.0) - 8.0)) / 2.0;
    for wp in &world.weapons {
        kit::icon_slot(bx, py + 178.0, s, Some(assets.icon(Upgrade::Weapon(wp.kind))), wp.level, MAX_LEVEL, wp.awakened);
        bx += s + 8.0;
    }
    for (pk, l) in &world.passives {
        kit::icon_slot(bx, py + 178.0, s, Some(assets.icon(Upgrade::Passive(*pk))), *l, passive_max(*pk), false);
        bx += s + 8.0;
    }

    // Rewards.
    if let Some(r) = report {
        let txt = format!("+{} Sen", r.sen);
        let d = kit::measure(&txt, 34);
        let x = (w - d.width) / 2.0 + 16.0;
        draw_texture_ex(&assets.coin, x - 36.0, py + 252.0, WHITE, DrawTextureParams { dest_size: Some(vec2(28.0, 28.0)), ..Default::default() });
        kit::text_shadow(&txt, x, py + 276.0, 34.0, GOLD);
        let mut y = py + 316.0;
        if r.discovered > 0 {
            centered(&format!("{} new Collection entr{}", r.discovered, if r.discovered == 1 { "y" } else { "ies" }), y, 17.0, SKY);
            y += 28.0;
        }
        for hero in &r.unlocked {
            let pulse = 0.75 + 0.25 * (get_time() * 4.0).sin() as f32;
            centered(&format!("NEW HERO UNLOCKED: {}", hero_def(*hero).name), y, 22.0, Color::new(1.0, 0.6, 0.85, pulse));
            y += 30.0;
        }
    }

    let mut action = None;
    let by = py + ph - 76.0;
    if kit::button(Rect::new(w / 2.0 - 250.0, by, 240.0, 52.0), "Try again", true, Style::Primary) || is_key_pressed(KeyCode::R) {
        action = Some(ResultsAction::Retry);
    }
    if kit::button(Rect::new(w / 2.0 + 10.0, by, 240.0, 52.0), "Main menu", true, Style::Secondary) || is_key_pressed(KeyCode::M) || is_key_pressed(KeyCode::Escape) {
        action = Some(ResultsAction::Menu);
    }
    action
}

pub fn debug_overlay(fps: i32, sim_ms: f32, world: &World, stress: bool) {
    let txt = format!(
        "{fps} fps · sim {sim_ms:.2} ms · {} enemies · {} proj{}",
        world.enemies.len(),
        world.projectiles.len(),
        if stress { " · STRESS (F1)  [ ] enemies  - = projectiles" } else { "" }
    );
    kit::text(&txt, 16.0, screen_height() - 12.0, 13.0, Color::new(1.0, 1.0, 1.0, 0.4));
}
