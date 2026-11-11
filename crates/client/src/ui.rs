//! Screen-space UI: HUD, level-up cards, shrine, pause menu, title and results screens.

use macroquad::prelude::*;
#[allow(unused_imports)]
use crate::kit::{draw_circle, draw_circle_lines};
use sim::content::{
    awakened_name, complement, hero_def, passive_max, Element, EnemyKind, ShrineEvent, SpecialKind, Tier, Upgrade, AWAKEN_FROM, MAX_LEVEL,
};
use sim::campaign::{Headline, StageDef, MID_BOSS_ROUND, ROUNDS, SUB_BOSS_ROUND};
use sim::World;

use crate::kit::{self, mouse_position, NavInput, screen_height, screen_width, Style, DIM, FAINT, GOLD, INK, JADE, SAKURA, SKY, VERMILION};
use crate::render::Assets;

pub fn tier_color(t: Tier) -> Color {
    use sim::gear::Rarity;
    crate::armory::rarity_color(match t {
        Tier::Common => Rarity::Common,
        Tier::Uncommon => Rarity::Uncommon,
        Tier::Rare => Rarity::Rare,
        Tier::Legendary => Rarity::Legendary,
        Tier::Mythic => Rarity::Mythic,
        Tier::Unique => Rarity::Exotic,
    })
}

pub fn element_color(e: Element) -> Color {
    match e {
        Element::Fire => Color::new(1.0, 0.45, 0.2, 1.0),
        Element::Water => Color::new(0.2, 0.45, 0.95, 1.0),
        Element::Electric => Color::new(1.0, 0.9, 0.3, 1.0),
        Element::Earth => Color::new(0.75, 0.55, 0.3, 1.0),
        Element::Wind => Color::new(0.7, 0.95, 0.3, 1.0),
        Element::Ice => Color::new(0.7, 0.95, 1.0, 1.0),
        Element::Light => Color::new(1.0, 0.88, 0.55, 1.0),
        Element::Darkness => Color::new(0.55, 0.35, 0.85, 1.0),
    }
}

const MIN_ELEMENT_SIZE: f32 = 14.0;

/// Rarity and element labels grow to 16 on a phone's short screen, where 14 reads as ~7 CSS px.
fn readable(size: f32) -> f32 {
    if macroquad::window::screen_height() < 560.0 { size.max(16.0) } else { size }
}

fn matchups(e: Element) -> (Option<Element>, Option<Element>) {
    let strong = match e {
        Element::Light => Some(Element::Darkness),
        Element::Darkness => Some(Element::Light),
        _ => e.beats(),
    };
    (strong, sim::content::ALL_ELEMENTS.into_iter().find(|w| w.beats() == Some(e)))
}

/// Diamond marker in the element colour, centred on (x, y).
pub fn element_pip(e: Element, x: f32, y: f32, r: f32) {
    draw_poly(x, y, 4, r, 0.0, element_color(e));
}

pub fn element_tag_width(e: Element, size: f32) -> f32 {
    let size = readable(size.max(MIN_ELEMENT_SIZE));
    size * 0.6 + 6.0 + kit::measure(e.name(), size as u16).width
}

/// Pip then name in the element colour; `y` is the text baseline. Returns the width drawn.
pub fn element_tag(e: Element, x: f32, y: f32, size: f32) -> f32 {
    let size = readable(size.max(MIN_ELEMENT_SIZE));
    element_tag_at(e, x, y, size);
    element_tag_width(e, size)
}

fn element_tag_at(e: Element, x: f32, y: f32, size: f32) {
    let r = size * 0.3;
    element_pip(e, x + r, y - size * 0.32, r);
    kit::text(e.name(), x + size * 0.6 + 6.0, y, size, element_color(e));
}

fn affinity_percent() -> f32 {
    ((sim::content::AFFINITY_DAMAGE - 1.0) * 100.0).round()
}

pub fn affinity_bonus_text() -> String {
    format!("+{}% damage", affinity_percent())
}

/// "Strong vs ◆Earth · Weak vs ◆Ice", each name behind its pip. Returns the width drawn.
fn matchup_line(e: Element, x: f32, y: f32, size: f32) -> f32 {
    let size = readable(size);
    let (strong, weak) = matchups(e);
    let mut cx = x;
    let mut part = |label: &str, el: Element| {
        kit::text(label, cx, y, size, DIM);
        cx += kit::measure(label, size as u16).width + 5.0;
        cx += element_tag(el, cx, y, size) + 2.0;
    };
    if let Some(st) = strong {
        part("Strong vs", st);
    }
    if let Some(w) = weak {
        part(if strong.is_some() { "·  Weak vs" } else { "Weak vs" }, w);
    }
    cx - x
}

pub fn special_color(kind: SpecialKind) -> Color {
    match kind {
        SpecialKind::Hannya | SpecialKind::Mukade => Color::new(0.9, 0.1, 0.15, 1.0),
        SpecialKind::Firmament => Color::new(0.45, 0.8, 1.0, 1.0),
        SpecialKind::Divine => Color::new(1.0, 0.75, 0.45, 1.0),
        _ => Color::new(1.0, 0.6, 0.15, 1.0),
    }
}

#[derive(Default)]
pub struct UltAlert {
    fired: bool,
    pub t: f32,
}

impl UltAlert {
    /// True on the frame the alert fires: once per ready period, never behind an overlay.
    pub fn update(&mut self, ready: bool, overlay_open: bool, dt: f32) -> bool {
        if !ready {
            *self = Self::default();
            return false;
        }
        if overlay_open {
            return false;
        }
        if self.fired {
            self.t += dt;
            return false;
        }
        self.fired = true;
        self.t = 0.0;
        true
    }
}

fn edge_glow(color: Color, strength: f32) {
    let (w, h) = (screen_width(), screen_height());
    for i in 0..4 {
        let depth = 10.0 + 22.0 * i as f32;
        let c = Color::new(color.r, color.g, color.b, strength * 0.22 * (1.0 - i as f32 / 4.0));
        draw_rectangle(0.0, 0.0, w, depth, c);
        draw_rectangle(0.0, h - depth, w, depth, c);
        draw_rectangle(0.0, 0.0, depth, h, c);
        draw_rectangle(w - depth, 0.0, depth, h, c);
    }
}

pub fn ult_alert(alert: &UltAlert, color: Color, touch: bool, ready: bool) {
    if !ready || !alert.fired {
        return;
    }
    let t = alert.t;
    if t < 1.5 {
        let pulse = (t / 0.5 * std::f32::consts::PI).sin().abs();
        edge_glow(color, pulse);
    } else {
        edge_glow(color, 0.35 * ((t - 1.5) / 0.3).min(1.0));
    }
    if t < 2.0 {
        let fade = if t > 1.6 { (2.0 - t) / 0.4 } else { 1.0 };
        let (w, y) = (screen_width(), screen_height() * 0.30);
        let c = Color::new(color.r, color.g, color.b, fade);
        kit::text_fit_centered("ALERT! — ULTIMATE READY", w / 2.0, y, w - 2.0 * kit::MARGIN, 34.0, c);
        if touch {
            kit::text_fit_centered("Tap SPECIAL", w / 2.0, y + 32.0, w - 2.0 * kit::MARGIN, 22.0, Color::new(INK.r, INK.g, INK.b, fade));
        }
    }
}

/// Portrait screens scale the canvas to a sliver; ask for landscape. Draws over everything; a run in progress is paused by the main loop, which also draws no widgets under it.
pub fn rotate_prompt() {
    let (w, h) = (screen_width(), screen_height());
    if h <= w {
        return;
    }
    dim_screen(0.92);
    let (cx, cy) = (w / 2.0, h / 2.0);
    // A phone outline turning from upright to sideways.
    let k = ((get_time() as f32 * 1.2).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let rot = -k * std::f32::consts::FRAC_PI_2;
    let (hw, hh) = (60.0, 100.0);
    let corners = [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)].map(|(x, y)| vec2(cx + x * rot.cos() - y * rot.sin(), cy - 220.0 + x * rot.sin() + y * rot.cos()));
    for i in 0..4 {
        let (a, b) = (corners[i], corners[(i + 1) % 4]);
        draw_line(a.x, a.y, b.x, b.y, 6.0, INK);
    }
    let width = w - 2.0 * kit::MARGIN;
    kit::text_fit_centered("Rotate your device", cx, cy + 30.0, width, 72.0, INK);
    kit::text_fit_centered("Kizuna Survivors plays in landscape", cx, cy + 90.0, width, 40.0, DIM);
}

