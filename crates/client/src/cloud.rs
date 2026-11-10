//! Cloud saves: the game side of web/cloud.js (Supabase sign-in and sync).
//!
//! The local save stays the working copy. Every `save::store` also hands the plugin the whole
//! state as JSON (`state_json`), shaped like the database: a profile row (currencies + a
//! key/value document of everything else), one object per item (with structured columns for
//! querying plus the game's own item encoding in `data`), and the loadout. When the plugin
//! decides this device should take the cloud's copy, `incoming()` returns it as a Profile.
//! Natively (and with no Supabase config) everything here is a no-op.

use serde_json::{json, Map, Value};
use sim::content::ALL_HEROES;
use sim::gear::{Item, LOADOUT};
use sim::meta::{Profile, RunReport};
use sim::world::{RunState, World};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// No cloud configured (or native build).
    Off,
    SignedOut,
    Connecting,
    Synced,
    Syncing,
    /// Offline or the server refused: retrying.
    Error,
    /// This device and the cloud both changed: the player must pick one.
    Conflict,
}

#[cfg(target_arch = "wasm32")]
mod backend {
    #[link(wasm_import_module = "env")]
    extern "C" {
        fn kz_cloud_status() -> u32;
        fn kz_cloud_providers() -> u32;
        fn kz_cloud_push(ptr: *const u8, len: u32);
        fn kz_cloud_run(ptr: *const u8, len: u32);
        fn kz_cloud_sign_in(ptr: *const u8, len: u32);
        fn kz_cloud_sign_out();
        fn kz_cloud_resolve(choice: u32);
        fn kz_cloud_incoming_len() -> u32;
        fn kz_cloud_incoming(ptr: *mut u8, len: u32);
        fn kz_cloud_user_len() -> u32;
        fn kz_cloud_user(ptr: *mut u8, len: u32);
        fn kz_cloud_password_sign_in(e: *const u8, el: u32, p: *const u8, pl: u32);
        fn kz_cloud_sign_up(u: *const u8, ul: u32, e: *const u8, el: u32, p: *const u8, pl: u32);
        fn kz_cloud_magic_link(e: *const u8, el: u32);
        fn kz_cloud_reset(e: *const u8, el: u32);
        fn kz_cloud_set_password(p: *const u8, pl: u32);
        fn kz_cloud_recovery() -> u32;
        fn kz_cloud_pending() -> u32;
        fn kz_cloud_message_kind() -> u32;
        fn kz_cloud_message_len() -> u32;
        fn kz_cloud_message(ptr: *mut u8, len: u32);
        fn kz_cloud_clear_message();
    }

    pub fn password_sign_in(e: &str, p: &str) {
        unsafe { kz_cloud_password_sign_in(e.as_ptr(), e.len() as u32, p.as_ptr(), p.len() as u32) }
    }
    pub fn sign_up(u: &str, e: &str, p: &str) {
        unsafe { kz_cloud_sign_up(u.as_ptr(), u.len() as u32, e.as_ptr(), e.len() as u32, p.as_ptr(), p.len() as u32) }
    }
    pub fn magic_link(e: &str) {
        unsafe { kz_cloud_magic_link(e.as_ptr(), e.len() as u32) }
    }
    pub fn reset(e: &str) {
        unsafe { kz_cloud_reset(e.as_ptr(), e.len() as u32) }
    }
    pub fn set_password(p: &str) {
        unsafe { kz_cloud_set_password(p.as_ptr(), p.len() as u32) }
    }
    pub fn recovery() -> bool {
        unsafe { kz_cloud_recovery() != 0 }
    }
    pub fn pending() -> bool {
        unsafe { kz_cloud_pending() != 0 }
    }
    pub fn message() -> (String, u32) {
        let n = unsafe { kz_cloud_message_len() };
        let text = if n == 0 { String::new() } else { take(n, kz_cloud_message) };
        (text, unsafe { kz_cloud_message_kind() })
    }
    pub fn clear_message() {
        unsafe { kz_cloud_clear_message() }
    }

    /// Plugin version check (see miniquad's plugin system).
    #[no_mangle]
    pub extern "C" fn kz_cloud_crate_version() -> u32 {
        1
    }

