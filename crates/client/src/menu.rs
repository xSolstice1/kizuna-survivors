//! Out-of-run screens: main menu, character select and the Shrine Shop.

use macroquad::prelude::*;
use sim::content::{awakened_name, awakened_text, complement, hero_def, EnemyKind, Upgrade, ALL_ENEMIES, ALL_HEROES, ALL_PASSIVES, ALL_SHRINE_EVENTS, ALL_WEAPONS};
use sim::meta::{collection_progress, enemy_found, unlock_text, PowerUp, Profile, ALL_POWERUPS};

use crate::render::Assets;

const PANEL: Color = Color::new(0.06, 0.05, 0.10, 0.88);
const ACCENT: Color = Color::new(1.0, 0.43, 0.63, 1.0);
const GOLD: Color = Color::new(1.0, 0.85, 0.4, 1.0);
const DIM: Color = Color::new(1.0, 1.0, 1.0, 0.55);

fn centered(text: &str, y: f32, size: f32, color: Color) {
    let d = measure_text(text, None, size as u16, 1.0);
    draw_text(text, (screen_width() - d.width) / 2.0, y, size, color);
}

/// Draws a button; returns true when clicked.
fn button(r: Rect, label: &str, enabled: bool) -> bool {
    let hover = enabled && r.contains(mouse_position().into());
    let bg = if hover { Color::new(0.20, 0.12, 0.26, 0.97) } else { PANEL };
    draw_rectangle(r.x, r.y, r.w, r.h, bg);
    let edge = if !enabled { Color::new(1.0, 1.0, 1.0, 0.12) } else if hover { ACCENT } else { Color::new(1.0, 1.0, 1.0, 0.3) };
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, edge);
    let size = 26.0;
    let d = measure_text(label, None, size as u16, 1.0);
    let c = if enabled { WHITE } else { Color::new(1.0, 1.0, 1.0, 0.3) };
    draw_text(label, r.x + (r.w - d.width) / 2.0, r.y + r.h / 2.0 + 9.0, size, c);
    hover && is_mouse_button_pressed(MouseButton::Left)
}

fn sen_badge(assets: &Assets, sen: u32) {
    let txt = format!("{sen}");
    let d = measure_text(&txt, None, 28, 1.0);
    let x = screen_width() - d.width - 28.0;
    draw_texture_ex(&assets.coin, x - 30.0, 18.0, WHITE, DrawTextureParams { dest_size: Some(vec2(22.0, 22.0)), ..Default::default() });
    draw_text(&txt, x, 38.0, 28.0, GOLD);
}

fn backdrop(time: f64) {
    clear_background(Color::from_rgba(20, 16, 32, 255));
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
    Shop,
    Collection,
    Settings,
}