fn mouse_moved() -> bool {
    mouse_delta_position() != Vec2::ZERO
}

/// Always mm:ss; menu::fmt_time switches to hours.
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
        DrawTextureParams { source: Some(Rect::new(0.0, 0.0, 24.0 * crate::sprites::UP, 18.0 * crate::sprites::UP)), dest_size: Some(vec2(24.0 * s, 18.0 * s)), ..Default::default() },
    );
}

// ───────────────────────── HUD ─────────────────────────

/// `touch`: on-screen controls are showing, so skip keyboard hints.
pub fn hud(world: &World, assets: &Assets, touch: bool) {
    let w = screen_width();
    let p = &world.player;
    let hero = hero_def(world.cfg.hero);
    let t = get_time() as f32;
    // Everything sits at least MARGIN from the edges; the XP bar is the outermost piece.
    let (ox, oy) = (kit::MARGIN - 8.0, kit::MARGIN - 6.0);

    // XP bar across the top with a level badge.
    let frac = world.player.xp as f32 / world.xp_to_next() as f32;
    kit::bar(kit::MARGIN, kit::MARGIN, w - 2.0 * kit::MARGIN, 10.0, frac, SKY, Color::new(0.05, 0.04, 0.10, 0.9));

    // Hero card, top-left: portrait, HP, special meter.
    kit::panel(12.0 + ox, 24.0 + oy, 300.0, 78.0);
    portrait(assets, world, 50.0 + ox, 63.0 + oy, 30.0);
    draw_circle(72.0 + ox, 88.0 + oy, 13.0, Color::new(0.08, 0.06, 0.14, 1.0));
    draw_circle_lines(72.0 + ox, 88.0 + oy, 13.0, 2.0, SKY);
    let lv = format!("{}", p.level);
    let d = kit::measure(&lv, 15);
    kit::text(&lv, 72.0 + ox - d.width / 2.0, 93.0 + oy, 15.0, INK);
    kit::bar(92.0 + ox, 38.0 + oy, 206.0, 16.0, p.hp / p.max_hp, Color::new(0.95, 0.30, 0.42, 1.0), Color::new(0.18, 0.06, 0.10, 0.95));
    kit::text(&format!("{:.0} / {:.0}", p.hp, p.max_hp), 100.0 + ox, 51.0 + oy, 13.0, INK);
    match world.form_kind() {
        Some(kind) => {
            let f = p.form / kind.duration();
            let flick = 0.8 + 0.2 * (t * 14.0).sin();
            let base = special_color(kind);
            let c = match kind {
                SpecialKind::Hannya | SpecialKind::Mukade => Color::new(base.r * flick, base.g, base.b, 1.0),
                _ => Color::new(base.r, base.g * flick, base.b, 1.0),
            };
            kit::bar(92.0 + ox, 62.0 + oy, 206.0, 10.0, f, c, Color::new(0.1, 0.08, 0.16, 0.95));
            let label = format!("{}  {:.0}s", hero.special_name.to_uppercase(), p.form.ceil());
            kit::text(&label, 94.0 + ox, 90.0 + oy, kit::fit(&label, 210.0, 14.0), c);
        }
        None => {
            let full = p.special >= 1.0;
            let glow = if full { 0.75 + 0.25 * (t * 6.0).sin() } else { 1.0 };
            kit::bar(92.0 + ox, 62.0 + oy, 206.0, 10.0, p.special, Color::new(glow, 0.85 * glow, 0.45 * glow, 1.0), Color::new(0.1, 0.08, 0.16, 0.95));
            let label = hero.special_name.to_uppercase();
            let ready = Color::new(1.0, 0.9, 0.5, glow);
            let room = if touch { 210.0 } else { 210.0 - keycap(298.0 + ox, 78.0 + oy, full, glow) - 6.0 };
            kit::text(&label, 94.0 + ox, 90.0 + oy, kit::fit(&label, room, 14.0), if full { ready } else { FAINT });
            if full && !touch {
                let prompt = format!("Press SPACE: {}", hero.special_name);
                let py = screen_height() - kit::MARGIN - ROUND_STRIP_H - 34.0;
                let pw = kit::measure(&prompt, 20).width.min(w - 2.0 * kit::MARGIN - 24.0) + 24.0;
                kit::rounded(w / 2.0 - pw / 2.0, py - 20.0, pw, 28.0, 14.0, Color::new(0.06, 0.05, 0.11, 0.7));
                kit::text_fit_centered(&prompt, w / 2.0, py, pw - 24.0, 20.0, ready);
            }
        }
    }

    // Loadout: weapon and passive slots (VS-style), under the hero card.
    let s = 34.0;
    let slotted: Vec<_> = world.weapons.iter().filter(|w| !w.extra).collect();
    let (lx, ly) = (14.0 + ox, 110.0 + oy);
    for k in 0..hero.weapon_slots {
        let x = lx + k as f32 * (s + 6.0);
        match slotted.get(k) {
            Some(wp) => {
                kit::icon_slot(x, ly, s, Some(assets.icon(Upgrade::Weapon(wp.kind))), wp.level, MAX_LEVEL, wp.awakened);
                weapon_pip(wp.kind, x, ly, s);
            }
            None => kit::icon_slot(x, ly, s, None, 0, 0, false),
        }
    }
    // The Arsenal weapon sits apart: it takes no slot.
    let ax = lx + hero.weapon_slots as f32 * (s + 6.0) + 8.0;
    if let Some(wp) = world.weapons.iter().find(|w| w.extra) {
        kit::rounded_outline(ax - 3.0, ly - 3.0, s + 6.0, s + 6.0, 8.0, 1.5, SKY);
        kit::icon_slot(ax, ly, s, Some(assets.icon(Upgrade::Weapon(wp.kind))), wp.level, MAX_LEVEL, wp.awakened);
        weapon_pip(wp.kind, ax, ly, s);
    } else if let Some(a) = &world.art {
        kit::rounded_outline(ax - 3.0, ly - 3.0, s + 6.0, s + 6.0, 8.0, 1.5, GOLD);
        kit::icon_slot(ax, ly, s, Some(assets.gear_icon_named(a.art.key())), 0, 0, true);
    }
    let ps = 26.0;
    for k in 0..hero.passive_slots {
        let x = lx + k as f32 * (ps + 5.0);
        match world.passives.get(k) {
            Some((pk, l)) => kit::icon_slot(x, 156.0 + oy, ps, Some(assets.icon(Upgrade::Passive(*pk))), *l, passive_max(*pk), false),
            None => kit::icon_slot(x, 156.0 + oy, ps, None, 0, 0, false),
        }
    }

    // Active pickup buffs: icon + countdown bar.
    let mut bx = lx;
    for (kind, left, total) in [
        (sim::content::PickupKind::SwiftGeta, world.speed_buff, sim::content::BUFF_SECONDS),
        (sim::content::PickupKind::OniSake, world.damage_buff, sim::content::BUFF_SECONDS),
        (sim::content::PickupKind::Hourglass, world.freeze, sim::content::FREEZE_SECONDS),
        (sim::content::PickupKind::ShrineBell, world.vacuum, sim::content::VACUUM_SECONDS),
    ] {
        if left > 0.0 {
            let col = crate::render::pickup_color(kind);
            kit::rounded(bx, 192.0 + oy, 46.0, 40.0, 8.0, Color::new(0.06, 0.05, 0.11, 0.88));
            kit::rounded_outline(bx, 192.0 + oy, 46.0, 40.0, 8.0, 1.5, col);
            draw_texture_ex(assets.pickup(kind), bx + 4.0, 196.0 + oy, WHITE, DrawTextureParams { dest_size: Some(vec2(20.0, 20.0)), ..Default::default() });
            kit::text(&format!("{:.0}", left.ceil()), bx + 27.0, 212.0 + oy, 14.0, INK);
            kit::bar(bx + 4.0, 222.0 + oy, 38.0, 5.0, left / total, col, Color::new(1.0, 1.0, 1.0, 0.1));
            bx += 52.0;
        }
    }

    // Timer card, top-centre.
    let (tw, tx) = (220.0, (w - 220.0) / 2.0);
    kit::panel(tx, 22.0 + oy, tw, 62.0);
    let st = world.stage();
    let night = world.time >= st.final_at;
    centered(&fmt_time(world.time), 60.0 + oy, 36.0, if night { Color::new(1.0, 0.55, 0.45, 1.0) } else { INK });
    let sub = if !night { format!("{} in {}", st.final_boss.short_name(), fmt_time(st.final_at - world.time)) } else { format!("dawn in {}", fmt_time(st.limit - world.time)) };
    kit::text_fit_centered(&sub, w / 2.0, 77.0 + oy, tw - 16.0, 13.0, if night { Color::new(1.0, 0.6, 0.5, 0.8 + 0.2 * (t * 4.0).sin()) } else { DIM });
    stage_progress(world, w / 2.0);

    // Boss bar, below the timer.
    if let Some((_, frac, kind)) = world.boss() {
        let (bw, bx) = (460.0, (w - 460.0) / 2.0);
        let by = 14.0 + oy;
        let name = match kind {
            EnemyKind::NightLord => "NIGHT LORD  ·  CHOCHIN DAIMYO".to_string(),
            k => k.name().to_uppercase(),
        };
        kit::panel_ex(bx - 10.0, 90.0 + by, bw + 20.0, BOSS_PANEL_BOTTOM - 90.0 - by, Color::new(0.12, 0.03, 0.06, 0.92), Color::new(1.0, 0.35, 0.35, 0.5));
        let name_c = Color::new(1.0, 0.75, 0.75, 1.0);
        let r = kind.rarity();
        let prefix = enemy_rarity_color(r).map(|c| (format!("{}  ·  ", r.name().to_uppercase()), c));
        let fs = readable(14.0);
        let prefix_w = prefix.as_ref().map_or(0.0, |(p, _)| kit::measure(p, fs as u16).width);
        let tag_w = element_tag_width(kind.element(), fs) + 10.0;
        let name_w = kit::measure(&name, fs as u16).width.min(bw - tag_w - prefix_w);
        let nx = w / 2.0 - (prefix_w + name_w + tag_w) / 2.0;
        if let Some((p, c)) = &prefix {
            kit::text(p, nx, 106.0 + by, fs, *c);
        }
        kit::text_fit(&name, nx + prefix_w, 106.0 + by, name_w, fs, name_c);
        element_tag(kind.element(), nx + prefix_w + name_w + 10.0, 106.0 + by, fs);
        // Phase pips for bosses that fight in phases.
        let phases = sim::content::boss_phases(kind, world.cfg.difficulty.extra_phases());
        if kind != EnemyKind::NightLord && kind != EnemyKind::GreatOni && phases > 1 {
            for k in 1..phases {
                let x = bx + bw * (1.0 - k as f32 / phases as f32);
                draw_line(x, 111.0 + by, x, 123.0 + by, 2.0, Color::new(1.0, 0.9, 0.7, 0.8));
            }
        }
        kit::bar(bx, 112.0 + by, bw, 10.0, frac, Color::new(0.9, 0.22, 0.28, 1.0), Color::new(0.2, 0.04, 0.08, 1.0));
    }

    // Stat chips, top-right.
    let sen = format!("{}", world.sen);
    let kills = format!("{}", world.kills);
    let cw = kit::measure(&sen, 20).width + kit::measure("SEN", 14).width + 28.0;
    let kw = kit::measure(&kills, 20).width + kit::measure("KILLS", 14).width + 28.0;
    kit::chip(w - kit::MARGIN - kw, 24.0 + oy, "KILLS", &kills, INK);
    let x = w - kit::MARGIN - cw;
    kit::chip(x, 60.0 + oy, "SEN", &sen, GOLD);
    draw_texture_ex(&assets.coin, x - 22.0, 66.0 + oy, WHITE, DrawTextureParams { dest_size: Some(vec2(18.0, 18.0)), ..Default::default() });
}