    pub fn status() -> u32 {
        unsafe { kz_cloud_status() }
    }
    pub fn providers() -> u32 {
        unsafe { kz_cloud_providers() }
    }
    pub fn push(s: &str) {
        unsafe { kz_cloud_push(s.as_ptr(), s.len() as u32) }
    }
    pub fn run(s: &str) {
        unsafe { kz_cloud_run(s.as_ptr(), s.len() as u32) }
    }
    pub fn sign_in(p: &str) {
        unsafe { kz_cloud_sign_in(p.as_ptr(), p.len() as u32) }
    }
    pub fn sign_out() {
        unsafe { kz_cloud_sign_out() }
    }
    pub fn resolve(c: u32) {
        unsafe { kz_cloud_resolve(c) }
    }
    fn take(len: u32, f: unsafe extern "C" fn(*mut u8, u32)) -> String {
        let mut buf = vec![0u8; len as usize];
        unsafe { f(buf.as_mut_ptr(), len) };
        String::from_utf8(buf).unwrap_or_default()
    }
    pub fn incoming() -> Option<String> {
        let n = unsafe { kz_cloud_incoming_len() };
        (n > 0).then(|| take(n, kz_cloud_incoming))
    }
    pub fn user() -> String {
        let n = unsafe { kz_cloud_user_len() };
        if n == 0 {
            String::new()
        } else {
            take(n, kz_cloud_user)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    pub fn status() -> u32 {
        0
    }
    pub fn providers() -> u32 {
        0
    }
    pub fn push(_: &str) {}
    pub fn run(_: &str) {}
    pub fn sign_in(_: &str) {}
    pub fn sign_out() {}
    pub fn resolve(_: u32) {}
    pub fn incoming() -> Option<String> {
        None
    }
    pub fn user() -> String {
        String::new()
    }
    pub fn password_sign_in(_: &str, _: &str) {}
    pub fn sign_up(_: &str, _: &str, _: &str) {}
    pub fn magic_link(_: &str) {}
    pub fn reset(_: &str) {}
    pub fn set_password(_: &str) {}
    pub fn recovery() -> bool {
        false
    }
    pub fn pending() -> bool {
        false
    }
    pub fn message() -> (String, u32) {
        (String::new(), 0)
    }
    pub fn clear_message() {}
}

pub fn status() -> Status {
    match backend::status() {
        1 => Status::SignedOut,
        2 => Status::Connecting,
        3 => Status::Synced,
        4 => Status::Syncing,
        5 => Status::Error,
        6 => Status::Conflict,
        _ => Status::Off,
    }
}

/// Sign-in options to show: (Google, Discord, email link).
pub fn providers() -> (bool, bool, bool) {
    let b = backend::providers();
    (b & 1 != 0, b & 2 != 0, b & 4 != 0)
}

pub fn sign_in(provider: &str) {
    backend::sign_in(provider);
}
pub fn sign_out() {
    backend::sign_out();
}
/// Settles a conflict: keep the cloud save (`true`) or this device's.
pub fn resolve(keep_cloud: bool) {
    backend::resolve(if keep_cloud { 0 } else { 1 });
}
pub fn password_sign_in(email: &str, password: &str) {
    backend::password_sign_in(email, password);
}
/// Creates an account (username, email, password).
pub fn sign_up(username: &str, email: &str, password: &str) {
    backend::sign_up(username, email, password);
}
pub fn magic_link(email: &str) {
    backend::magic_link(email);
}
/// Emails a password-reset link.
pub fn reset_password(email: &str) {
    backend::reset(email);
}
/// After opening a reset link: sets the new password.
pub fn set_password(password: &str) {
    backend::set_password(password);
}
/// Opened from a password-reset link (show "set a new password").
pub fn recovery() -> bool {
    backend::recovery()
}
/// A sign-in request is in flight.
pub fn pending() -> bool {
    backend::pending()
}
/// Feedback for the sign-in screen: (text, kind) with kind 0 info, 1 error, 2 success.
pub fn message() -> (String, u32) {
    backend::message()
}
pub fn clear_message() {
    backend::clear_message();
}

/// The signed-in account's username (or email).
pub fn user() -> String {
    backend::user()
}

/// Offers the current state for syncing (cheap no-op when the cloud is off).
pub fn push(p: &Profile) {
    if status() != Status::Off {
        backend::push(&state_json(p));
    }
}

/// A cloud save this device should load, if the plugin has one.
pub fn incoming() -> Option<Profile> {
    backend::incoming().and_then(|s| apply_json(&s))
}

/// Logs a finished run (queued while offline).
pub fn record_run(w: &World, r: &RunReport) {
    if status() == Status::Off {
        return;
    }
    let result = match w.state {
        RunState::Won => "won",
        _ if w.dawn_out => "dawn",
        _ => "fallen",
    };
    let weapons: Vec<Value> = w.weapons.iter().map(|x| json!({ "weapon": format!("{:?}", x.kind).to_lowercase(), "level": x.level, "awakened": x.awakened })).collect();
    let run = json!({
        "hero": w.cfg.hero.key(),
        "stage": w.cfg.stage.key(),
        "difficulty": w.cfg.difficulty.key(),
        "ascension": w.cfg.ascension,
        "result": result,
        "seconds": w.time,
        "level": w.player.level,
        "kills": w.kills,
        "sen": r.sen,
        "gear_found": r.gear.len(),
        "detail": { "weapons": weapons, "bosses_down": w.bosses_down, "unlocked": r.unlocked.iter().map(|h| h.key()).collect::<Vec<_>>() },
    });
    backend::run(&run.to_string());
}

/// Lines of the save that become their own rows or columns (everything else is the doc).
fn is_split(key: &str) -> bool {
    key == "sen" || key == "dust" || key == "item" || key.starts_with("loadout.")
}

fn item_json(it: &Item) -> Value {
    let affixes: Vec<Value> = it.affixes.iter().map(|a| json!({ "stat": format!("{:?}", a.stat).to_lowercase(), "quality": (a.quality * 1000.0).round() / 1000.0 })).collect();
    json!({
        "item_id": it.id,
        "slot": it.slot.key(),
        "rarity": it.rarity.name().to_lowercase(),
        "ilvl": it.ilvl,
        "plus": it.plus,
        "set": it.set.map(|s| s.key()),
        "exotic": it.exotic.map(|e| e.key()),
        "unique": it.unique.map(|u| u.key()),
        "element": it.element.key(),
        "locked": it.locked,
        "affixes": affixes,
        "data": it.serialize(),
    })
}

/// The whole save, shaped like the database rows.
pub fn state_json(p: &Profile) -> String {
    let mut doc = Map::new();
    for line in p.serialize().lines() {
        if let Some((k, v)) = line.split_once('=') {
            if !is_split(k) {
                doc.insert(k.to_string(), Value::String(v.to_string()));
            }
        }
    }
    let mut items: Vec<&Item> = p.items.iter().collect();
    items.sort_by_key(|i| i.id);
    let mut loadouts: Vec<(String, usize, u32)> = Vec::new();
    for (hi, h) in ALL_HEROES.iter().enumerate() {
        for (pos, id) in p.loadouts[hi].iter().enumerate() {
            if *id != 0 {
                loadouts.push((h.key().to_string(), pos, *id));
            }
        }
    }
    loadouts.sort();
    json!({
        "profile": { "sen": p.sen, "dust": p.dust, "doc": Value::Object(doc) },
        "items": items.iter().map(|i| item_json(i)).collect::<Vec<_>>(),
        "loadouts": loadouts.iter().map(|(h, pos, id)| json!({ "hero": h, "pos": pos, "item_id": id })).collect::<Vec<_>>(),
    })
    .to_string()
}

/// Rebuilds a Profile from a cloud state (the inverse of `state_json`).
pub fn apply_json(s: &str) -> Option<Profile> {
    let v: Value = serde_json::from_str(s).ok()?;
    let prof = v.get("profile")?;
    let mut text = String::new();
    if let Some(doc) = prof.get("doc").and_then(Value::as_object) {
        for (k, val) in doc {
            if let Some(val) = val.as_str() {
                text += &format!("{k}={val}\n");
            }
        }
    }
    text += &format!("sen={}\n", prof.get("sen").and_then(Value::as_u64).unwrap_or(0));
    text += &format!("dust={}\n", prof.get("dust").and_then(Value::as_u64).unwrap_or(0));
    for it in v.get("items").and_then(Value::as_array).into_iter().flatten() {
        if let Some(d) = it.get("data").and_then(Value::as_str) {
            text += &format!("item={d}\n");
        }
    }
    let mut slots: Vec<(String, [u32; 12])> = Vec::new();
    for l in v.get("loadouts").and_then(Value::as_array).into_iter().flatten() {
        let (Some(h), Some(pos), Some(id)) = (l.get("hero").and_then(Value::as_str), l.get("pos").and_then(Value::as_u64), l.get("item_id").and_then(Value::as_u64)) else { continue };
        if pos as usize >= LOADOUT.len() {
            continue;
        }
        let i = match slots.iter().position(|(k, _)| k == h) {
            Some(i) => i,
            None => {
                slots.push((h.to_string(), [0; 12]));
                slots.len() - 1
            }
        };
        slots[i].1[pos as usize] = id as u32;
    }
    for (h, ids) in slots {
        let list: Vec<String> = ids.iter().map(|x| x.to_string()).collect();
        text += &format!("loadout.{h}={}\n", list.join(","));
    }
    Some(Profile::parse(&text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim::content::HeroId;
    use sim::gear::{roll_item, Rarity, Slot};
    use sim::rng::Rng;

    #[test]
    fn cloud_state_round_trips() {
        let mut p = Profile { sen: 1234, dust: 77, ..Default::default() };
        let mut rng = Rng::new(3);
        let a = p.add_item(roll_item(&mut rng, Some(Slot::Ring), Rarity::Epic, 12));
        p.add_item(roll_item(&mut rng, None, Rarity::Legendary, 30));
        assert!(p.equip(HeroId::Ren, p.slot_for(HeroId::Ren, Slot::Ring), a));
        p.unlocked.push(HeroId::Riku);
        let back = apply_json(&state_json(&p)).expect("parses");
        assert_eq!(back.serialize(), p.serialize());
    }
}
