//! Equipment screens: the Armory (paper doll, bag, item details, crafting) and the
//! Shrine Draw (Sen gacha with visible odds and pity).

use macroquad::prelude::*;

use crate::kit::{self, mouse_position, screen_width, Style, DIM, FAINT, GOLD, INK, JADE, SKY};
use crate::render::Assets;
use sim::content::{hero_def, HeroId, Upgrade, ALL_HEROES};
use sim::gear::{self, set_name, set_text, Item, Rarity, Slot, ALL_SLOTS, ALL_STATS, LOADOUT};
use sim::meta::Profile;

pub fn rarity_color(r: Rarity) -> Color {
    match r {
        Rarity::Common => Color::new(0.78, 0.78, 0.82, 1.0),
        Rarity::Uncommon => Color::new(0.45, 0.9, 0.5, 1.0),
        Rarity::Rare => Color::new(0.35, 0.65, 1.0, 1.0),
        Rarity::Epic => Color::new(0.75, 0.45, 1.0, 1.0),
        Rarity::Legendary => Color::new(1.0, 0.65, 0.2, 1.0),
        Rarity::Mythic => Color::new(1.0, 0.3, 0.45, 1.0),
        // Exotic shimmers through the spectrum.
        Rarity::Exotic => {
            let h = (get_time() as f32 * 0.35).fract();
            let k = |o: f32| 0.6 + 0.4 * ((h + o) * std::f32::consts::TAU).sin();
            Color::new(k(0.0), k(0.33), k(0.66), 1.0)
        }
    }
}

fn item_icon<'a>(assets: &'a Assets, it: &Item) -> &'a Texture2D {
    if let Some(e) = it.exotic {
        return assets.gear_icon_named(e.key());
    }
    if let Some(a) = it.art {
        return assets.gear_icon_named(a.key());
    }
    match it.arsenal {
        Some(k) => assets.icon(Upgrade::Weapon(k)),
        None => assets.gear_icon(it.slot),
    }
}

/// An item tile: rarity-tinted frame, icon, upgrade level, equipped / locked marks.
pub fn item_cell(assets: &Assets, it: &Item, x: f32, y: f32, s: f32, selected: bool, worn: bool) {
    let c = rarity_color(it.rarity);
    kit::rounded(x, y, s, s, 7.0, Color::new(c.r * 0.22, c.g * 0.22, c.b * 0.28, 0.95));
    if it.rarity == Rarity::Exotic {
        let t = get_time() as f32;
        for k in 0..3 {
            let a = t * 1.5 + k as f32 * 2.1;
            draw_circle(x + s / 2.0 + a.cos() * s * 0.42, y + s / 2.0 + a.sin() * s * 0.42, 2.0, Color::new(1.0, 1.0, 1.0, 0.8));
        }
    }
    if it.rarity >= Rarity::Legendary {
        let pulse = 0.25 + 0.15 * (get_time() as f32 * 3.0 + x * 0.05).sin();
        kit::rounded(x + 2.0, y + 2.0, s - 4.0, s - 4.0, 6.0, Color::new(c.r, c.g, c.b, pulse * 0.35));
    }
    kit::rounded_outline(x, y, s, s, 7.0, if selected { 3.0 } else { 1.5 }, if selected { INK } else { c });
    let pad = s * 0.16;
    draw_texture_ex(item_icon(assets, it), x + pad, y + pad, WHITE, DrawTextureParams { dest_size: Some(vec2(s - 2.0 * pad, s - 2.0 * pad)), ..Default::default() });
    if it.plus > 0 {
        kit::text(&format!("+{}", it.plus), x + 3.0, y + 12.0, 11.0, GOLD);
    }
    if worn {
        kit::rounded(x + s - 14.0, y + 2.0, 12.0, 12.0, 4.0, JADE);
        kit::text("E", x + s - 11.5, y + 12.0, 10.0, Color::new(0.05, 0.1, 0.05, 1.0));
    }
    if it.locked {
        draw_rectangle(x + s - 11.0, y + s - 11.0, 8.0, 6.0, GOLD);
        draw_circle_lines(x + s - 7.0, y + s - 12.0, 3.0, 1.5, GOLD);
    }
    if it.set.is_some() {
        draw_poly(x + 7.0, y + s - 7.0, 4, 4.0, 45.0, SKY);
    }
}