pub fn main_menu(profile: &Profile, assets: &Assets, time: f64) -> Option<MenuAction> {
    backdrop(time);
    let (w, h) = (screen_width(), screen_height());
    centered("KIZUNA SURVIVORS", h * 0.24, 72.0, ACCENT);
    centered("Heroes of many stories. One night. Defeat the Night Lord before dawn.", h * 0.24 + 32.0, 20.0, DIM);
    sen_badge(assets, profile.sen);

    let (bw, bh) = (320.0, 56.0);
    let x = (w - bw) / 2.0;
    let mut action = None;
    if button(Rect::new(x, h * 0.42, bw, bh), "Start", true) || is_key_pressed(KeyCode::Enter) {
        action = Some(MenuAction::Start);
    }
    if button(Rect::new(x, h * 0.42 + 68.0, bw, bh), "Shrine Shop", true) || is_key_pressed(KeyCode::S) {
        action = Some(MenuAction::Shop);
    }
    let (found, total) = collection_progress(profile);
    if button(Rect::new(x, h * 0.42 + 136.0, bw, bh), &format!("Collection  {found}/{total}"), true) || is_key_pressed(KeyCode::C) {
        action = Some(MenuAction::Collection);
    }
    if button(Rect::new(x, h * 0.42 + 204.0, bw, bh), "Settings", true) || is_key_pressed(KeyCode::O) {
        action = Some(MenuAction::Settings);
    }

    let unlocked = profile.unlocked.len();
    let lines = [
        format!("Nights played {}   ·   Otherworlders defeated {}", profile.runs, profile.total_kills),
        format!(
            "Longest night {:02}:{:02}   ·   Highest level {}   ·   Night Lords defeated {}   ·   Heroes {}/{}",
            profile.best_time as u32 / 60,
            profile.best_time as u32 % 60,
            profile.best_level,
            profile.night_lords,
            unlocked,
            ALL_HEROES.len()
        ),
    ];
    for (i, l) in lines.iter().enumerate() {
        centered(l, h - 70.0 + i as f32 * 26.0, 20.0, DIM);
    }
    centered("WASD / arrows to move   ·   Space: special   ·   Esc to pause", h * 0.42 + 290.0, 20.0, Color::new(1.0, 1.0, 1.0, 0.7));
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
    let (cw, ch, gap) = (150.0, 190.0, 16.0);
    let x0 = (w - (n as f32 * cw + (n as f32 - 1.0) * gap)) / 2.0;
    let y0 = 110.0;
    let mouse: Vec2 = mouse_position().into();
    for (i, hero) in ALL_HEROES.iter().enumerate() {
        let r = Rect::new(x0 + i as f32 * (cw + gap), y0, cw, ch);
        let open = profile.is_unlocked(*hero);
        let sel = *cursor == i;
        let hover = r.contains(mouse);
        draw_rectangle(r.x, r.y, r.w, r.h, if sel { Color::new(0.20, 0.12, 0.26, 0.97) } else { PANEL });
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, if sel { ACCENT } else if hover { Color::new(1.0, 1.0, 1.0, 0.5) } else { Color::new(1.0, 1.0, 1.0, 0.2) });
        let tint = if open { WHITE } else { Color::new(0.0, 0.0, 0.0, 0.85) };
        let bob = if sel && open { ((time * 4.0).sin() * 2.0) as f32 } else { 0.0 };
        draw_texture_ex(&assets.hero(*hero)[0], r.x + (cw - 72.0) / 2.0, r.y + 18.0 + bob, tint, DrawTextureParams { dest_size: Some(vec2(72.0, 96.0)), ..Default::default() });
        let def = hero_def(*hero);
        let name = if open { def.name.split(' ').next().unwrap_or(def.name) } else { "???" };
        let d = measure_text(name, None, 24, 1.0);
        draw_text(name, r.x + (cw - d.width) / 2.0, r.y + 140.0, 24.0, if open { WHITE } else { DIM });
        let d = measure_text(def.archetype, None, 16, 1.0);
        draw_text(def.archetype, r.x + (cw - d.width) / 2.0, r.y + 162.0, 16.0, DIM);
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
    let (px, py, pw, ph) = ((w - 640.0) / 2.0, y0 + ch + 24.0, 640.0, 200.0);
    draw_rectangle(px, py, pw, ph, PANEL);
    draw_rectangle_lines(px, py, pw, ph, 2.0, Color::new(1.0, 1.0, 1.0, 0.2));
    if open {
        draw_text(def.name, px + 20.0, py + 40.0, 32.0, WHITE);
        draw_text(def.archetype, px + 20.0, py + 64.0, 18.0, ACCENT);
        draw_texture_ex(assets.icon(Upgrade::Weapon(def.signature)), px + 20.0, py + 82.0, WHITE, DrawTextureParams { dest_size: Some(vec2(32.0, 32.0)), ..Default::default() });
        draw_text(&format!("Starts with {}", Upgrade::Weapon(def.signature).name()), px + 62.0, py + 104.0, 22.0, WHITE);
        draw_text(&format!("Innate: {}", def.innate_text), px + 20.0, py + 142.0, 22.0, Color::new(0.6, 1.0, 0.7, 1.0));
        if !def.finished {
            draw_text("Placeholder: own sprite and special move coming later.", px + 20.0, py + 178.0, 18.0, DIM);
        } else {
            let what = match def.special {
                sim::content::SpecialKind::Barrier => "a purifying ring that erases fireballs",
                sim::content::SpecialKind::NineTails => "8s transformation: nine tails, aura, golden orb volleys",
                sim::content::SpecialKind::Hannya => "8s berserk: beams, crescent waves, shadow steps, finale blast",
            };
            draw_text(&format!("Special: {} ({what})", def.special_name), px + 20.0, py + 178.0, 18.0, Color::new(1.0, 0.8, 0.45, 1.0));
        }
    } else {
        draw_text("LOCKED", px + 20.0, py + 44.0, 32.0, DIM);
        draw_text(&format!("{} ({})", def.archetype, "unlock to reveal"), px + 20.0, py + 72.0, 20.0, DIM);
        draw_text(&format!("Unlock: {}", unlock_text(hero)), px + 20.0, py + 120.0, 24.0, GOLD);
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
        draw_rectangle(r.x, r.y, r.w, r.h, if sel { Color::new(0.22, 0.16, 0.08, 0.97) } else { PANEL });
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, if sel { GOLD } else if hover { Color::new(1.0, 1.0, 1.0, 0.5) } else { Color::new(1.0, 1.0, 1.0, 0.2) });
        draw_texture_ex(powerup_icon(assets, *p), r.x + (cw - 40.0) / 2.0, r.y + 12.0, WHITE, DrawTextureParams { dest_size: Some(vec2(40.0, 40.0)), ..Default::default() });
        let d = measure_text(p.name(), None, 20, 1.0);
        draw_text(p.name(), r.x + (cw - d.width) / 2.0, r.y + 74.0, 20.0, WHITE);
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
    draw_rectangle(px, py, pw, ph, PANEL);
    draw_rectangle_lines(px, py, pw, ph, 2.0, Color::new(1.0, 1.0, 1.0, 0.2));
    draw_text(p.name(), px + 20.0, py + 38.0, 30.0, WHITE);
    draw_text(&format!("Rank {}/{}", profile.rank(p), p.max_rank()), px + pw - 140.0, py + 38.0, 22.0, DIM);
    draw_text(p.describe(), px + 20.0, py + 72.0, 22.0, Color::new(0.9, 0.9, 0.95, 1.0));
    let maxed = profile.rank(p) >= p.max_rank();
    let price = if maxed { "Fully blessed".to_string() } else { format!("Price: {} Sen", profile.cost(p)) };
    draw_text(&price, px + 20.0, py + 108.0, 22.0, if maxed || profile.can_buy(p) { GOLD } else { Color::new(1.0, 0.5, 0.5, 1.0) });

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
}

