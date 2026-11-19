//! First-run tutorial: the run waits while a spotlight shows one thing at a time with a caption.

use macroquad::prelude::*;
use sim::content::hero_def;
use sim::World;

use crate::kit::{self, screen_height, screen_width, Style, DIM, GOLD, INK};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Step {
    Hero,
    Xp,
    Health,
    Loadout,
    Special,
    Timer,
    Map,
    Sen,
    SpecialReady,
    Chest,
    Shrine,
    Boss,
    /// Added after the others (bits are saved): the movement key.
    Move,
}

/// The first `TOUR` steps play back to back when the first run starts; the rest wait for their moment.
const ALL: [Step; 13] = [Step::Hero, Step::Move, Step::Xp, Step::Health, Step::Loadout, Step::Special, Step::Timer, Step::Map, Step::Sen, Step::SpecialReady, Step::Chest, Step::Shrine, Step::Boss];
const TOUR: usize = 9;

impl Step {
    pub fn bit(self) -> u32 {
        1 << self as u32
    }
}

/// What the run shows right now that a tip can point at.
#[derive(Default)]
pub struct Sights {
    pub special_ready: bool,
    pub chest: bool,
    pub shrine: bool,
    pub boss: bool,
}

/// The next unseen step that has something to point at, if any.
pub fn due(seen: u32, s: &Sights) -> Option<Step> {
    ALL.into_iter().find(|st| {
        seen & st.bit() == 0
            && match st {
                Step::SpecialReady => s.special_ready,
                Step::Chest => s.chest,
                Step::Shrine => s.shrine,
                Step::Boss => s.boss,
                _ => true,
            }
    })
}

fn caption(step: Step, world: &World, touch: bool) -> (&'static str, String) {
    let special = hero_def(world.cfg.hero).special_name;
    let pick = |keys: &str, tap: &str| if touch { tap.to_string() } else { keys.to_string() };
    match step {
        Step::Hero => ("This is you", pick("Move with WASD or the arrow keys. No need to aim: your weapons fire on their own.", "Drag anywhere on the left side to move. No need to aim: your weapons fire on their own.")),
        Step::Xp => ("Experience", "Fallen foes drop spirit gems. Walk over them to fill this bar; every level up lets you pick a new skill.".into()),
        Step::Health => ("Health", "Otherworlders hurt on contact. If this runs out, the night is over.".into()),
        Step::Loadout => ("Weapons and passives", "Your skills live here. Level ups add new ones or power up the ones you have.".into()),
        Step::Special => ("Special move", format!("Fills as you fight. When it glows, {} to unleash {special}.", if touch { "tap SPECIAL" } else { "press R" })),
        Step::Move => {
            let m = sim::content::move_def(hero_def(world.cfg.hero).movement);
            ("Your move", format!("{}: {}. {}", m.name, m.text.to_lowercase(), pick("Space dashes the way you're moving, right-click towards the cursor. The ring round your portrait refills it.", "Tap MOVE to go the way you're dragging. The ring shows it refilling.")))
        }
        Step::Timer => {
            let st = world.stage();
            let at = st.final_at as u32;
            ("The night", format!("Survive {} rounds. {} rises at {:02}:{:02}; defeat it before dawn to win.", sim::campaign::ROUNDS, st.final_boss.short_name(), at / 60, at % 60))
        }
        Step::Map => ("Map", pick("Chests, shrines and bosses nearby show here. Press M or Tab for the full map.", "Chests, shrines and bosses nearby show here. Tap it for the full map.")),
        Step::Sen => ("Kills and Sen", "You keep your Sen after every night, won or lost. Spend it in the Shrine Shop on permanent upgrades.".into()),
        Step::SpecialReady => ("Special ready!", format!("Your meter is full. {} to unleash {special} and clear the crowd.", if touch { "Tap SPECIAL" } else { "Press R" })),
        Step::Chest => ("A chest", "Walk into it to open it: free skill upgrades inside.".into()),
        Step::Shrine => ("A roadside shrine", "Step in to receive a blessing. Every blessing comes with a price, so you may also walk away.".into()),
        Step::Boss => ("A boss", "Its health bar sits here. Bosses hit hard: keep moving and let your weapons work.".into()),
    }
}

pub enum Action {
    Next,
    Skip,
}

