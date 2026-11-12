use macroquad::prelude::*;
use sim::campaign::{MID_BOSS_ROUND, SUB_BOSS_ROUND};
use sim::world::DT;
use sim::meta::{Profile, RunReport};
use sim::{Input, RunState, Stress, World, WorldConfig};

mod armory;
mod audio;
mod chest_ui;
mod cloud;
mod devmenu;
mod kit;
mod menu;
mod platform;
mod render;
mod save;
mod sprites;
mod stage;
mod synth;
mod textinput;
mod touch;
mod ui;
mod voice;

use render::{Assets, Fx};

fn window_conf() -> Conf {
    // high_dpi: render at the device's real pixel density (sharp text on phones and retina screens).
    let mut conf = Conf { window_title: "Kizuna Survivors".into(), window_width: 1280, window_height: 720, high_dpi: true, ..Default::default() };
    if perf_stage().is_some() {
        conf.platform.swap_interval = Some(0); // vsync off so frame time is real work
    }
    conf
}

/// Dev perf harness (native): `KZ_PERF=<stage key>` plays a 2000-enemy stress fight,
/// prints the average frame time and exits.
/// On the web, a `kz_perf=<stage>` line in the save does the same (for headless tests), and
/// `kz_perf_real` picks the realistic late-night scene.
fn perf_stage() -> Option<sim::campaign::StageId> {
    let from_save = save::raw().lines().find_map(|l| l.strip_prefix("kz_perf=").map(str::to_string));
    std::env::var("KZ_PERF").ok().or(from_save).map(|k| sim::campaign::StageId::from_key(&k).unwrap_or(sim::campaign::StageId::Shotengai))
}

fn perf_real() -> bool {
    std::env::var("KZ_PERF_REAL").is_ok() || save::raw().lines().any(|l| l.trim() == "kz_perf_real")
}

#[derive(PartialEq)]
enum Screen {
    /// Sign-in (mandatory when cloud saves are configured).
    Login,
    Menu,
    /// Map select (world + difficulty), before character select.
    Stages,
    Armory,
    Draw,
    Shop,
    Collection,
    Stats,
    Settings,
    Playing,
    Paused,
}

fn read_input() -> Input {
    let mut d = sim::Vec2::ZERO;
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        d.y -= 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        d.y += 1.0;
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        d.x -= 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        d.x += 1.0;
    }
    Input { move_dir: d, ..Default::default() }
}

fn toggle_auto_special(profile: &mut Profile, world: &mut World) {
    profile.auto_special_off = !profile.auto_special_off;
    world.cfg.auto_special = profile.auto_special();
    save::store(profile);
}