pub enum DetailAction {
    Equip,
    Unequip,
    Upgrade,
    Reroll(usize),
    Lock,
    Salvage,
}

/// Item details with stats, rolls, unique, set and actions. Returns the clicked action.
pub fn item_detail(profile: &Profile, assets: &Assets, it: &Item, hero: HeroId, x: f32, y: f32, w: f32, h: f32, actions: bool) -> Option<DetailAction> {
    let c = rarity_color(it.rarity);
    kit::panel_ex(x, y, w, h, kit::PANEL, Color::new(c.r, c.g, c.b, 0.6));
    item_cell(assets, it, x + 14.0, y + 14.0, 56.0, false, false);
    kit::text_fit(&it.name(), x + 80.0, y + 36.0, w - 94.0, 21.0, c);
    let sub = format!("{} {}  ·  iLvl {}  ·  Score {}", it.rarity.name(), it.slot.name(), it.ilvl, it.score());
    kit::text_fit(&sub, x + 80.0, y + 58.0, w - 94.0, 14.0, DIM);
    let mut ly = y + 96.0;
    let line_w = w - 28.0;
    if let Some((st, v)) = it.main_value() {
        kit::text_fit(&st.format(v), x + 14.0, ly, line_w, 18.0, INK);
        ly += 18.0;
    }
    if let Some(e) = it.exotic {
        kit::text("EXOTIC", x + 14.0, ly + 4.0, 14.0, c);
        ly += 20.0;
        let mut text = e.describe();
        if e.treasure() {
            text = format!("{text} Sacred Treasure. {}", gear::REGALIA_TEXT);
        }
        for l in kit::wrap(&text, line_w, 14) {
            kit::text(&l, x + 14.0, ly, 14.0, Color::new(1.0, 0.95, 0.85, 1.0));
            ly += 17.0;
        }
        if e == gear::Exotic::SovereignFinger {
            let n = profile.items.iter().filter(|i| i.exotic == Some(gear::Exotic::SovereignFinger)).count().min(gear::MAX_FINGERS as usize);
            kit::text(&format!("Fingers owned: {n}/{}", gear::MAX_FINGERS), x + 14.0, ly, 14.0, GOLD);
            ly += 17.0;
        }
    } else if let Some(a) = it.art {
        kit::text("EXCLUSIVE ARSENAL", x + 14.0, ly + 4.0, 14.0, c);
        ly += 20.0;
        let t = format!("{} (no weapon slot). {}", a.name(), a.describe(it.rarity >= Rarity::Mythic));
        for l in kit::wrap(&t, line_w, 14) {
            kit::text(&l, x + 14.0, ly, 14.0, SKY);
            ly += 17.0;
        }
    }
    if let Some(k) = it.arsenal {
        let lvl = it.rarity.arsenal_level();
        let awak = if it.rarity == Rarity::Mythic { ", Awakened" } else { "" };
        let t = format!("Grants {} Lv {}{} (no weapon slot), damage x{:.2}", Upgrade::Weapon(k).name(), lvl, awak, gear::ilvl_scale(it.ilvl) * (1.0 + 0.06 * it.plus as f32));
        for l in kit::wrap(&t, line_w, 15) {
            kit::text(&l, x + 14.0, ly, 15.0, SKY);
            ly += 18.0;
        }
    }
    ly += 10.0;
    let mut action = None;
    let reroll_cost = it.reroll_cost();
    for (i, a) in it.affixes.iter().enumerate() {
        let v = it.affix_value(a);
        kit::text_fit(&a.stat.format(v), x + 14.0, ly + 4.0, line_w - 150.0, 16.0, INK);
        // Roll quality bar.
        let qc = if a.quality > 0.9 { GOLD } else if a.quality > 0.6 { JADE } else { DIM };
        kit::bar(x + w - 150.0, ly - 6.0, 60.0, 8.0, a.quality, qc, Color::new(1.0, 1.0, 1.0, 0.08));
        kit::text(&format!("{:.0}%", a.quality * 100.0), x + w - 84.0, ly + 3.0, 12.0, qc);
        if actions && kit::button(Rect::new(x + w - 58.0, ly - 12.0, 46.0, 22.0), "Reroll", profile.dust >= reroll_cost, Style::Secondary) {
            action = Some(DetailAction::Reroll(i));
        }
        ly += 26.0;
    }
    if let Some(u) = it.unique {
        ly += 4.0;
        kit::text(u.name(), x + 14.0, ly, 16.0, c);
        ly += 18.0;
        for l in kit::wrap(&u.describe(it.unique_strength()), line_w, 14) {
            kit::text(&l, x + 14.0, ly, 14.0, Color::new(1.0, 0.92, 0.8, 1.0));
            ly += 17.0;
        }
    }
    if let Some(s) = it.set {
        ly += 6.0;
        let worn = profile.loadouts[hero.index()].iter().filter_map(|id| profile.item(*id)).filter(|x| x.set == Some(s)).count();
        kit::text(&format!("{} set  ({worn}/4 worn)", set_name(s)), x + 14.0, ly, 15.0, SKY);
        ly += 17.0;
        let (two, four) = set_text(s);
        for (n, t) in [(2, two), (4, four)] {
            let on = worn >= n;
            for l in kit::wrap(&format!("{n}: {t}"), line_w, 13) {
                kit::text(&l, x + 14.0, ly, 13.0, if on { JADE } else { FAINT });
                ly += 15.0;
            }
        }
    }
    let quality = format!("Roll quality {:.0}%", it.quality() * 100.0);
    kit::text(&quality, x + 14.0, y + h - 66.0, 13.0, FAINT);
    if actions {
        let by = y + h - 50.0;
        let bw = (w - 28.0 - 3.0 * 8.0) / 4.0;
        let worn = profile.worn_by(it.id).is_some_and(|(hh, _)| hh == hero.index());
        if kit::button(Rect::new(x + 14.0, by, bw, 38.0), if worn { "Unequip" } else { "Equip" }, true, Style::Primary) {
            action = Some(if worn { DetailAction::Unequip } else { DetailAction::Equip });
        }
        let up = if it.plus >= gear::MAX_PLUS { "Max +10".to_string() } else { format!("+1 ({})", it.upgrade_cost()) };
        if kit::button(Rect::new(x + 14.0 + bw + 8.0, by, bw, 38.0), &up, it.plus < gear::MAX_PLUS && profile.dust >= it.upgrade_cost(), Style::Secondary) {
            action = Some(DetailAction::Upgrade);
        }
        if kit::button(Rect::new(x + 14.0 + 2.0 * (bw + 8.0), by, bw, 38.0), if it.locked { "Unlock" } else { "Lock" }, true, Style::Secondary) {
            action = Some(DetailAction::Lock);
        }
        if kit::button(Rect::new(x + 14.0 + 3.0 * (bw + 8.0), by, bw, 38.0), &format!("Salvage {}", it.salvage()), !it.locked, Style::Danger) {
            action = Some(DetailAction::Salvage);
        }
        kit::text(&format!("Reroll cost: {reroll_cost} dust"), x + w - 150.0, y + h - 66.0, 12.0, FAINT);
    }
    action
}