/// Element pip at a weapon slot's top-right corner; the full tag shows in the pause menu.
fn weapon_pip(kind: sim::content::WeaponKind, x: f32, y: f32, s: f32) {
    element_pip(kind.element(), x + s - 7.0, y + 7.0, 4.5);
}

/// A SPACE key badge ending at `right`; dim while charging, pulsing gold when ready. Returns its width.
fn keycap(right: f32, y: f32, ready: bool, glow: f32) -> f32 {
    let kw = kit::measure("SPACE", 11).width + 12.0;
    let x = right - kw;
    let edge = if ready { Color::new(1.0, 0.85, 0.45, glow) } else { Color::new(1.0, 1.0, 1.0, 0.3) };
    kit::rounded(x, y, kw, 17.0, 4.0, Color::new(0.1, 0.08, 0.16, 0.95));
    kit::rounded_outline(x, y, kw, 17.0, 4.0, 1.5, edge);
    kit::text("SPACE", x + 6.0, y + 13.0, 11.0, if ready { edge } else { DIM });
    kw
}

pub fn headline_color(h: Headline) -> Color {
    match h {
        Headline::Uncommon => JADE,
        Headline::Elite => SKY,
        Headline::Leader => Color::new(0.72, 0.5, 1.0, 1.0),
        Headline::SubBoss => Color::new(1.0, 0.62, 0.25, 1.0),
        Headline::Boss => Color::new(1.0, 0.35, 0.35, 1.0),
    }
}

pub fn headline_name(h: Headline) -> &'static str {
    match h {
        Headline::Uncommon => "UNCOMMON WAVE",
        Headline::Elite => "ELITE",
        Headline::Leader => "LEADER",
        Headline::SubBoss => "SUB-BOSS",
        Headline::Boss => "BOSS",
    }
}

const BOSS_PANEL_BOTTOM: f32 = 144.0 + 18.0;
/// Height of the round strip, so the HUD above it can stay clear.
const ROUND_STRIP_H: f32 = 30.0;

/// The night as 20 round segments along the bottom edge, with the round count and the next headline above.
fn stage_progress(world: &World, cx: f32) {
    let st = world.stage();
    let round = world.round();
    let bw = 460.0_f32.min(screen_width() - 2.0 * kit::MARGIN);
    let x0 = cx - bw / 2.0;
    let y = screen_height() - kit::MARGIN - ROUND_STRIP_H;
    kit::panel(x0, y, bw, ROUND_STRIP_H);

    let label = format!("Round {round} / {ROUNDS}");
    label_backing(x0, y - 7.0, kit::measure(&label, 14).width);
    kit::text(&label, x0 + 2.0, y - 7.0, 14.0, INK);
    let next = (round..=ROUNDS).find_map(|r| StageDef::headline(r).map(|h| (r, h)));
    let (note, note_color) = match next {
        _ if round == ROUNDS => (format!("dawn in {}", fmt_time((st.limit - world.time).max(0.0))), Color::new(1.0, 0.6, 0.5, 1.0)),
        Some((r, h)) if r == round => (format!("{} now", title_case(headline_name(h))), headline_color(h)),
        Some((r, h)) => {
            let n = r - round;
            (format!("{} in {n} round{}", title_case(headline_name(h)), if n == 1 { "" } else { "s" }), headline_color(h))
        }
        None => (String::new(), DIM),
    };
    let nw = kit::measure(&note, 14).width;
    if !note.is_empty() {
        label_backing(x0 + bw - nw - 4.0, y - 7.0, nw);
    }
    kit::text(&note, x0 + bw - nw - 2.0, y - 7.0, 14.0, note_color);

    let (pad, gap) = (8.0, 2.0);
    let sw = (bw - 2.0 * pad - gap * (ROUNDS - 1) as f32) / ROUNDS as f32;
    for r in 1..=ROUNDS {
        let sx = x0 + pad + (r - 1) as f32 * (sw + gap);
        let sy = y + 6.0;
        let tint = StageDef::headline(r).map(headline_color);
        let fill = if r < round {
            1.0
        } else if r > round {
            0.0
        } else if r == ROUNDS {
            ((st.limit - world.time) / (st.limit - st.final_at)).clamp(0.0, 1.0)
        } else {
            ((world.time - st.round_start(r)) / st.round_len()).clamp(0.0, 1.0)
        };
        let back = tint.map_or(Color::new(0.2, 0.18, 0.3, 0.9), |c| Color::new(c.r, c.g, c.b, 0.22));
        draw_rectangle(sx, sy, sw, 14.0, back);
        draw_rectangle(sx, sy, sw * fill, 14.0, tint.unwrap_or(SKY));
        if let Some(c) = tint {
            draw_rectangle_lines(sx, sy, sw, 14.0, 2.0, c);
        }
        if st.shrine_rounds.contains(&r) {
            draw_circle(sx + sw / 2.0, y + 25.0, 2.0, GOLD);
        }
    }
}