fn new_world(stress: Option<Stress>, profile: &Profile) -> World {
    let seed = (macroquad::miniquad::date::now() * 1000.0) as u64;
    {
    let ascension = if profile.difficulty == sim::campaign::Difficulty::Nightmare { profile.ascension.min(profile.ascension_open(profile.stage)) } else { 0 };
    World::new(WorldConfig {
        seed,
        stress,
        hero: profile.selected,
        meta: profile.bonus(),
        stage: profile.stage,
        difficulty: profile.difficulty,
        ascension,
        gear: profile.loadout(profile.selected),
        auto_special: profile.auto_special(),
        ..Default::default()
    })
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let assets = Assets::load();
    let stage_art = stage::StageArt::load();
    let mut fx = Fx::new();
    // Accounts are mandatory where cloud saves are set up (web builds with a Supabase config).
    let mut screen = if cloud::status() == cloud::Status::Off { Screen::Menu } else { Screen::Login };
    let mut stress: Option<Stress> = None;
    let mut profile = save::load();
    // Hand the cloud plugin our state so it can reconcile with the account (if signed in).
    cloud::push(&profile);
    let mut world = new_world(stress, &profile);
    // Set once a finished run has been banked into the profile.
    let mut report: Option<RunReport> = None;
    let mut shop_cursor = 0usize;
    let mut acc = 0.0f32;
    let mut sim_ms = 0.0f32;
    let mut chest_anim: Option<chest_ui::ChestAnim> = None;
    // Space is latched until the sim next steps; at high refresh rates many frames take no step.
    let mut special_queued = false;
    let mut overlay_focus = 0usize;
    let mut overlay_was_open = false;
    let mut cards_focus = 0usize;

    let mut collection = menu::CollectionState::default();
    let mut armory_state = armory::ArmoryState::default();
    let mut draw_state = armory::DrawState::default();
    let mut dev = devmenu::DevMenu::default();
    let mut stats_scroll = menu::StripScroll::default();
    let mut login_state = menu::LoginState::default();
    let mut world_rt: Option<RenderTarget> = None;
    let mut dev_pending: Option<devmenu::DevAction> = None;
    // The big map view is open (the run pauses meanwhile).
    let mut map_open = false;
    let mut map_opened_now = false;
    let mut ult_alert = ui::UltAlert::default();
    let mut touch = touch::TouchControls::default();

    kit::init();
    // One shared atlas for every sprite and font glyph: draws stop switching textures, so a
    // frame is a handful of GPU draw calls instead of hundreds (the big cost in browsers).
    set_default_filter_mode(FilterMode::Linear);
    build_textures_atlas();
    ui::loading();
    next_frame().await;
    let t_boot = get_time();
    ui::loading();
    next_frame().await;
    macroquad::logging::info!("fonts ready in {:.2}s", get_time() - t_boot);
    let t_audio = get_time();
    let mut audio = audio::Audio::load(profile.music_volume, profile.sfx_volume).await;
    macroquad::logging::info!("audio ready in {:.2}s", get_time() - t_audio);
    // Which run-end jingle has played (so it plays once).
    let mut jingle_played = false;
    let perf = perf_stage();
    let (mut perf_frames, mut perf_ms, mut perf_cpu) = (0u32, 0.0f64, 0.0f64);
    let mut perf_parts = [0.0f64; 5];
    if let Some(st) = perf {
        profile.stage = st;
        if perf_real() {
            // A realistic late night: full awakened loadout at 22:00, the player can't die.
            world = new_world(None, &profile);
            world.debug_max_loadout();
            world.debug_awaken_all();
            world.time = 22.0 * 60.0;
            world.player.max_hp = 1e9;
            world.player.hp = 1e9;
        } else {
            stress = Some(Stress { enemies: 2000, projectiles: 4000 });
            world = new_world(stress, &profile);
        }
        screen = Screen::Playing;
    }

    loop {
        let dt = get_frame_time().min(0.1);
        let frame_start = get_time();
        if let Some(st) = perf {
            // Keep the scene in play: take level-ups and chests automatically.
            if world.offer.is_some() {
                world.bot_pick();
            }
            if world.chest_result.is_some() {
                world.close_chest();
                chest_anim = None;
            }
            if world.shrine_offer.is_some() {
                world.choose_shrine(None);
            }
            perf_frames += 1;
            if perf_frames > 60 {
                perf_ms += get_frame_time() as f64 * 1000.0;
            }
            if perf_frames == 60 + 300 {
                let avg = perf_ms / 300.0;
                macroquad::logging::info!("KZ_PERF {}: {avg:.2} ms/frame ({:.0} fps), {:.2} ms CPU, {} enemies, {} projectiles", st.key(), 1000.0 / avg, perf_cpu / 360.0, world.enemies.len(), world.projectiles.len());
                let n = perf_frames as f64;
                macroquad::logging::info!("KZ_PERF parts (ms): sim {:.2} world {:.2} hud {:.2} minimap {:.2} rest-ui {:.2}", perf_parts[0] / n, perf_parts[1] / n, perf_parts[2] / n, perf_parts[3] / n, perf_parts[4] / n);
                return;
            }
        }
        // UI draws in virtual 1280x720 pixels (scaled down on small screens).
        kit::begin_ui();
        let in_run = screen == Screen::Playing && !matches!(world.state, RunState::Dead | RunState::Won) && !world.paused();
        touch.update(in_run && !map_open);
        let portrait = screen_height() > screen_width();
        if (touch.pause || portrait) && in_run {
            screen = Screen::Paused;
        }
        let now = get_time();
        // A cloud save to load (another device's progress), applied between runs.
        if !matches!(screen, Screen::Playing | Screen::Paused) {
            if let Some(p) = cloud::incoming() {
                profile = p;
                save::store_local(&profile);
            }
        }
        audio.begin_frame();
        // UI click feedback on any out-of-run screen.
        if !matches!(screen, Screen::Playing | Screen::Paused) && (is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Enter)) {
            audio.play(synth::Sfx::Click);
        }
        match screen {
            Screen::Playing | Screen::Paused => {}
            _ => audio.set_music(Some(synth::Track::Menu)),
        }
        // Hidden dev menu: the secret is watched on the main menu.
        if screen == Screen::Menu && !dev.open {
            let title = menu::title_rect();
            let tapped = is_mouse_button_pressed(MouseButton::Left) && title.contains(kit::mouse_position().into());
            if dev.watch_secret(tapped, now) {
                audio.play(synth::Sfx::Awaken);
            }
        }
        // Signed out (or the cloud and this device disagree): back to the login screen.
        if screen != Screen::Login && !matches!(screen, Screen::Playing | Screen::Paused) && matches!(cloud::status(), cloud::Status::SignedOut | cloud::Status::Conflict) {
            screen = Screen::Login;
        }
        let out_of_run = !matches!(screen, Screen::Playing | Screen::Paused);
        if out_of_run && dev.open {
            menu::backdrop(now);
            if let Some(a) = dev.draw(false) {
                dev_profile_action(a, &mut profile);
                save::store(&profile);
                audio.play(synth::Sfx::Buy);
            }
            ui::rotate_prompt();
            next_frame().await;
            continue;
        }

        // --- Out-of-run screens ---
        match screen {
            Screen::Login => {
                if menu::login(&mut login_state, &assets, now) {
                    screen = Screen::Menu;
                }
            }
            Screen::Menu => match menu::main_menu(&profile, &assets, &stage_art, now, dev.unlocked) {
                Some(menu::MenuAction::Dev) => dev.open = true,
                Some(menu::MenuAction::Start) => screen = Screen::Stages,
                Some(menu::MenuAction::Shop) => screen = Screen::Shop,
                Some(menu::MenuAction::Armory) => {
                    armory_state.hero = profile.selected.index();
                    screen = Screen::Armory;
                }
                Some(menu::MenuAction::Draw) => screen = Screen::Draw,
                Some(menu::MenuAction::Collection) => screen = Screen::Collection,
                Some(menu::MenuAction::Stats) => screen = Screen::Stats,
                Some(menu::MenuAction::Settings) => screen = Screen::Settings,
                None => {}
            },
            Screen::Collection => {
                if menu::collection(&profile, &assets, &mut collection, now) {
                    screen = Screen::Menu;
                }
            }
            Screen::Stats => {
                if menu::stats(&profile, &assets, &mut stats_scroll, now) {
                    screen = Screen::Menu;
                }
            }
            Screen::Settings => match menu::settings(&mut profile, now) {
                Some(menu::SettingsAction::Changed) => {
                    audio.music_volume = profile.music_volume;
                    audio.sfx_volume = profile.sfx_volume;
                    audio.apply_music_volume();
                    save::store(&profile);
                }
                Some(menu::SettingsAction::Back) => screen = Screen::Menu,
                None => {}
            },
            Screen::Armory => match armory::armory(&mut profile, &assets, &mut armory_state, now) {
                Some(armory::ArmoryAction::Changed) => save::store(&profile),
                Some(armory::ArmoryAction::Back) => {
                    save::store(&profile);
                    screen = Screen::Menu;
                }
                None => {}
            },
            Screen::Draw => match armory::shrine_draw(&mut profile, &assets, &mut draw_state, now) {
                Some(armory::DrawAction::Drew(best)) => {
                    audio.play(if best >= sim::gear::Rarity::Legendary { synth::Sfx::Awaken } else { synth::Sfx::Chime });
                    save::store(&profile);
                }
                Some(armory::DrawAction::Changed) => save::store(&profile),
                Some(armory::DrawAction::Back) => {
                    save::store(&profile);
                    screen = Screen::Menu;
                }
                None => {}
            },
            Screen::Stages => match menu::stage_select(&mut profile, &assets, &stage_art, now) {
                Some(menu::StageAction::Go) => {
                    save::store(&profile);
                    if profile.stage.index() > 0 {
                        audio.prepare(synth::Track::World(profile.stage.index() as u8)).await;
                    }
                    world = new_world(stress, &profile);
                    report = None;
                    fx.set_hero(profile.selected);
                    fx.accent = stage::accent(profile.stage);
                    fx.clear();
                    jingle_played = false;
                    screen = Screen::Playing;
                }
                Some(menu::StageAction::Back) => {
                    save::store(&profile);
                    screen = Screen::Menu;
                }
                None => {}
            },
            Screen::Shop => match menu::shop(&mut profile, &assets, &mut shop_cursor, now) {
                Some(menu::ShopAction::Changed) => {
                    audio.play(synth::Sfx::Buy);
                    save::store(&profile);
                }
                Some(menu::ShopAction::Back) => screen = Screen::Menu,
                None => {}
            },
            Screen::Playing | Screen::Paused => {}
        }
        if !matches!(screen, Screen::Playing | Screen::Paused) {
            ui::rotate_prompt();
            next_frame().await;
            continue;
        }

        // --- Global keys ---
        let over = matches!(world.state, RunState::Dead | RunState::Won);
        // Esc pauses only during play; on the shrine screen it means "walk away".
        if is_key_pressed(KeyCode::Escape) && !dev.open && !over && !map_open && (screen == Screen::Paused || !world.paused()) {
            screen = if screen == Screen::Paused { Screen::Playing } else { Screen::Paused };
        }
        if is_key_pressed(KeyCode::N) {
            fx.show_numbers = !fx.show_numbers;
        }
        if is_key_pressed(KeyCode::U) && profile.rank(sim::meta::PowerUp::AutoSpecial) > 0 {
            toggle_auto_special(&mut profile, &mut world);
        }
        let mut toggle_stress = false;
        let mut dev_run: Option<devmenu::DevAction> = None;
        if screen == Screen::Paused && dev.open {
            // Drawn later with the pause UI; actions come back through `dev_run` next frame.
            dev_run = dev_pending.take();
        }
        if dev_run == Some(devmenu::DevAction::Stress) {
            toggle_stress = true;
        }
        if toggle_stress {
            stress = match stress {
                None => Some(Stress { enemies: 2000, projectiles: 4000 }),
                Some(_) => None,
            };
        }
        // Giving up from the pause menu ends the run normally (Sen is still banked).
        if screen == Screen::Paused && is_key_pressed(KeyCode::Q) {
            world.state = RunState::Dead;
            screen = Screen::Playing;
        }
        let over = matches!(world.state, RunState::Dead | RunState::Won);
        if over && report.is_none() && world.cfg.stress.is_none() {
            let r = profile.record_run(&world);
            cloud::record_run(&world, &r);
            report = Some(r);
            save::store(&profile);
        }
        // Music: battle theme, the boss theme while the Night Lord lives, a jingle at the end.
        if over {
            audio.set_music(None);
            if !jingle_played {
                jingle_played = true;
                audio.play(if world.state == RunState::Won { synth::Sfx::Victory } else { synth::Sfx::Defeat });
            }
        } else {
            let boss = world.boss().is_some_and(|b| b.2.rank() >= sim::content::Rank::Boss);
            let theme = match world.cfg.stage.index() {
                0 => synth::Track::Battle,
                i => synth::Track::World(i as u8),
            };
            audio.set_music(Some(if boss { synth::Track::Boss } else { theme }));
        }
        let restart = (is_key_pressed(KeyCode::R) && screen == Screen::Paused) || toggle_stress;
        if restart {
            world = new_world(stress, &profile);
            report = None;
            jingle_played = false;
            fx.clear();
            screen = Screen::Playing;
        }
        if let Some(a) = dev_run {
            use devmenu::DevAction::*;
            let st = world.stage();
            match a {
                SkipMini => world.time = world.time.max(st.round_start(SUB_BOSS_ROUND) - 3.0),
                SkipBoss => {
                    if st.boss.is_some() {
                        world.time = world.time.max(st.round_start(MID_BOSS_ROUND) - 3.0);
                    }
                }
                SkipFinal => world.time = world.time.max(st.final_at - 3.0),
                MaxLoadout => world.debug_max_loadout(),
                AllWeapons => world.debug_all_weapons(),
                AwakenAll => world.debug_awaken_all(),
                FillSpecial => world.player.special = 1.0,
                Chest => world.chests.push((world.player.pos + sim::Vec2::new(90.0, -40.0), 1)),
                BigChest => world.chests.push((world.player.pos + sim::Vec2::new(90.0, -40.0), 5)),
                Pickup => {
                    let k = sim::content::PickupKind::roll(rand::gen_range(0.0, 1.0));
                    let ang = rand::gen_range(0.0, std::f32::consts::TAU);
                    world.pickups.push((world.player.pos + sim::Vec2::from_angle(ang) * 70.0, k));
                }
                Shrine => world.shrine = Some(world.player.pos + sim::Vec2::new(0.0, 40.0)),
                ClearEnemies => world.debug_clear_enemies(),
                Newcomers => world.debug_spawn_newcomers(),
                _ => {}
            }
            audio.play(synth::Sfx::Buy);
        }
        if let Some(s) = stress.as_mut() {
            let mut changed = true;
            if is_key_pressed(KeyCode::RightBracket) {
                s.enemies += 500;
            } else if is_key_pressed(KeyCode::LeftBracket) {
                s.enemies = s.enemies.saturating_sub(500);
            } else if is_key_pressed(KeyCode::Equal) {
                s.projectiles += 1000;
            } else if is_key_pressed(KeyCode::Minus) {
                s.projectiles = s.projectiles.saturating_sub(1000);
            } else {
                changed = false;
            }
            if changed {
                world.cfg.stress = Some(*s);
            }
        }

        // --- Fixed-timestep sim (paused while a menu or level-up is open) ---
        // Map toggle (M / Tab) during play.
        if screen == Screen::Playing && !over && !world.paused() && (is_key_pressed(KeyCode::M) || is_key_pressed(KeyCode::Tab)) {
            map_open = !map_open;
        }
        if screen != Screen::Playing || over {
            map_open = false;
        }
        if screen == Screen::Playing && !world.paused() && !map_open {
            acc += dt;
            let mut input = read_input();
            input.move_dir = input.move_dir + touch.dir();
            special_queued |= is_key_pressed(KeyCode::Space) || touch.special;
            input.special = special_queued;
            let scale = render::world_scale();
            world.view_half = sim::Vec2::new(screen_width() / scale / 2.0, screen_height() / scale / 2.0);
            let t0 = get_time();
            let mut steps = 0;
            while acc >= DT && steps < 4 && !world.paused() {
                let t_s = get_time();
                world.step(input);
                perf_parts[0] += (get_time() - t_s) * 1000.0;
                fx.absorb(&world.events, world.player.pos, world.reach_half().x);
                audio.absorb(&world.events);
                // Delivered exactly once, even if the sim steps several times this frame.
                input.special = false;
                special_queued = false;
                acc -= DT;
                steps += 1;
            }
            if !fx.low_hp_warned && world.player.hp < world.player.max_hp * 0.25 {
                fx.low_hp_warned = true;
                fx.say(fx.lines().low_hp);
            }
            if steps == 4 {
                acc = 0.0; // too slow: drop backlog rather than spiral
            }
            if steps > 0 {
                sim_ms = sim_ms * 0.95 + ((get_time() - t0) * 1000.0 / steps as f64) as f32 * 0.05;
            }
            fx.update(dt);
            fx.animate(&assets.ren_anim, &world, now);
            fx.track_trail(&world, dt);
        } else {
            acc = 0.0;
            // Space on a menu (e.g. dismissing a chest) must not fire Purify afterwards.
            special_queued = false;
        }

        // --- Render ---
        clear_background(Color::from_rgba(28, 24, 40, 255));
        // The world renders at CSS-pixel resolution (not device pixels): ~4x fewer pixels on
        // retina screens and phones. The HUD and text stay at full resolution.
        let (tw, th) = (screen_width().max(1.0) as u32, screen_height().max(1.0) as u32);
        if world_rt.as_ref().is_none_or(|rt: &RenderTarget| rt.texture.width() as u32 != tw || rt.texture.height() as u32 != th) {
            let rt = render_target(tw, th);
            rt.texture.set_filter(FilterMode::Linear);
            world_rt = Some(rt);
        }
        let mut cam = render::camera_for(&world, &fx);
        cam.render_target = world_rt.clone();
        let t_w = get_time();
        render::draw_world(&world, &assets, &stage_art, &fx, &cam, now, world_rt.as_ref());
        perf_parts[1] += (get_time() - t_w) * 1000.0;
        set_default_camera();
        if world.stage().mechanic == sim::campaign::Mechanic::Downpour {
            let pp = cam.world_to_screen(vec2(world.player.pos.x, world.player.pos.y));
            stage_art.draw_downpour(pp, now, fx.flash);
        }
        stage_art.draw_vignette();
        kit::begin_ui();
        if screen != Screen::Paused && !map_open {
            let t_h = get_time();
            ui::hud(&world, &assets, touch.active);
            perf_parts[2] += (get_time() - t_h) * 1000.0;
            map_opened_now = false;
            let t_m = get_time();
            let clicked = !map_open && !over && !world.paused() && ui::minimap(&world, &assets, false, false);
            perf_parts[3] += (get_time() - t_m) * 1000.0;
            if clicked {
                map_open = true;
                map_opened_now = true;
            }
        }
        let alert_blocked = world.paused() || screen == Screen::Paused || map_open;
        let ult_ready = matches!(screen, Screen::Playing | Screen::Paused) && !matches!(world.state, RunState::Dead | RunState::Won) && world.player.special >= 1.0 && world.form_kind().is_none();
        if ult_alert.update(ult_ready, alert_blocked, dt) {
            audio.play(synth::Sfx::Rise);
        }
        ui::ult_alert(&ult_alert, ui::element_color(sim::content::hero_def(world.cfg.hero).element), touch.active, ult_ready && !alert_blocked);
        if dev.show_fps || stress.is_some() {
            ui::debug_overlay(get_fps(), sim_ms, &world, stress.is_some());
        }
        if in_run && !map_open && screen == Screen::Playing {
            let form = world.form_kind().map(|k| world.player.form / k.duration());
            touch.draw(world.player.special, world.player.special >= 1.0, form);
        }

        if let (Some((text, left)), false) = (&fx.banner, world.paused()) {
            ui::banner(text, *left, world.boss().is_some());
        }
        let mut overlay_picked = false;
        let focus_overlay_open = world.chest_result.is_none() && (world.shrine_offer.is_some() || world.offer.is_some());
        if portrait {
            // The rotate prompt covers the screen; draw no widgets so taps reach nothing.
        } else if let Some(items) = world.chest_result.clone() {
            let anim = chest_anim.get_or_insert_with(|| chest_ui::ChestAnim::new(items));
            let done = anim.update_and_draw(&assets, dt);
            for cue in anim.cues.drain(..) {
                audio.play(cue);
            }
            if done {
                audio.play(synth::Sfx::Click);
                world.close_chest();
                chest_anim = None;
            }
        } else if let Some(offer) = world.shrine_offer.clone() {
            let fresh = !overlay_was_open;
            if fresh {
                overlay_focus = 0;
            }
            let choice = ui::shrine(&offer, &assets, &mut overlay_focus);
            if let Some(choice) = choice.filter(|_| !fresh) {
                audio.play(if choice.is_some() { synth::Sfx::Chime } else { synth::Sfx::Click });
                world.choose_shrine(choice);
                overlay_picked = true;
            }
        } else if world.offer.is_some() && world.replacing.is_some() {
            let fresh = !overlay_was_open;
            if fresh {
                overlay_focus = 0;
            }
            let choice = ui::replace_picker(&world, &assets, &mut overlay_focus, touch.active);
            if let Some(choice) = choice.filter(|_| !fresh) {
                audio.play(synth::Sfx::Click);
                if choice.is_none() {
                    cards_focus = world.offer.as_ref().and_then(|o| o.iter().position(|&u| world.is_replacement(u))).unwrap_or(0);
                }
                world.replace_with(choice);
                overlay_picked = true;
            }
        } else if let Some(offer) = world.offer.clone() {
            let fresh = !overlay_was_open;
            if fresh {
                overlay_focus = std::mem::take(&mut cards_focus);
            }
            let picked = ui::level_up(&world, &offer, &assets, &mut overlay_focus, touch.active);
            if let Some(i) = picked.filter(|_| !fresh) {
                audio.play(synth::Sfx::Click);
                world.choose(i);
                overlay_picked = true;
            }
        } else if over {
            match ui::results(&world, world.state == RunState::Won, report.as_ref(), &fx.lines(), &assets) {
                Some(ui::ResultsAction::Retry) => {
                    audio.play(synth::Sfx::Click);
                    world = new_world(stress, &profile);
                    report = None;
                    jingle_played = false;
                    fx.clear();
                    screen = Screen::Playing;
                }
                Some(ui::ResultsAction::Menu) => {
                    audio.play(synth::Sfx::Click);
                    screen = Screen::Menu;
                }
                None => {}
            }
        } else if map_open {
            if ui::minimap(&world, &assets, true, map_opened_now) {
                map_open = false;
            }
        } else if screen == Screen::Paused && dev.open {
            if let Some(a) = dev.draw(true) {
                dev_pending = Some(a);
            }
        } else if screen == Screen::Paused {
            let auto_special = (profile.rank(sim::meta::PowerUp::AutoSpecial) > 0).then_some(profile.auto_special());
            match ui::pause(&world, &assets, &mut profile.music_volume, &mut profile.sfx_volume, dev.unlocked, auto_special) {
                Some(ui::PauseAction::Dev) => dev.open = true,
                Some(ui::PauseAction::AutoSpecial) => {
                    audio.play(synth::Sfx::Click);
                    toggle_auto_special(&mut profile, &mut world);
                }
                Some(ui::PauseAction::Resume) => {
                    audio.play(synth::Sfx::Click);
                    screen = Screen::Playing;
                }
                Some(ui::PauseAction::Restart) => {
                    audio.play(synth::Sfx::Click);
                    world = new_world(stress, &profile);
                    report = None;
                    jingle_played = false;
                    fx.clear();
                    screen = Screen::Playing;
                }
                Some(ui::PauseAction::GiveUp) => {
                    audio.play(synth::Sfx::Click);
                    world.state = RunState::Dead;
                    screen = Screen::Playing;
                }
                Some(ui::PauseAction::Volume) => {
                    audio.music_volume = profile.music_volume;
                    audio.sfx_volume = profile.sfx_volume;
                    audio.apply_music_volume();
                    save::store(&profile);
                }
                None => {}
            }
        }

        overlay_was_open = focus_overlay_open && !overlay_picked;

        if perf_frames > 60 {
            perf_cpu += (get_time() - frame_start) * 1000.0;
        }
        if perf_frames == 600 {
            if let Ok(path) = std::env::var("KZ_SHOT") {
                get_screen_data().export_png(&path);
            }
        }
        ui::rotate_prompt();
        next_frame().await
    }
}