const TABS: [Tab; 6] = [Tab::Otherworlders, Tab::Weapons, Tab::Awakenings, Tab::Passives, Tab::Shrines, Tab::Heroes];

impl Tab {
    fn name(self) -> &'static str {
        match self {
            Tab::Otherworlders => "Otherworlders",
            Tab::Weapons => "Weapons",
            Tab::Awakenings => "Awakenings",
            Tab::Passives => "Passives",
            Tab::Shrines => "Shrine Events",
            Tab::Heroes => "Heroes",
        }
    }
    fn len(self) -> usize {
        match self {
            Tab::Otherworlders => ALL_ENEMIES.len(),
            Tab::Weapons | Tab::Awakenings => ALL_WEAPONS.len(),
            Tab::Passives => ALL_PASSIVES.len(),
            Tab::Shrines => ALL_SHRINE_EVENTS.len(),
            Tab::Heroes => ALL_HEROES.len(),
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
    match k {
        EnemyKind::Wisp => (&assets.wisp, WHITE),
        EnemyKind::Kasa => (&assets.kasa, WHITE),
        EnemyKind::Chochin => (&assets.chochin, WHITE),
        EnemyKind::Nopperabo => (&assets.nopperabo, WHITE),
        EnemyKind::Oni => (&assets.oni, WHITE),
        EnemyKind::GreatOni => (&assets.oni, Color::new(0.85, 0.7, 1.0, 1.0)),
        EnemyKind::NightLord => (&assets.night_lord, WHITE),
    }
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
                lines: vec![k.lore().into(), format!("Defeated: {kills}{}", if k.is_boss() { "   (boss)" } else { "" })],
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
    let (tw, th, tg) = (170.0, 36.0, 8.0);
    let tx0 = (w - (TABS.len() as f32 * tw + (TABS.len() as f32 - 1.0) * tg)) / 2.0;
    let mouse: Vec2 = mouse_position().into();
    for (i, t) in TABS.iter().enumerate() {
        let r = Rect::new(tx0 + i as f32 * (tw + tg), 108.0, tw, th);
        let sel = st.tab == *t;
        let n = (0..t.len()).filter(|k| entry(profile, assets, *t, *k).found).count();
        draw_rectangle(r.x, r.y, r.w, r.h, if sel { Color::new(0.24, 0.12, 0.22, 0.97) } else { PANEL });
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, if sel { ACCENT } else { Color::new(1.0, 1.0, 1.0, 0.2) });
        let label = format!("{} {}/{}", t.name(), n, t.len());
        let d = measure_text(&label, None, 16, 1.0);
        draw_text(&label, r.x + (tw - d.width) / 2.0, r.y + 24.0, 16.0, if sel { WHITE } else { DIM });
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
    let cols = 10.min(n);
    let (cw, gap) = (84.0, 10.0);
    let x0 = (w - (cols as f32 * cw + (cols as f32 - 1.0) * gap)) / 2.0;
    let y0 = 160.0;
    for i in 0..n {
        let e = entry(profile, assets, st.tab, i);
        let r = Rect::new(x0 + (i % cols) as f32 * (cw + gap), y0 + (i / cols) as f32 * (cw + gap), cw, cw);
        let sel = st.cursor == i;
        draw_rectangle(r.x, r.y, r.w, r.h, if sel { Color::new(0.20, 0.12, 0.26, 0.97) } else { PANEL });
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, if sel { ACCENT } else if r.contains(mouse) { Color::new(1.0, 1.0, 1.0, 0.5) } else { Color::new(1.0, 1.0, 1.0, 0.2) });
        // Fit the icon into the cell keeping its aspect ratio.
        let (iw, ih) = (e.icon.width(), e.icon.height());
        let k = (60.0 / iw.max(ih)).floor().max(1.0).min(4.0);
        let (dw, dh) = (iw * k, ih * k);
        let tint = if e.found { e.tint } else { Color::new(0.0, 0.0, 0.0, 0.85) };
        draw_texture_ex(e.icon, (r.x + (cw - dw) / 2.0).round(), (r.y + (cw - dh) / 2.0).round(), tint, DrawTextureParams { dest_size: Some(vec2(dw, dh)), ..Default::default() });
        if !e.found {
            let d = measure_text("?", None, 30, 1.0);
            draw_text("?", r.x + (cw - d.width) / 2.0, r.y + cw / 2.0 + 10.0, 30.0, DIM);
        }
        if r.contains(mouse) && is_mouse_button_pressed(MouseButton::Left) {
            st.cursor = i;
        }
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
    draw_rectangle(px, py, pw, ph, PANEL);
    draw_rectangle_lines(px, py, pw, ph, 2.0, Color::new(1.0, 1.0, 1.0, 0.2));
    if e.found {
        draw_text(&e.name, px + 20.0, py + 40.0, 30.0, if st.tab == Tab::Awakenings { GOLD } else { WHITE });
        let mut y = py + 72.0;
        for line in &e.lines {
            for l in wrap(line, pw - 40.0, 20) {
                draw_text(&l, px + 20.0, y, 20.0, Color::new(0.9, 0.9, 0.95, 1.0));
                y += 24.0;
            }
        }
    } else {
        draw_text("???", px + 20.0, py + 40.0, 30.0, DIM);
        draw_text(hint(st.tab), px + 20.0, py + 74.0, 20.0, DIM);
        if st.tab == Tab::Heroes {
            draw_text(&format!("Unlock: {}", unlock_text(ALL_HEROES[st.cursor])), px + 20.0, py + 104.0, 20.0, GOLD);
        }
    }
    button(Rect::new((w - 200.0) / 2.0, py + ph + 18.0, 200.0, 48.0), "Back", true) || is_key_pressed(KeyCode::Escape)
}

fn wrap(text: &str, width: f32, size: u16) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if measure_text(&candidate, None, size, 1.0).width > width && !line.is_empty() {
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
    draw_text(label, x, y + 30.0, 26.0, WHITE);
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
    draw_text(&format!("{:.0}%", *v * 100.0), bx + bw + 72.0, y + 30.0, 24.0, DIM);
    (*v - before).abs() > 1e-6
}

pub fn settings(profile: &mut Profile, time: f64) -> Option<SettingsAction> {
    backdrop(time);
    let h = screen_height();
    centered("SETTINGS", h * 0.22, 44.0, ACCENT);
    let mut changed = slider("Music", h * 0.36, &mut profile.music_volume);
    changed |= slider("Sound effects", h * 0.36 + 70.0, &mut profile.sfx_volume);
    centered("All music and sound effects are generated in code.", h * 0.36 + 160.0, 18.0, DIM);
    if button(Rect::new((screen_width() - 200.0) / 2.0, h * 0.36 + 200.0, 200.0, 48.0), "Back", true) || is_key_pressed(KeyCode::Escape) {
        return Some(SettingsAction::Back);
    }
    changed.then_some(SettingsAction::Changed)
}
