//! Hidden developer menu (testing tools). Opened by a secret: type "kizunadev" on the
//! main menu, or tap the game title 7 times quickly (for phones). Unlocks for the session
//! only; a "Dev" button then appears on the main menu and the pause menu.

use macroquad::prelude::*;

use crate::kit::{self, Style, DIM, GOLD};

const SECRET: &str = "kizunadev";
const TAPS: usize = 7;

#[derive(Default)]
pub struct DevMenu {
    /// The secret was entered this session.
    pub unlocked: bool,
    /// The overlay is showing.
    pub open: bool,
    /// Show the fps / sim timing line.
    pub show_fps: bool,
    typed: String,
    taps: Vec<f64>,
    /// When "Reset to a new save" was first clicked (a second click within 3 s confirms).
    reset_armed: Option<f64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DevAction {
    // Out of a run.
    UnlockHeroes,
    OpenWorlds,
    GrantExotics,
    GrantArts,
    GrantGear,
    GrantSet,
    Sen,
    Dust,
    MeetAll,
    /// Wipe the save back to a brand-new player (keeps sound settings).
    ResetSave,
    // In a run.
    SkipMini,
    SkipBoss,
    SkipFinal,
    MaxLoadout,
    AllWeapons,
    AwakenAll,
    FillSpecial,
    Chest,
    BigChest,
    Pickup,
    Shrine,
    ClearEnemies,
    Newcomers,
    Stress,
    // Both.
    ToggleFps,
}

impl DevMenu {
    /// Watches for the secret on the main menu. `title_tapped` = the title was clicked this frame.
    pub fn watch_secret(&mut self, title_tapped: bool, now: f64) -> bool {
        while let Some(c) = get_char_pressed() {
            self.typed.push(c.to_ascii_lowercase());
            if self.typed.len() > SECRET.len() {
                self.typed.remove(0);
            }
        }
        if title_tapped {
            self.taps.push(now);
            self.taps.retain(|t| now - t < 3.0);
        }
        let hit = self.typed == SECRET || self.taps.len() >= TAPS;
        if hit && !self.unlocked {
            self.unlocked = true;
            self.open = true;
            self.typed.clear();
            self.taps.clear();
            return true;
        }
        false
    }

    /// Draws the overlay. `in_run` picks the run tools. Returns the chosen action.
    pub fn draw(&mut self, in_run: bool) -> Option<DevAction> {
        use DevAction::*;
        let (w, h) = (kit::screen_width(), kit::screen_height());
        draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, 0.6));
        let items: &[(&str, DevAction)] = if in_run {
            &[
                ("Skip to sub-boss", SkipMini),
                ("Skip to second boss", SkipBoss),
                ("Skip to final boss", SkipFinal),
                ("Max loadout", MaxLoadout),
                ("Every library weapon", AllWeapons),
                ("Awaken all weapons", AwakenAll),
                ("Fill special meter", FillSpecial),
                ("Open a chest", Chest),
                ("Open a 5-item chest", BigChest),
                ("Drop a pickup", Pickup),
                ("Shrine underfoot", Shrine),
                ("Clear all enemies", ClearEnemies),
                ("Spawn Leader + Uncommons", Newcomers),
                ("Stress test on/off", Stress),
                ("Show fps on/off", ToggleFps),
            ]
        } else {
            &[
                ("Unlock all heroes", UnlockHeroes),
                ("Open every world + difficulty", OpenWorlds),
                ("Grant every Exotic", GrantExotics),
                ("Grant every Arsenal attack", GrantArts),
                ("Grant 20 Legendary/Mythic", GrantGear),
                ("Grant Set pieces (selected world)", GrantSet),
                ("+10,000 Sen", Sen),
                ("+5,000 Spirit Dust", Dust),
                ("Mark every Otherworlder met", MeetAll),
                ("Show fps on/off", ToggleFps),
                ("Reset to a new save", ResetSave),
            ]
        };
        let cols = 2usize;
        let (bw, bh, gap) = (300.0, 42.0, 10.0);
        let rows = items.len().div_ceil(cols);
        let pw = cols as f32 * bw + (cols as f32 + 1.0) * gap + 20.0;
        let ph = rows as f32 * (bh + gap) + 130.0;
        let (px, py) = ((w - pw) / 2.0, (h - ph) / 2.0);
        kit::panel_ex(px, py, pw, ph, kit::PANEL_HI, GOLD);
        kit::panel_title(if in_run { "DEV MENU  ·  RUN" } else { "DEV MENU" }, px + 18.0, py + 34.0, 20.0, GOLD);
        kit::text("Testing tools. Unlocked for this session only.", px + 20.0, py + 58.0, 13.0, DIM);
        let mut out = None;
        for (i, (label, act)) in items.iter().enumerate() {
            let (c, r) = (i % cols, i / cols);
            let rect = Rect::new(px + 20.0 + c as f32 * (bw + gap), py + 72.0 + r as f32 * (bh + gap), bw, bh);
            let armed = *act == ResetSave && self.reset_armed.is_some_and(|t| get_time() - t < 3.0);
            let label = match *act {
                ToggleFps => format!("{label}  ({})", if self.show_fps { "on" } else { "off" }),
                ResetSave if armed => "Click again to wipe the save".to_string(),
                _ => label.to_string(),
            };
            if kit::button(rect, &label, true, if armed { Style::Primary } else { Style::Secondary }) {
                if *act == ResetSave && !armed {
                    self.reset_armed = Some(get_time());
                } else {
                    self.reset_armed = None;
                    out = Some(*act);
                }
            }
        }
        if kit::button(Rect::new(px + pw / 2.0 - 90.0, py + ph - 52.0, 180.0, 40.0), "Close", true, Style::Primary) || is_key_pressed(KeyCode::Escape) {
            self.open = false;
        }
        if out == Some(ToggleFps) {
            self.show_fps = !self.show_fps;
            out = None;
        }
        out
    }
}