/// Dev menu actions on the profile (out of a run).
fn dev_profile_action(a: devmenu::DevAction, profile: &mut Profile) {
    use devmenu::DevAction::*;
    use sim::gear::{self, Rarity, Slot};
    let mut rng = sim::rng::Rng::new((get_time() * 1e6) as u64);
    match a {
        UnlockHeroes => {
            for h in sim::content::ALL_HEROES {
                if !profile.unlocked.contains(&h) {
                    profile.unlocked.push(h);
                }
            }
        }
        OpenWorlds => {
            for (i, row) in profile.cleared.iter_mut().enumerate() {
                *row = [true; 3];
                profile.asc_open[i] = profile.asc_open[i].max(5);
            }
        }
        GrantExotics => {
            for e in gear::ALL_EXOTICS {
                profile.add_item(gear::roll_exotic(&mut rng, e, 40));
            }
        }
        GrantArts => {
            for art in gear::LEGENDARY_ARTS {
                let mut it = gear::roll_item(&mut rng, Some(Slot::Arsenal), Rarity::Mythic, 40);
                it.art = Some(art);
                profile.add_item(it);
            }
        }
        GrantGear => {
            for k in 0..20 {
                let r = if k % 3 == 0 { Rarity::Mythic } else { Rarity::Legendary };
                let set = sim::campaign::ALL_STAGES[(rng.next_u32() as usize) % 6];
                let mut it = gear::roll_item(&mut rng, None, r, 40);
                it.set = Some(set);
                profile.add_item(it);
            }
        }
        GrantSet => {
            let world = profile.stage;
            for slot in gear::set_pieces(world) {
                let piece = (0..500).map(|_| gear::roll_set_piece(&mut rng, world, 40)).find(|it| it.slot == slot);
                profile.add_item(piece.expect("a Set piece for every slot"));
            }
        }
        Sen => profile.sen += 10_000,
        Dust => profile.dust += 5_000,
        MeetAll => {
            for k in &mut profile.enemy_kills {
                *k = (*k).max(1);
            }
        }
        ResetSave => {
            // Back to a brand-new player: heroes, worlds, shop, relics, gear, currencies, stats
            // and collection all reset. Sound and Auto Special settings stay.
            let (music, sfx, auto_off) = (profile.music_volume, profile.sfx_volume, profile.auto_special_off);
            *profile = Profile::default();
            (profile.music_volume, profile.sfx_volume, profile.auto_special_off) = (music, sfx, auto_off);
        }
        _ => {}
    }
}