#[derive(Default)]
pub struct ArmoryState {
    pub hero: usize,
    pub filter: Option<Slot>,
    pub selected: Option<u32>,
    pub page: usize,
    /// 0 score, 1 rarity, 2 newest.
    pub sort: u8,
}

pub enum ArmoryAction {
    Changed,
    Back,
}

fn dust_badge(assets: &Assets, dust: u32, x: f32, y: f32) {
    let v = format!("{dust}");
    let w = kit::chip(x, y, "DUST", &v, SKY);
    draw_texture_ex(assets.gear_icon_named("dust"), x - 26.0, y + 4.0, WHITE, DrawTextureParams { dest_size: Some(vec2(22.0, 22.0)), ..Default::default() });
    let _ = w;
}

pub fn armory(profile: &mut Profile, assets: &Assets, st: &mut ArmoryState, time: f64) -> Option<ArmoryAction> {
    crate::menu::backdrop(time);
    let w = screen_width();
    let mouse: Vec2 = mouse_position().into();
    crate::menu::centered("ARMORY", 52.0, 40.0, crate::menu::ACCENT);
    let sen = format!("{}", profile.sen);
    kit::chip(w - 20.0 - kit::measure(&sen, 20).width - kit::measure("SEN", 14).width - 28.0, 18.0, "SEN", &sen, GOLD);
    dust_badge(assets, profile.dust, w - 330.0, 18.0);
    let mut changed = false;

    // Heroes: pick one to view, equip it to make it the active hero.
    let heroes: Vec<HeroId> = ALL_HEROES.to_vec();
    let hero = ALL_HEROES[st.hero.min(ALL_HEROES.len() - 1)];
    let open = profile.is_unlocked(hero);
    let lx = 24.0;
    for (i, h) in heroes.iter().enumerate() {
        let r = Rect::new(lx + i as f32 * 42.0, 74.0, 38.0, 46.0);
        let sel = *h == hero;
        let active = *h == profile.selected;
        let unlocked = profile.is_unlocked(*h);
        kit::rounded(r.x, r.y, r.w, r.h, 6.0, if sel { Color::new(0.3, 0.16, 0.3, 0.95) } else { kit::PANEL });
        kit::rounded_outline(r.x, r.y, r.w, r.h, 6.0, if active { 2.5 } else { 1.5 }, if active { GOLD } else if sel { crate::menu::ACCENT } else { Color::new(1.0, 1.0, 1.0, 0.2) });
        if hero_def(*h).finished {
            let tint = if unlocked { WHITE } else { Color::new(0.0, 0.0, 0.0, 0.85) };
            draw_texture_ex(&assets.hero(*h)[0], r.x + 4.0, r.y + 3.0, tint, DrawTextureParams { dest_size: Some(vec2(30.0, 40.0)), ..Default::default() });
        } else {
            kit::text_fit_centered("?", r.x + r.w / 2.0, r.y + 32.0, r.w, 24.0, FAINT);
        }
        if r.contains(mouse) && is_mouse_button_pressed(MouseButton::Left) {
            st.hero = h.index();
        }
    }

    // Paper doll.
    let (dx, dy, dw, dh) = (lx, 128.0, 410.0, 400.0);
    kit::panel(dx, dy, dw, dh);
    let def = hero_def(hero);
    let soon = !def.finished;
    kit::text_fit(if open { def.name } else { "???" }, dx + 16.0, dy + 30.0, dw - 150.0, 20.0, INK);
    // Equip the hero: it becomes the one you play (and the menu follows it).
    let er = Rect::new(dx + dw - 126.0, dy + 10.0, 112.0, 30.0);
    if hero == profile.selected {
        kit::rounded(er.x, er.y, er.w, er.h, 8.0, Color::new(0.3, 0.24, 0.08, 0.95));
        kit::rounded_outline(er.x, er.y, er.w, er.h, 8.0, 1.5, GOLD);
        kit::text_fit_centered("EQUIPPED", er.x + er.w / 2.0, er.y + 21.0, er.w - 10.0, 15.0, GOLD);
    } else if kit::button(er, if open { "Equip hero" } else if soon { "Coming soon" } else { "Locked" }, open, Style::Primary) {
        profile.selected = hero;
        changed = true;
    }
    let bob = ((time * 2.0).sin() * 2.0) as f32;
    if soon {
        crate::menu::centered_at("?", dx + dw / 2.0, dy + 160.0, 90.0, FAINT);
    } else {
        let tint = if open { WHITE } else { Color::new(0.0, 0.0, 0.0, 0.85) };
        draw_texture_ex(&assets.hero(hero)[0], dx + dw / 2.0 - 48.0, dy + 66.0 + bob, tint, DrawTextureParams { dest_size: Some(vec2(96.0, 128.0)), ..Default::default() });
    }
    // Who they are, in the column between the slots.
    let colx = dx + 16.0 + 50.0 + 64.0;
    let colw = dw - 2.0 * (16.0 + 50.0 + 64.0) + 8.0;
    let mut ty2 = dy + 214.0;
    let lines: Vec<(String, Color, u16)> = if open {
        vec![
            (def.archetype.to_string(), crate::menu::ACCENT, 13),
            (format!("Starts with {}", Upgrade::Weapon(def.signature).name()), INK, 12),
            (format!("Innate: {}", def.innate_text), JADE, 12),
            (crate::menu::special_text(&def), GOLD, 12),
        ]
    } else if soon {
        vec![("COMING SOON".to_string(), GOLD, 15), ("A new hero is on the way.".to_string(), DIM, 12)]
    } else {
        vec![("LOCKED".to_string(), DIM, 14), (format!("Unlock: {}", sim::meta::unlock_text(hero)), GOLD, 12)]
    };
    for (t, c, sz) in lines {
        for l in kit::wrap(&t, colw, sz) {
            if ty2 > dy + dh - 10.0 {
                break;
            }
            kit::text_fit_centered(&l, colx + colw / 2.0, ty2, colw, sz as f32, c);
            ty2 += sz as f32 + 3.0;
        }
        ty2 += 4.0;
    }
    let cs = 50.0;
    let loadout = profile.loadouts[hero.index()];
    for (pos, slot) in LOADOUT.iter().enumerate() {
        let (col, row) = if pos < 6 { (0, pos) } else { (1, pos - 6) };
        let x = if col == 0 { dx + 16.0 } else { dx + dw - 16.0 - cs };
        let y = dy + 46.0 + row as f32 * (cs + 8.0);
        let id = loadout[pos];
        match profile.item(id) {
            Some(it) => item_cell(assets, it, x, y, cs, st.selected == Some(id), false),
            None => {
                kit::rounded(x, y, cs, cs, 7.0, Color::new(0.05, 0.04, 0.09, 0.9));
                kit::rounded_outline(x, y, cs, cs, 7.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.15));
                draw_texture_ex(assets.gear_icon(*slot), x + 9.0, y + 9.0, Color::new(1.0, 1.0, 1.0, 0.18), DrawTextureParams { dest_size: Some(vec2(32.0, 32.0)), ..Default::default() });
            }
        }
        let label = slot.name();
        let lx2 = if col == 0 { x + cs + 6.0 } else { x - 6.0 - kit::measure(label, 12).width };
        kit::text(label, lx2, y + cs / 2.0 + 4.0, 12.0, FAINT);
        if Rect::new(x, y, cs, cs).contains(mouse) && is_mouse_button_pressed(MouseButton::Left) {
            st.filter = Some(*slot);
            st.page = 0;
            st.selected = (id != 0).then_some(id);
        }
    }

    // Loadout totals.
    let l = profile.loadout(hero);
    let (tx, ty, tw, th) = (lx, dy + dh + 10.0, dw, 720.0 - dy - dh - 22.0);
    kit::panel(tx, ty, tw, th);
    let mut lines: Vec<String> = Vec::new();
    let b = l.bonus;
    let vals = [
        b.might, b.max_hp, b.armor, b.recovery, b.cooldown, b.area, b.move_speed, b.magnet, b.growth, b.greed, b.crit_chance, b.crit_damage, b.boss_damage, b.special_charge, b.luck, b.gear_find,
    ];
    for (stt, v) in ALL_STATS.iter().zip(vals) {
        if v > 0.0 {
            lines.push(stt.format(v));
        }
    }
    if lines.is_empty() {
        kit::text("Nothing equipped. Gear drops from bosses and elites,", tx + 14.0, ty + 26.0, 14.0, FAINT);
        kit::text("or try the Shrine Draw.", tx + 14.0, ty + 44.0, 14.0, FAINT);
    }
    let colw = (tw - 28.0) / 3.0;
    for (i, t) in lines.iter().enumerate() {
        let (c, r) = (i % 3, i / 3);
        kit::text_fit(t, tx + 14.0 + c as f32 * colw, ty + 22.0 + r as f32 * 17.0, colw - 6.0, 13.0, INK);
    }
    let extra: Vec<String> = l.exotics.iter().map(|e| e.name().to_string()).chain(l.uniques.iter().map(|(u, _)| u.name().to_string())).chain(l.sets4.iter().map(|s| format!("{} set", set_name(*s)))).collect();
    if !extra.is_empty() {
        kit::text_fit(&extra.join("  ·  "), tx + 14.0, ty + th - 10.0, tw - 28.0, 13.0, GOLD);
    }

    // Bag: filters, sort, grid.
    let (bx, by, bw) = (dx + dw + 16.0, 74.0, 420.0);
    let fs = 30.0;
    let mut fx = bx;
    let all = Rect::new(fx, by, 46.0, fs);
    if kit::button(all, "All", true, if st.filter.is_none() { Style::Primary } else { Style::Secondary }) {
        st.filter = None;
        st.page = 0;
    }
    fx += 50.0;
    for slot in ALL_SLOTS {
        let r = Rect::new(fx, by, fs, fs);
        let on = st.filter == Some(slot);
        kit::rounded(r.x, r.y, r.w, r.h, 6.0, if on { Color::new(0.8, 0.24, 0.3, 1.0) } else { Color::new(0.16, 0.12, 0.27, 1.0) });
        draw_texture_ex(assets.gear_icon(slot), r.x + 5.0, r.y + 5.0, WHITE, DrawTextureParams { dest_size: Some(vec2(20.0, 20.0)), ..Default::default() });
        if r.contains(mouse) && is_mouse_button_pressed(MouseButton::Left) {
            st.filter = Some(slot);
            st.page = 0;
        }
        fx += fs + 3.0;
    }
    let sort_label = ["Sort: Score", "Sort: Rarity", "Sort: Newest"][st.sort as usize % 3];
    let mut ids: Vec<u32> = profile.items.iter().filter(|i| st.filter.is_none_or(|f| i.slot == f)).map(|i| i.id).collect();
    match st.sort % 3 {
        0 => ids.sort_by_key(|id| std::cmp::Reverse(profile.item(*id).map_or(0, |i| i.score()))),
        1 => ids.sort_by_key(|id| std::cmp::Reverse(profile.item(*id).map_or(0, |i| i.rarity.index() * 10_000 + i.score() as usize))),
        _ => ids.sort_by_key(|id| std::cmp::Reverse(*id)),
    }
    let (gs, gap, cols, rows) = (52.0, 6.0, 7usize, 8usize);
    let per = cols * rows;
    let pages = ids.len().div_ceil(per).max(1);
    st.page = st.page.min(pages - 1);
    let wheel = mouse_wheel().1;
    let gy = by + fs + 10.0;
    let grid = Rect::new(bx, gy, cols as f32 * (gs + gap), rows as f32 * (gs + gap));
    if grid.contains(mouse) && wheel != 0.0 {
        if wheel < 0.0 && st.page + 1 < pages {
            st.page += 1;
        } else if wheel > 0.0 && st.page > 0 {
            st.page -= 1;
        }
    }
    for (k, id) in ids.iter().skip(st.page * per).take(per).enumerate() {
        let Some(it) = profile.item(*id) else { continue };
        let x = bx + (k % cols) as f32 * (gs + gap);
        let y = gy + (k / cols) as f32 * (gs + gap);
        let worn = profile.loadouts[hero.index()].contains(id);
        item_cell(assets, it, x, y, gs, st.selected == Some(*id), worn);
        if Rect::new(x, y, gs, gs).contains(mouse) && is_mouse_button_pressed(MouseButton::Left) {
            st.selected = Some(*id);
        }
    }
    if ids.is_empty() {
        kit::text("The bag is empty.", bx + 10.0, gy + 30.0, 16.0, FAINT);
    }
    let fy = gy + grid.h + 4.0;
    kit::text(&format!("{} items  ·  page {}/{}", profile.items.len(), st.page + 1, pages), bx, fy + 22.0, 14.0, DIM);
    if kit::button(Rect::new(bx + 190.0, fy + 2.0, 36.0, 30.0), "<", st.page > 0, Style::Secondary) {
        st.page -= 1;
    }
    if kit::button(Rect::new(bx + 230.0, fy + 2.0, 36.0, 30.0), ">", st.page + 1 < pages, Style::Secondary) {
        st.page += 1;
    }
    if kit::button(Rect::new(bx + 272.0, fy + 2.0, 140.0, 30.0), sort_label, true, Style::Secondary) {
        st.sort = (st.sort + 1) % 3;
    }
    if kit::button(Rect::new(bx, fy + 40.0, 200.0, 34.0), "Salvage Common/Uncommon", true, Style::Danger) {
        if profile.salvage_below(Rarity::Uncommon) > 0 {
            changed = true;
        }
    }

    // Details.
    let (ix, iy, iw, ih) = (bx + bw + 14.0, 74.0, w - (bx + bw + 14.0) - 20.0, 590.0);
    match st.selected.and_then(|id| profile.item(id)).cloned() {
        Some(it) => {
            if let Some(a) = item_detail(profile, assets, &it, hero, ix, iy, iw, ih, true) {
                changed = true;
                match a {
                    DetailAction::Equip => {
                        let pos = profile.slot_for(hero, it.slot);
                        profile.equip(hero, pos, it.id);
                    }
                    DetailAction::Unequip => {
                        if let Some((_, p)) = profile.worn_by(it.id) {
                            profile.unequip(hero, p);
                        }
                    }
                    DetailAction::Upgrade => {
                        profile.upgrade(it.id);
                    }
                    DetailAction::Reroll(i) => {
                        profile.reroll(it.id, i, (get_time() * 1e6) as u64 ^ it.id as u64);
                    }
                    DetailAction::Lock => {
                        if let Some(x) = profile.items.iter_mut().find(|x| x.id == it.id) {
                            x.locked = !x.locked;
                        }
                    }
                    DetailAction::Salvage => {
                        profile.salvage(it.id);
                        st.selected = None;
                    }
                }
            }
        }
        None => {
            kit::panel(ix, iy, iw, ih);
            kit::text("Select an item.", ix + 16.0, iy + 34.0, 18.0, DIM);
            for (k, l) in [
                "Common has 1 bonus roll, Uncommon 2, Rare 3,",
                "Epic 4, Legendary 5 and Mythic 6.",
                "Legendary and Mythic add a unique effect.",
                "Each roll has a quality: chase 100%.",
                "Higher difficulties and Ascension drop",
                "higher item levels. Salvage into Spirit Dust",
                "to upgrade (+10) or reroll a bonus.",
            ]
            .iter()
            .enumerate()
            {
                kit::text(l, ix + 16.0, iy + 70.0 + k as f32 * 20.0, 14.0, FAINT);
            }
        }
    }
    if kit::button(Rect::new(ix + iw - 160.0, 720.0 - 52.0, 160.0, 40.0), "Back", true, Style::Secondary) || is_key_pressed(KeyCode::Escape) {
        return Some(ArmoryAction::Back);
    }
    changed.then_some(ArmoryAction::Changed)
}