/// Dims everything but `hole`, rings it and captions it. `age` is how long the step has shown:
/// input waits a moment, so a key already held for play can't dismiss a tip unread.
pub fn draw(step: Step, hole: Rect, world: &World, touch: bool, age: f32) -> Option<Action> {
    let (w, h) = (screen_width(), screen_height());
    let fade = (age / 0.25).min(1.0);
    let pad = 8.0;
    let r = Rect::new(hole.x - pad, hole.y - pad, hole.w + 2.0 * pad, hole.h + 2.0 * pad);
    let dim = Color::new(0.02, 0.01, 0.05, 0.72 * fade);
    draw_rectangle(0.0, 0.0, w, r.y, dim);
    draw_rectangle(0.0, r.bottom(), w, h - r.bottom(), dim);
    draw_rectangle(0.0, r.y, r.x, r.h, dim);
    draw_rectangle(r.right(), r.y, w - r.right(), r.h, dim);
    let pulse = 0.65 + 0.35 * (get_time() as f32 * 5.0).sin();
    kit::rounded_outline(r.x, r.y, r.w, r.h, 10.0, 3.0, Color::new(GOLD.r, GOLD.g, GOLD.b, pulse * fade));

    let (title, body) = caption(step, world, touch);
    let cw = 460.0_f32.min(w - 2.0 * kit::MARGIN);
    let lines = kit::wrap(&body, cw - 40.0, 18);
    let ch = 64.0 + lines.len() as f32 * 24.0 + 58.0;
    let cx = (r.center().x - cw / 2.0).clamp(kit::MARGIN, (w - kit::MARGIN - cw).max(kit::MARGIN));
    let gap = 16.0;
    let cy = if r.bottom() + gap + ch <= h - kit::MARGIN {
        r.bottom() + gap
    } else if r.y - gap - ch >= kit::MARGIN {
        r.y - gap - ch
    } else {
        (h - ch) / 2.0
    };
    kit::panel_ex(cx, cy, cw, ch, kit::PANEL_HI, GOLD);
    kit::text_fit(title, cx + 20.0, cy + 38.0, cw - 110.0, 26.0, GOLD);
    let n = ALL.iter().position(|s| *s == step).unwrap_or(0);
    if n < TOUR {
        let count = format!("{} / {TOUR}", n + 1);
        kit::text(&count, cx + cw - 20.0 - kit::measure(&count, 16).width, cy + 36.0, 16.0, DIM);
    }
    for (i, line) in lines.iter().enumerate() {
        kit::text(line, cx + 20.0, cy + 72.0 + i as f32 * 24.0, 18.0, INK);
    }
    let by = cy + ch - 52.0;
    let skip_r = Rect::new(cx + 20.0, by, 170.0, 38.0);
    let next_r = Rect::new(cx + cw - 20.0 - 150.0, by, 150.0, 38.0);
    let last = n + 1 == TOUR || n >= TOUR;
    let next_label = match (last, touch) {
        (true, true) => "Got it",
        (true, false) => "Got it (Enter)",
        (false, true) => "Next",
        (false, false) => "Next (Enter)",
    };
    let skip = kit::button(skip_r, if touch { "Skip tutorial" } else { "Skip tutorial (Esc)" }, true, Style::Secondary);
    let next = kit::button(next_r, next_label, true, Style::Primary);
    if age < 0.35 {
        return None;
    }
    // A tap anywhere but Skip moves on: the whole screen is the Next button.
    let tapped = is_mouse_button_pressed(MouseButton::Left) && !skip_r.contains(kit::mouse_position().into());
    if skip || is_key_pressed(KeyCode::Escape) {
        Some(Action::Skip)
    } else if next || tapped || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
        Some(Action::Next)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tour_runs_in_order_then_tips_wait_for_their_moment() {
        let quiet = Sights::default();
        assert_eq!(due(0, &quiet), Some(Step::Hero));
        assert_eq!(due(Step::Hero.bit(), &quiet), Some(Step::Move), "the movement key right after moving");
        assert_eq!(due(Step::Hero.bit() | Step::Move.bit() | Step::Xp.bit(), &quiet), Some(Step::Health));
        let toured = ALL[..TOUR].iter().fold(0, |m, s| m | s.bit());
        assert_eq!(due(toured, &quiet), None, "event tips need their event");
        assert_eq!(due(toured, &Sights { shrine: true, boss: true, ..Default::default() }), Some(Step::Shrine));
        assert_eq!(due(toured | Step::Shrine.bit(), &Sights { shrine: true, ..Default::default() }), None, "each tip shows once");
        assert_eq!(due(u32::MAX, &Sights { special_ready: true, chest: true, shrine: true, boss: true }), None, "skipped");
    }
}