/// Translucent chip behind a 14 px label whose baseline is `baseline`, so world art does not run through the text.
fn label_backing(x: f32, baseline: f32, text_w: f32) {
    kit::rounded(x - 4.0, baseline - 15.0, text_w + 12.0, 20.0, 8.0, Color::new(0.06, 0.05, 0.11, 0.7));
}

fn title_case(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or_else(String::new, |f| f.to_string() + &c.as_str().to_lowercase())
}

// ───────────────────────── Level-up ─────────────────────────

/// Draws the level-up cards; returns the card taken by click, digit, or focus plus Enter/Space.
pub fn level_up(world: &World, offer: &[Upgrade], assets: &Assets, focus: &mut usize, touch: bool) -> Option<usize> {
    dim_screen(0.62);
    let (w, h) = (screen_width(), screen_height());
    let t = get_time() as f32;
    // Header ribbon.
    kit::panel_ex(w / 2.0 - 220.0, h * 0.13, 440.0, 74.0, Color::new(0.22, 0.07, 0.12, 0.95), GOLD);
    centered("LEVEL UP!", h * 0.13 + 44.0, 40.0, GOLD);
    let hint = if touch { "choose one" } else { "Arrows choose  ·  Enter take" };
    centered(&format!("Level {}  ·  {hint}", world.player.level), h * 0.13 + 66.0, 15.0, DIM);

    let (ch, gap) = (250.0, 26.0);
    // Phone labels are bigger, so the cards widen to keep the matchup line on one row.
    let cw = if readable(14.0) > 14.0 { ((w - 2.0 * kit::MARGIN - 2.0 * gap) / 3.0).min(340.0) } else { 290.0 };
    let total = offer.len() as f32 * cw + (offer.len() as f32 - 1.0) * gap;
    let x0 = (w - total) / 2.0;
    let y0 = h * 0.30;
    let mouse: Vec2 = mouse_position().into();
    let hovered = (0..offer.len()).find(|&i| Rect::new(x0 + i as f32 * (cw + gap), y0, cw, ch).contains(mouse));
    if let Some(i) = hovered.filter(|_| mouse_moved() || is_mouse_button_pressed(MouseButton::Left)) {
        *focus = i;
    }
    let (f, key_pick) = kit::nav(*focus, offer.len(), &NavInput::read());
    *focus = f;
    let mut picked = key_pick;
    for (i, &u) in offer.iter().enumerate() {
        let x = x0 + i as f32 * (cw + gap);
        let focused = i == *focus;
        let y = y0 + if focused { -6.0 } else { 0.0 };
        let lvl = world.next_level(u);
        let (accent, kind_tag) = match u {
            Upgrade::Weapon(_) if lvl == 1 => (SAKURA, "NEW WEAPON".to_string()),
            Upgrade::Passive(_) if lvl == 1 => (SKY, "NEW PASSIVE".to_string()),
            Upgrade::Weapon(_) => (SAKURA, format!("LEVEL {lvl}")),
            Upgrade::Passive(_) => (SKY, format!("LEVEL {lvl}")),
            Upgrade::Heal => (JADE, "FOOD".to_string()),
            Upgrade::Sen => (JADE, "SEN".to_string()),
            Upgrade::Blessing => (JADE, "BLESSING".to_string()),
        };
        let tier_name = u.tier().name().to_uppercase();
        let frame = tier_color(u.tier());
        let tag = if world.is_replacement(u) { format!("REPLACE · {tier_name}") } else { format!("{tier_name} · {kind_tag}") };
        if focused {
            kit::rounded(x - 6.0, y - 6.0, cw + 12.0, ch + 12.0, 16.0, Color::new(accent.r, accent.g, accent.b, 0.15 + 0.05 * (t * 6.0).sin()));
        }
        kit::panel_ex(x, y, cw, ch, if focused { kit::PANEL_HI } else { kit::PANEL }, if focused { frame } else { Color::new(frame.r, frame.g, frame.b, 0.6) });
        if focused {
            kit::focus_ring(x - 3.0, y - 3.0, cw + 6.0, ch + 6.0, accent);
        }
        kit::rounded(x + 14.0, y + 10.0, cw - 28.0, 4.0, 2.0, Color::new(frame.r, frame.g, frame.b, 0.8));
        // Key badge.
        draw_circle(x + cw - 22.0, y + 32.0, 13.0, Color::new(0.05, 0.04, 0.1, 1.0));
        draw_circle_lines(x + cw - 22.0, y + 32.0, 13.0, 1.5, FAINT);
        kit::text(&format!("{}", i + 1), x + cw - 27.0, y + 38.0, 16.0, DIM);
        // Icon + name + tag chip.
        let max = u.max_level();
        kit::icon_slot(x + 16.0, y + 24.0, 60.0, Some(assets.icon(u)), lvl.saturating_sub(1), if sim::content::MAXED_FALLBACKS.contains(&u) { 0 } else { max }, false);
        kit::text_fit(u.name(), x + 90.0, y + 50.0, cw - 140.0, 22.0, INK);
        let ps = readable(13.0);
        let tw = (kit::measure(&tag, ps as u16).width + 16.0).min(cw - 104.0);
        kit::rounded(x + 90.0, y + 60.0, tw, 20.0, 10.0, Color::new(frame.r, frame.g, frame.b, 0.22));
        kit::text_fit(&tag, x + 98.0, y + 75.0, tw - 16.0, ps, frame);
        let mut ly = y + 116.0;
        if let Upgrade::Weapon(k) = u {
            let e = k.element();
            element_tag(e, x + 90.0, y + 102.0, 14.0);
            matchup_line(e, x + 16.0, y + 126.0, 14.0);
            ly = y + 148.0;
        }
        for line in kit::wrap(&world.describe_upgrade(u, lvl), cw - 32.0, 16) {
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
    }
    if let Some(i) = hovered.filter(|_| is_mouse_button_pressed(MouseButton::Left)) {
        picked = Some(i);
    }
    picked
}

/// Replace-skill picker. Returns Some(Some(j)) to overwrite `replace_targets()[j]`, Some(None) to cancel, None while undecided.
pub fn replace_picker(world: &World, assets: &Assets, focus: &mut usize, touch: bool) -> Option<Option<usize>> {
    let Some(new) = world.replacing else { return Some(None) };
    let targets = world.replace_targets();
    let n = targets.len();
    dim_screen(0.72);
    let (w, h) = (screen_width(), screen_height());
    let (pw, row_h, gap) = if touch { (520.0, 70.0, 10.0) } else { (520.0, 50.0, 6.0) };
    let head_h = 92.0;
    let ph = head_h + (n + 1) as f32 * (row_h + gap) + 8.0;
    let (px, py) = ((w - pw) / 2.0, ((h - ph) / 2.0).max(kit::MARGIN));
    kit::panel_ex(px, py, pw, ph, Color::new(0.14, 0.07, 0.1, 0.96), tier_color(new.tier()));
    centered("Replace which skill?", py + 38.0, 30.0, GOLD);

    let row = |i: usize| Rect::new(px + 16.0, py + head_h + i as f32 * (row_h + gap), pw - 32.0, row_h);
    let mouse: Vec2 = mouse_position().into();
    let hovered = (0..=n).find(|&i| row(i).contains(mouse));
    if let Some(i) = hovered.filter(|_| mouse_moved() || is_mouse_button_pressed(MouseButton::Left)) {
        *focus = i;
    }
    let mut input = NavInput::read();
    input.digit = input.digit.filter(|&d| d < n);
    let (f, key_pick) = kit::nav(*focus, n + 1, &input);
    *focus = f;
    let mut picked = key_pick.map(|i| (i < n).then_some(i));
    for (i, &(u, lvl)) in targets.iter().enumerate() {
        let r = row(i);
        let dy = (r.h - 50.0) / 2.0;
        let focused = i == *focus;
        let tc = tier_color(u.tier());
        kit::panel_ex(r.x, r.y, r.w, r.h, if focused { kit::PANEL_HI } else { kit::PANEL }, if focused { tc } else { Color::new(tc.r, tc.g, tc.b, 0.45) });
        if focused {
            kit::focus_ring(r.x - 3.0, r.y - 3.0, r.w + 6.0, r.h + 6.0, GOLD);
        }
        kit::icon_slot(r.x + 8.0, r.y + dy + 5.0, 40.0, Some(assets.icon(u)), 0, 0, false);
        let chip = u.tier().name().to_uppercase();
        let cwid = kit::measure(&chip, 12).width + 14.0;
        let elem_w = match u {
            Upgrade::Weapon(k) => {
                let tag_w = element_tag_width(k.element(), 14.0);
                element_tag_at(k.element(), r.x + r.w - cwid - 18.0 - tag_w, r.y + dy + 30.0, 14.0);
                tag_w + 6.0
            }
            _ => 0.0,
        };
        kit::text_fit(&format!("{}  Lv {lvl}", u.name()), r.x + 58.0, r.y + dy + 21.0, r.w - 58.0 - cwid - 52.0 - elem_w, 18.0, INK);
        kit::text(&format!("becomes Lv {}", sim::content::inherit_level(new, lvl)), r.x + 58.0, r.y + dy + 40.0, 14.0, DIM);
        kit::rounded(r.x + r.w - cwid - 12.0, r.y + dy + 15.0, cwid, 20.0, 10.0, Color::new(tc.r, tc.g, tc.b, 0.22));
        kit::text(&chip, r.x + r.w - cwid - 5.0, r.y + dy + 30.0, 12.0, tc);
        kit::text(&format!("{}", i + 1), r.x + r.w - cwid - 34.0 - elem_w, r.y + dy + 31.0, 16.0, FAINT);
    }
    let joins = match targets.get(*focus) {
        Some(&(_, lvl)) => format!("{} joins at Lv {}", new.name(), sim::content::inherit_level(new, lvl)),
        None => format!("{} stays in the offer", new.name()),
    };
    let new_tag_w = match new {
        Upgrade::Weapon(k) => element_tag_width(k.element(), 14.0) + 10.0,
        _ => 0.0,
    };
    let joins_w = pw - 40.0 - new_tag_w;
    let joins_size = kit::fit(&joins, joins_w, 16.0);
    let joins_text_w = kit::measure(&joins, joins_size as u16).width;
    let joins_x = w / 2.0 - (joins_text_w + new_tag_w) / 2.0;
    kit::text(&joins, joins_x, py + 66.0, joins_size, DIM);
    if let Upgrade::Weapon(k) = new {
        element_tag_at(k.element(), joins_x + joins_text_w + 10.0, py + 66.0, 14.0);
    }
    let cancel = row(n);
    if *focus == n {
        kit::focus_ring(cancel.x - 3.0, cancel.y - 3.0, cancel.w + 6.0, cancel.h + 6.0, GOLD);
    }
    if kit::button(cancel, "Cancel", true, Style::Secondary) || is_key_pressed(KeyCode::Escape) {
        picked = Some(None);
    }
    if let Some(i) = hovered.filter(|&i| i < n && is_mouse_button_pressed(MouseButton::Left)) {
        picked = Some(Some(i));
    }
    picked
}

// ───────────────────────── Shrine ─────────────────────────

/// Shrine choice. Returns Some(Some(i)) for a pick, Some(None) to walk away, None while undecided.
pub fn shrine(offer: &[ShrineEvent], assets: &Assets, focus: &mut usize) -> Option<Option<usize>> {
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
    let away = Rect::new((w - 220.0) / 2.0, y0 + ch + 30.0, 220.0, 46.0);
    let hovered = (0..offer.len())
        .find(|&i| Rect::new(x0 + i as f32 * (cw + gap), y0, cw, ch).contains(mouse))
        .or_else(|| away.contains(mouse).then_some(offer.len()));
    if let Some(i) = hovered.filter(|_| mouse_moved() || is_mouse_button_pressed(MouseButton::Left)) {
        *focus = i;
    }
    let mut input = NavInput::read();
    input.digit = input.digit.filter(|&d| d < offer.len());
    let (f, key_pick) = kit::nav(*focus, offer.len() + 1, &input);
    *focus = f;
    let mut picked = key_pick.map(|i| (i < offer.len()).then_some(i));
    for (i, ev) in offer.iter().enumerate() {
        let x = x0 + i as f32 * (cw + gap);
        let focused = i == *focus;
        let y = y0 + if focused { -6.0 } else { 0.0 };
        kit::panel_ex(x, y, cw, ch, if focused { Color::new(0.20, 0.13, 0.08, 0.97) } else { kit::PANEL }, if focused { GOLD } else { kit::EDGE });
        if focused {
            kit::focus_ring(x - 3.0, y - 3.0, cw + 6.0, ch + 6.0, GOLD);
        }
        kit::text(&format!("{}", i + 1), x + cw - 28.0, y + 34.0, 18.0, FAINT);
        kit::text_fit(ev.name(), x + 20.0, y + 44.0, cw - 60.0, 28.0, INK);
        let (bless, price) = ev.text();
        kit::rounded(x + 16.0, y + 62.0, cw - 32.0, 58.0, 8.0, Color::new(0.1, 0.3, 0.18, 0.5));
        kit::text("BLESSING", x + 28.0, y + 82.0, 13.0, JADE);
        kit::text_fit(bless, x + 28.0, y + 106.0, cw - 56.0, 17.0, Color::new(0.7, 1.0, 0.8, 1.0));
        kit::rounded(x + 16.0, y + 130.0, cw - 32.0, 58.0, 8.0, Color::new(0.35, 0.08, 0.1, 0.5));
        kit::text("PRICE", x + 28.0, y + 150.0, 13.0, VERMILION);
        kit::text_fit(price, x + 28.0, y + 174.0, cw - 56.0, 17.0, Color::new(1.0, 0.7, 0.7, 1.0));
    }
    if let Some(i) = hovered.filter(|&i| i < offer.len() && is_mouse_button_pressed(MouseButton::Left)) {
        picked = Some(Some(i));
    }
    if *focus == offer.len() {
        kit::focus_ring(away.x - 3.0, away.y - 3.0, away.w + 6.0, away.h + 6.0, GOLD);
    }
    if kit::button(away, "Walk away", true, Style::Secondary) || is_key_pressed(KeyCode::Escape) {
        picked = Some(None);
    }
    picked
}

/// Outline and label colour of an enemy rarity, shared with the gear tiers; Common has none.
pub fn enemy_rarity_color(r: sim::enemies::EnemyRarity) -> Option<Color> {
    use sim::enemies::EnemyRarity::*;
    use sim::gear::Rarity;
    let tier = match r {
        Common => return None,
        Uncommon => Rarity::Uncommon,
        Elite => Rarity::Rare,
        Leader => Rarity::Epic,
        SubBoss => Rarity::Legendary,
        Boss => Rarity::Mythic,
    };
    Some(crate::armory::rarity_color(tier))
}

/// `below_boss_bar` drops the banner under the boss panel so both stay readable.
pub fn banner(text: &str, left: f32, below_boss_bar: bool) {
    let a = (left / 0.5).min(1.0);
    let (w, h) = (screen_width(), screen_height());
    let text_w = kit::measure(text, 30).width.min(w - 2.0 * kit::MARGIN - 80.0);
    let bw = text_w + 80.0;
    let y = if below_boss_bar { BOSS_PANEL_BOTTOM + 6.0 + 36.0 } else { h * 0.22 };
    kit::panel_ex((w - bw) / 2.0, y - 36.0, bw, 52.0, Color::new(0.10, 0.05, 0.08, 0.85 * a), Color::new(1.0, 0.82, 0.42, 0.6 * a));
    kit::text_fit_centered(text, w / 2.0, y, text_w, 30.0, Color::new(1.0, 0.86, 0.5, a));
}

// ───────────────────────── Title / loading ─────────────────────────

pub fn loading() {
    clear_background(Color::from_rgba(20, 16, 32, 255));
    centered("Tuning the shamisen...", screen_height() * 0.5, 26.0, DIM);
}

// ───────────────────────── Pause menu ─────────────────────────

pub enum PauseAction {
    Resume,
    Restart,
    GiveUp,
    /// Volume changed (save it).
    Volume,
    Dev,
    /// Flip Auto Special on/off.
    AutoSpecial,
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

/// "<Set> set (n) · Resist ◆Earth 60% ◆Fire 20%": the worn Set bonus and the three best resistances. Nothing without gear.
fn gear_line(g: &sim::gear::Loadout, x: f32, y: f32, width: f32) {
    let set = g.sets6.first().map(|s| (*s, 6)).or_else(|| g.sets5.first().map(|s| (*s, 5))).or_else(|| g.sets4.first().map(|s| (*s, 4)));
    let mut res: Vec<(Element, f32)> = sim::content::ALL_ELEMENTS.into_iter().map(|e| (e, g.resist(e))).filter(|r| r.1 > 0.0).collect();
    res.sort_by(|a, b| b.1.total_cmp(&a.1));
    res.truncate(3);
    if set.is_none() && res.is_empty() {
        return;
    }
    let mut cx = x;
    if let Some((stage, n)) = set {
        let label = format!("{} set ({n}) · {}", sim::gear::set_name(stage), if res.is_empty() { "" } else { "Resist " });
        kit::text_fit(&label, cx, y, width, 14.0, GOLD);
        cx += kit::measure(&label, 14).width;
    }
    for (e, v) in res {
        let pct = format!("{:.0}%", v * 100.0);
        if cx + element_tag_width(e, 14.0) + kit::measure(&pct, 14).width > x + width {
            break;
        }
        cx += element_tag(e, cx, y, 14.0) + 4.0;
        kit::text(&pct, cx, y, 14.0, INK);
        cx += kit::measure(&pct, 14).width + 12.0;
    }
}

/// `auto_special` is `None` until Auto Special is bought, then whether it's on.
pub fn pause(world: &World, assets: &Assets, music: &mut f32, sfx: &mut f32, dev: bool, auto_special: Option<bool>) -> Option<PauseAction> {
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
    let aw = element_tag(hero.element, lx + 84.0, top + 84.0, 14.0);
    kit::text_fit(&affinity_bonus_text(), lx + 84.0 + aw + 8.0, top + 84.0, lw - 120.0 - aw, 14.0, element_color(hero.element));
    kit::panel_title(&format!("WEAPONS  {}/{}", world.weapons.iter().filter(|w| !w.extra).count(), hero.weapon_slots), lx + 18.0, top + 104.0, 16.0, GOLD);
    let mut y = top + 116.0;
    for wp in &world.weapons {
        kit::icon_slot(lx + 18.0, y, 40.0, Some(assets.icon(Upgrade::Weapon(wp.kind))), wp.level, MAX_LEVEL, wp.awakened);
        let (name, c) = if wp.awakened { (awakened_name(wp.kind).to_string(), GOLD) } else { (Upgrade::Weapon(wp.kind).name().to_string(), INK) };
        let name = if wp.extra { format!("{name}  (Arsenal)") } else { name };
        let el = wp.kind.element();
        let tw = element_tag_width(el, 14.0);
        kit::text_fit(&name, lx + 70.0, y + 18.0, lw - 90.0 - tw - 12.0, 18.0, c);
        element_tag(el, lx + lw - 18.0 - tw, y + 18.0, 14.0);
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
                if lvl_ok && has && world.time < AWAKEN_FROM { format!(" · chest after {}", fmt_time(AWAKEN_FROM)) } else if lvl_ok && has { " · open a chest!".to_string() } else { String::new() }
            );
            kit::text_fit(&hint, lx + 92.0, y + 38.0, lw - 110.0, 13.0, if has && lvl_ok { JADE } else { FAINT });
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
    let sy = top + panel_h - 46.0;
    gear_line(&world.cfg.gear, lx + 18.0, if world.shrines_taken.is_empty() { sy + 18.0 } else { sy - 12.0 }, lw - 36.0);
    if !world.shrines_taken.is_empty() {
        kit::text("Shrine blessings:", lx + 18.0, sy + 18.0, 14.0, DIM);
        let names: Vec<&str> = world.shrines_taken.iter().map(|e| e.name()).collect();
        kit::text_fit(&names.join(" · "), lx + 140.0, sy + 18.0, lw - 160.0, 14.0, GOLD);
    }

    // ── Right: night progress, stats, actions ──
    kit::panel(rx, top, rw, 200.0);
    kit::panel_title(&format!("{}  ·  {}", world.stage().name.to_uppercase(), world.cfg.difficulty.name().to_uppercase()), rx + 18.0, top + 32.0, 16.0, GOLD);
    // Timeline 0 -> dawn with markers.
    let (tx, tw2, ty) = (rx + 30.0, rw - 60.0, top + 70.0);
    let st = world.stage();
    let at = |s: f32| tx + tw2 * (s / st.limit).min(1.0);
    kit::bar(tx, ty, tw2, 10.0, world.time / st.limit, Color::new(0.55, 0.45, 0.95, 1.0), Color::new(1.0, 1.0, 1.0, 0.08));
    let mut low_end = f32::MIN;
    let mut mark = |x: f32, label: &str, c: Color, up: bool| {
        draw_line(x, ty - 4.0, x, ty + 14.0, 2.0, c);
        let d = kit::measure(label, 12);
        let left = x - d.width / 2.0;
        let down = !up && left >= low_end + 4.0;
        if down {
            low_end = left + d.width;
        }
        kit::text(label, left, if down { ty + 30.0 } else { ty - 8.0 }, 12.0, c);
    };
    for &r in st.shrine_rounds {
        mark(at(st.round_start(r)), "shrine", Color::new(1.0, 0.85, 0.5, 0.6), true);
    }
    mark(at(st.round_start(SUB_BOSS_ROUND)), "sub-boss", Color::new(1.0, 0.5, 0.5, 0.9), false);
    if st.boss.is_some() {
        mark(at(st.round_start(MID_BOSS_ROUND)), "boss", Color::new(1.0, 0.42, 0.42, 0.95), false);
    }
    mark(at(st.final_at), st.final_boss.short_name(), Color::new(1.0, 0.35, 0.35, 1.0), false);
    mark(at(st.limit), "dawn", Color::new(1.0, 0.9, 0.6, 1.0), false);
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
    if dev {
        crate::menu::fullscreen_button(Rect::new(rx + 30.0, by + 144.0, bw / 2.0 - 8.0, 40.0));
        if kit::button(Rect::new(rx + 30.0 + bw / 2.0 + 8.0, by + 144.0, bw / 2.0 - 8.0, 40.0), "Dev", true, Style::Danger) {
            action = Some(PauseAction::Dev);
        }
    } else {
        crate::menu::fullscreen_button(Rect::new(rx + 30.0, by + 144.0, bw, 40.0));
    }
    kit::panel_title("SOUND", rx + 18.0, by + 210.0, 16.0, GOLD);
    let mut changed = mini_slider("Music", rx + 30.0, by + 224.0, rw - 60.0, music);
    changed |= mini_slider("Effects", rx + 30.0, by + 256.0, rw - 60.0, sfx);
    // A button, not just the U key, so touch players can switch it too.
    if let Some(on) = auto_special {
        let label = format!("Auto Special: {}  [U]", if on { "ON" } else { "OFF" });
        if kit::button(Rect::new(rx + 30.0, by + 292.0, bw, 40.0), &label, true, if on { Style::Primary } else { Style::Secondary }) {
            action = Some(PauseAction::AutoSpecial);
        }
    }
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

    // Equipment found (shown from the run itself; banked into the bag at the end).
    if !world.loot.is_empty() {
        let s = 30.0;
        let n = world.loot.len().min(16);
        let mut gx = (w - (n as f32 * (s + 6.0) - 6.0)) / 2.0;
        for it in world.loot.iter().take(n) {
            crate::armory::item_cell(assets, it, gx, py + ph - 116.0, s, false, false);
            gx += s + 6.0;
        }
        let more = if world.loot.len() > n { format!("  (+{} more)", world.loot.len() - n) } else { String::new() };
        centered(&format!("{} gear found{more}", world.loot.len()), py + ph - 124.0, 14.0, DIM);
    }
    // Rewards.
    if let Some(r) = report {
        let txt = format!("+{} Sen", r.sen);
        let d = kit::measure(&txt, 34);
        let x = (w - d.width) / 2.0 + 16.0;
        draw_texture_ex(&assets.coin, x - 36.0, py + 252.0, WHITE, DrawTextureParams { dest_size: Some(vec2(28.0, 28.0)), ..Default::default() });
        kit::text_shadow(&txt, x, py + 276.0, 34.0, GOLD);
        let mut y = py + 308.0;
        // Reward lines stop above the loot label, or just above the buttons when there is none.
        let limit = if world.loot.is_empty() { py + ph - 96.0 } else { py + ph - 136.0 };
        let line = |t: &str, y: f32, size: f32, c: Color| {
            if y <= limit {
                centered(t, y, size, c);
            }
        };
        const STEP: f32 = 24.0;
        for hero in &r.unlocked {
            let pulse = 0.75 + 0.25 * (get_time() * 4.0).sin() as f32;
            line(&format!("NEW HERO UNLOCKED: {}", hero_def(*hero).name), y, 22.0, Color::new(1.0, 0.6, 0.85, pulse));
            y += STEP;
        }
        if let Some(stg) = r.stage_unlocked {
            line(&format!("NEW WORLD OPEN: {}", stg.def().name), y, 20.0, JADE);
            y += STEP;
        }
        if let Some(d) = r.first_clear {
            line(&format!("FIRST {} CLEAR  ·  +{} Sen bonus", d.name().to_uppercase(), d.first_clear_sen()), y, 20.0, GOLD);
            y += STEP;
        }
        if let Some(rel) = r.relic {
            line(&format!("RELIC EARNED: {} ({})", rel.name(), rel.describe()), y, 18.0, Color::new(0.75, 0.9, 1.0, 1.0));
            y += STEP;
        }
        if let Some(n) = r.ascension_opened {
            line(&format!("NIGHTMARE+{n} OPEN"), y, 18.0, Color::new(1.0, 0.5, 0.6, 1.0));
            y += STEP;
        }
        if r.discovered > 0 {
            line(&format!("{} new Collection entr{}", r.discovered, if r.discovered == 1 { "y" } else { "ies" }), y, 17.0, SKY);
        }
    }

    let mut action = None;
    let by = py + ph - 76.0;
    if kit::button(Rect::new(w / 2.0 - 250.0, by, 240.0, 52.0), if won { "Play again" } else { "Try again" }, true, Style::Primary) || is_key_pressed(KeyCode::R) {
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
        if stress { " · STRESS  [ ] enemies  - = projectiles" } else { "" }
    );
    kit::text(&txt, kit::MARGIN, screen_height() - kit::MARGIN, 13.0, Color::new(1.0, 1.0, 1.0, 0.4));
}

// ───────────────────────── Minimap ─────────────────────────

/// Draws the map. Small: corner panel (returns true when clicked, to expand).
/// Big: a large centred map with a legend (returns true when it should close).
pub fn minimap(world: &World, assets: &Assets, big: bool, just_opened: bool) -> bool {
    let (w, h) = (screen_width(), screen_height());
    let (size, x, y, range) = if big {
        let size = (h - 140.0).min(w - 80.0).min(620.0);
        (size, (w - size) / 2.0, 90.0, 900.0)
    } else {
        (150.0, w - kit::MARGIN - 150.0, 116.0, 650.0)
    };
    if big {
        dim_screen(0.6);
        centered(&world.stage().name.to_uppercase(), 64.0, 36.0, INK);
    }
    kit::panel(x, y, size, size);
    let g = crate::stage::ground_color(world.cfg.stage);
    kit::rounded(x + 3.0, y + 3.0, size - 6.0, size - 6.0, 10.0, Color::new(g.r, g.g, g.b, 0.55));
    let c = vec2(x + size / 2.0, y + size / 2.0);
    let scale = (size / 2.0 - 6.0) / range;
    let p = world.player.pos;
    let to_map = |q: sim::Vec2| -> Option<Vec2> {
        let d = vec2(q.x - p.x, q.y - p.y) * scale;
        let half = size / 2.0 - 5.0;
        (d.x.abs() <= half && d.y.abs() <= half).then(|| c + d)
    };
    // Range rings.
    for k in 1..=3 {
        draw_circle_lines(c.x, c.y, (size / 2.0 - 6.0) * k as f32 / 3.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.05));
    }
    // Stage obstacles, from whatever stage is active.
    for (mn, mx) in world.map_obstacles(sim::Vec2::new(p.x - range, p.y - range), sim::Vec2::new(p.x + range, p.y + range)) {
        if let (Some(a0), Some(a1)) = (to_map(mn), to_map(mx)) {
            // Props are small at map scale: give them a minimum size so they read.
            let (bw, bh) = ((a1.x - a0.x).max(3.0), (a1.y - a0.y).max(3.0));
            let (cx2, cy2) = ((a0.x + a1.x) / 2.0, (a0.y + a1.y) / 2.0);
            draw_rectangle(cx2 - bw / 2.0, cy2 - bh / 2.0, bw, bh, Color::new(0.85, 0.78, 0.65, 0.45));
        }
    }
    // Stage bounds, for bounded maps.
    if let Some((mn, mx)) = world.map_bounds() {
        let a0 = c + vec2(mn.x - p.x, mn.y - p.y) * scale;
        let a1 = c + vec2(mx.x - p.x, mx.y - p.y) * scale;
        draw_rectangle_lines(a0.x, a0.y, a1.x - a0.x, a1.y - a0.y, 2.0, Color::new(1.0, 0.85, 0.5, 0.5));
    }
    // Markers by layer: ordinary enemies, then bosses and the shrine, then drops on top.
    let pulse = 0.6 + 0.4 * (get_time() as f32 * 5.0).sin();
    // The sprite of whatever elite / boss the stage fields.
    let st = world.stage();
    let boss_kind = |q: sim::Vec2| {
        let e = &world.enemies;
        (0..e.len()).filter(|&i| e.kind[i].is_boss()).min_by(|&a, &b| (e.pos[a] - q).length_sq().partial_cmp(&(e.pos[b] - q).length_sq()).unwrap()).map(|i| e.kind[i])
    };
    let mut markers = world.map_markers();
    markers.sort_by_key(|m| match m.kind {
        sim::MarkerKind::Enemy => 0,
        sim::MarkerKind::Coin => 2,
        sim::MarkerKind::Pickup(_) => 3,
        sim::MarkerKind::Chest { .. } => 4,
        _ => 1,
    });
    let dark = Color::new(0.04, 0.03, 0.08, 0.85);
    for m in markers {
        let Some(q) = to_map(m.pos) else { continue };
        match m.kind {
            sim::MarkerKind::Enemy => {
                let d = if big { 2.5 } else { 2.0 };
                draw_rectangle(q.x - d / 2.0, q.y - d / 2.0, d, d, Color::new(1.0, 0.25, 0.3, 0.85));
            }
            sim::MarkerKind::Elite => {
                let s = if big { 16.0 } else { 11.0 };
                draw_texture_ex(assets.enemy_tex(st.elite), q.x - s / 2.0, q.y - s / 2.0, WHITE, DrawTextureParams { dest_size: Some(vec2(s, s)), ..Default::default() });
            }
            sim::MarkerKind::MiniBoss => {
                let s = if big { 24.0 } else { 16.0 };
                draw_circle(q.x, q.y, s * 0.7, Color::new(0.6, 0.4, 1.0, 0.25 + 0.2 * pulse));
                draw_circle_lines(q.x, q.y, s * 0.7, 1.5, Color::new(0.8, 0.6, 1.0, 1.0));
                draw_texture_ex(assets.enemy_tex(st.mini), q.x - s / 2.0, q.y - s / 2.0, if st.mini == EnemyKind::GreatOni { Color::new(0.85, 0.7, 1.0, 1.0) } else { WHITE }, DrawTextureParams { dest_size: Some(vec2(s, s)), ..Default::default() });
            }
            sim::MarkerKind::Boss => {
                let s = if big { 28.0 } else { 18.0 };
                draw_circle(q.x, q.y, s * 0.75, Color::new(1.0, 0.3, 0.1, 0.25 + 0.25 * pulse));
                draw_circle_lines(q.x, q.y, s * 0.75 + 2.0 * pulse, 2.0, Color::new(1.0, 0.5, 0.2, 1.0));
                let k = boss_kind(m.pos).unwrap_or(st.final_boss);
                draw_texture_ex(assets.enemy_tex(k), q.x - s / 2.0, q.y - s / 2.0, WHITE, DrawTextureParams { dest_size: Some(vec2(s, s)), ..Default::default() });
            }
            sim::MarkerKind::Chest { tier } => {
                let s = if tier >= 3 { 1.5 } else { 1.0 };
                draw_circle(q.x, q.y, 5.5 * s, dark);
                draw_rectangle(q.x - 3.0 * s, q.y - 2.5 * s, 6.0 * s, 5.0 * s, Color::new(1.0, 0.8, 0.3, pulse));
            }
            sim::MarkerKind::Shrine => {
                draw_texture_ex(&assets.shrine, q.x - 7.0, q.y - 8.0, WHITE, DrawTextureParams { dest_size: Some(vec2(14.0, 16.0)), ..Default::default() });
            }
            sim::MarkerKind::Pickup(k) => {
                let col = crate::render::pickup_color(k);
                draw_circle(q.x, q.y, if big { 10.0 } else { 6.0 }, dark);
                if big {
                    draw_texture_ex(assets.pickup(k), q.x - 8.0, q.y - 8.0, WHITE, DrawTextureParams { dest_size: Some(vec2(16.0, 16.0)), ..Default::default() });
                } else {
                    draw_circle(q.x, q.y, 3.5, col);
                    draw_circle_lines(q.x, q.y, 5.0, 1.0, Color::new(col.r, col.g, col.b, pulse));
                }
            }
            sim::MarkerKind::Coin => {
                if big {
                    draw_texture_ex(&assets.coin, q.x - 5.0, q.y - 5.0, WHITE, DrawTextureParams { dest_size: Some(vec2(10.0, 10.0)), ..Default::default() });
                } else {
                    draw_circle(q.x, q.y, 2.5, dark);
                    draw_circle(q.x, q.y, 1.5, GOLD);
                }
            }
            sim::MarkerKind::Objective => draw_poly(q.x, q.y, 5, 5.0 + 2.0 * pulse, 0.0, GOLD),
        }
    }
    // The player: an arrow pointing where they face.
    let f = world.player.facing.normalize_or_zero();
    let f = if f == sim::Vec2::ZERO { vec2(1.0, 0.0) } else { vec2(f.x, f.y) };
    let side = vec2(-f.y, f.x);
    let r = if big { 8.0 } else { 6.0 };
    draw_triangle(c + f * r, c - f * r * 0.6 + side * r * 0.7, c - f * r * 0.6 - side * r * 0.7, INK);

    let mouse: Vec2 = mouse_position().into();
    let over = Rect::new(x, y, size, size).contains(mouse);
    if big {
        // Legend.
        let ly = y + size + 26.0;
        let items: [(&str, Color); 8] = [
            ("you", INK),
            ("Otherworlders", Color::new(1.0, 0.25, 0.3, 1.0)),
            ("elite", Color::new(1.0, 0.45, 0.35, 1.0)),
            ("sub-boss", Color::new(0.8, 0.6, 1.0, 1.0)),
            ("boss", Color::new(1.0, 0.5, 0.2, 1.0)),
            ("chest", GOLD),
            ("pickup", JADE),
            ("sen", Color::new(1.0, 0.85, 0.3, 1.0)),
        ];
        let widths: Vec<f32> = items.iter().map(|(l, _)| kit::measure(l, 15).width + 40.0).collect();
        let mut lx = (w - widths.iter().sum::<f32>()) / 2.0;
        for ((label, col), wd) in items.iter().zip(&widths) {
            draw_circle(lx + 6.0, ly - 5.0, 5.0, *col);
            kit::text(label, lx + 16.0, ly, 15.0, DIM);
            lx += wd;
        }
        kit::text("M / Tab / Esc to close", w / 2.0 - 70.0, ly + 24.0, 13.0, FAINT);
        let close = kit::button(Rect::new(x + size - 40.0, y + 8.0, 32.0, 32.0), "X", true, Style::Secondary);
        // The click that opened the map must not also close it.
        let outside = !just_opened && is_mouse_button_pressed(MouseButton::Left) && !over && mouse.y > 100.0;
        close || is_key_pressed(KeyCode::Escape) || outside
    } else {
        kit::text("MAP", x + 10.0, y + 18.0, 12.0, FAINT);
        if over {
            rounded_hint(x, y, size);
        }
        over && is_mouse_button_pressed(MouseButton::Left)
    }
}

fn rounded_hint(x: f32, y: f32, size: f32) {
    kit::rounded_outline(x, y, size, size, 12.0, 2.0, GOLD);
    kit::text("click to expand", x + size - 92.0, y + size - 8.0, 12.0, DIM);
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim::content::ALL_ELEMENTS;

    fn distance(a: Color, b: Color) -> f32 {
        ((a.r - b.r).powi(2) + (a.g - b.g).powi(2) + (a.b - b.b).powi(2)).sqrt()
    }

    #[test]
    fn element_colors_are_distinct() {
        for (i, a) in ALL_ELEMENTS.iter().enumerate() {
            assert!(!a.name().is_empty());
            let ca = element_color(*a);
            for b in &ALL_ELEMENTS[i + 1..] {
                let cb = element_color(*b);
                let gap = (ca.r - cb.r).abs().max((ca.g - cb.g).abs()).max((ca.b - cb.b).abs());
                assert!(gap > 0.15, "{} and {} too close", a.name(), b.name());
            }
        }
    }

    #[test]
    fn element_colors_differ_from_rarity_and_theme() {
        use sim::gear::{Rarity, ALL_RARITIES};
        let mut others: Vec<(String, Color)> = ALL_RARITIES
            .iter()
            .filter(|&&r| r != Rarity::Exotic)
            .map(|&r| (format!("{r:?}"), crate::armory::rarity_color(r)))
            .collect();
        others.extend([("GOLD".into(), GOLD), ("JADE".into(), JADE), ("INK".into(), INK)]);
        for e in ALL_ELEMENTS {
            for (name, c) in &others {
                let d = distance(element_color(e), *c);
                assert!(d >= 0.12, "{} vs {name}: {d:.3}", e.name());
            }
        }
    }

    #[test]
    fn enemy_rarity_colors_match_gear_tiers() {
        use sim::enemies::EnemyRarity::*;
        use sim::gear::Rarity;
        assert!(enemy_rarity_color(Common).is_none());
        let tiers = [(Uncommon, Rarity::Uncommon), (Elite, Rarity::Rare), (Leader, Rarity::Epic), (SubBoss, Rarity::Legendary), (Boss, Rarity::Mythic)];
        for (i, (r, g)) in tiers.iter().enumerate() {
            let c = enemy_rarity_color(*r).expect("coloured");
            assert_eq!(c, crate::armory::rarity_color(*g), "{r:?}");
            for (o, _) in &tiers[i + 1..] {
                let d = distance(c, enemy_rarity_color(*o).unwrap());
                assert!(d >= 0.12, "{r:?} vs {o:?}: {d:.3}");
            }
        }
    }

    #[test]
    fn affinity_bonus_text_shows_percent() {
        assert_eq!(affinity_bonus_text(), "+10% damage");
    }

    #[test]
    fn alert_fires_once_per_ready_period() {
        let mut a = UltAlert::default();
        assert!(!a.update(false, false, 0.1));
        assert!(a.update(true, false, 0.1));
        assert!(!a.update(true, false, 0.1));
        assert!(!a.update(false, false, 0.1));
        assert!(a.update(true, false, 0.1));
    }

    #[test]
    fn alert_waits_for_overlay_to_close() {
        let mut a = UltAlert::default();
        assert!(!a.update(true, true, 0.1));
        assert!(!a.update(true, true, 0.1));
        assert!(a.update(true, false, 0.1));
    }

    #[test]
    fn alert_pauses_under_overlay() {
        let mut a = UltAlert::default();
        assert!(a.update(true, false, 0.1));
        a.update(true, false, 0.5);
        a.update(true, true, 1.0);
        a.update(true, true, 1.0);
        assert!((a.t - 0.5).abs() < 1e-4);
        assert!(!a.update(true, false, 0.1));
        assert!((a.t - 0.6).abs() < 1e-4);
    }

    #[test]
    fn alert_survives_pause() {
        let mut a = UltAlert::default();
        assert!(a.update(true, false, 0.1));
        assert!(!a.update(true, true, 0.1));
        assert!(!a.update(true, false, 0.1));
    }
}