// ───────────────────────── Shrine Draw ─────────────────────────

#[derive(Default)]
pub struct DrawState {
    /// Ids from the last draw, and when it happened (for the reveal).
    pub results: Vec<u32>,
    pub at: f64,
    pub selected: Option<u32>,
}

pub enum DrawAction {
    /// Drew: the best rarity pulled (for the sound).
    Drew(Rarity),
    Changed,
    Back,
}

pub fn shrine_draw(profile: &mut Profile, assets: &Assets, st: &mut DrawState, time: f64) -> Option<DrawAction> {
    crate::menu::backdrop(time);
    let w = screen_width();
    let mouse: Vec2 = mouse_position().into();
    crate::menu::centered("SHRINE DRAW", 52.0, 40.0, crate::menu::ACCENT);
    let sen = format!("{}", profile.sen);
    kit::chip(w - 20.0 - kit::measure(&sen, 20).width - kit::measure("SEN", 14).width - 28.0, 18.0, "SEN", &sen, GOLD);
    let mut action = None;

    // Odds and pity.
    let (ox, oy, ow, oh) = (24.0, 80.0, 300.0, 380.0);
    kit::panel(ox, oy, ow, oh);
    kit::panel_title("ODDS", ox + 16.0, oy + 30.0, 16.0, GOLD);
    for (i, r) in gear::ALL_RARITIES.iter().take(6).enumerate() {
        let y = oy + 60.0 + i as f32 * 30.0;
        kit::text(r.name(), ox + 20.0, y, 17.0, rarity_color(*r));
        let p = format!("{}%", gear::DRAW_ODDS[i]);
        let d = kit::measure(&p, 17);
        kit::text(&p, ox + ow - 20.0 - d.width, y, 17.0, INK);
    }
    let left = gear::DRAW_PITY.saturating_sub(profile.draw_pity);
    for (k, l) in [
        format!("Legendary or better guaranteed within {left} draws."),
        format!("Items are item level {}.", profile.draw_ilvl()),
        "Clear harder content to raise it.".to_string(),
    ]
    .iter()
    .enumerate()
    {
        for (j, ln) in kit::wrap(l, ow - 32.0, 14).iter().enumerate() {
            kit::text(ln, ox + 16.0, oy + 250.0 + k as f32 * 40.0 + j as f32 * 17.0, 14.0, if k == 0 { GOLD } else { DIM });
        }
    }
    let one = format!("Draw x1  ({} Sen)", gear::DRAW_COST);
    let ten = format!("Draw x10  ({} Sen)", gear::DRAW10_COST);
    let seed = (get_time() * 1e6) as u64 ^ profile.next_item as u64 * 7919;
    let mut drew = None;
    if kit::button(Rect::new(ox, oy + oh + 14.0, ow, 50.0), &one, profile.sen >= gear::DRAW_COST, Style::Primary) {
        drew = Some(profile.draw(1, seed));
    }
    if kit::button(Rect::new(ox, oy + oh + 72.0, ow, 50.0), &ten, profile.sen >= gear::DRAW10_COST, Style::Primary) {
        drew = Some(profile.draw(10, seed));
    }
    if let Some(ids) = drew {
        if !ids.is_empty() {
            let best = ids.iter().filter_map(|id| profile.item(*id)).map(|i| i.rarity).max().unwrap_or(Rarity::Common);
            st.results = ids;
            st.at = time;
            st.selected = None;
            action = Some(DrawAction::Drew(best));
        }
    }

    for (k, l) in kit::wrap("Exotics can't be drawn: only a world's final boss drops them, rarely.", ow, 13).iter().enumerate() {
        kit::text(l, ox, oy + oh + 148.0 + k as f32 * 16.0, 13.0, rarity_color(Rarity::Exotic));
    }

    // Results with a staggered reveal.
    let (rx, ry) = (ox + ow + 20.0, 80.0);
    let rw = 470.0;
    kit::panel(rx, ry, rw, 520.0);
    if st.results.is_empty() {
        kit::text("Offer Sen to the shrine and draw a fortune.", rx + 20.0, ry + 40.0, 16.0, DIM);
    }
    let cs = 80.0;
    for (k, id) in st.results.iter().enumerate() {
        let x = rx + 20.0 + (k % 5) as f32 * (cs + 10.0);
        let y = ry + 30.0 + (k / 5) as f32 * (cs + 60.0);
        let shown = time - st.at > k as f64 * 0.12;
        let Some(it) = profile.item(*id) else { continue };
        if !shown {
            kit::rounded(x, y, cs, cs, 8.0, Color::new(0.2, 0.12, 0.25, 1.0));
            continue;
        }
        let age = ((time - st.at) - k as f64 * 0.12) as f32;
        if it.rarity >= Rarity::Epic && age < 1.0 {
            let c = rarity_color(it.rarity);
            draw_circle(x + cs / 2.0, y + cs / 2.0, cs * (0.6 + age), Color::new(c.r, c.g, c.b, 0.35 * (1.0 - age)));
        }
        item_cell(assets, it, x, y, cs, st.selected == Some(*id), false);
        kit::text_fit_centered(it.rarity.name(), x + cs / 2.0, y + cs + 18.0, cs, 13.0, rarity_color(it.rarity));
        kit::text_fit_centered(it.slot.name(), x + cs / 2.0, y + cs + 34.0, cs, 12.0, DIM);
        if Rect::new(x, y, cs, cs).contains(mouse) && is_mouse_button_pressed(MouseButton::Left) {
            st.selected = Some(*id);
        }
    }
    let (ix, iw) = (rx + rw + 14.0, w - (rx + rw + 14.0) - 20.0);
    if let Some(it) = st.selected.and_then(|id| profile.item(id)).cloned() {
        let hero = profile.selected;
        if let Some(DetailAction::Lock) = item_detail(profile, assets, &it, hero, ix, 80.0, iw, 520.0, false) {
            action = Some(DrawAction::Changed);
        }
    }
    if kit::button(Rect::new(w - 180.0, 720.0 - 56.0, 160.0, 42.0), "Back", true, Style::Secondary) || is_key_pressed(KeyCode::Escape) {
        action = Some(DrawAction::Back);
    }
    action
}
