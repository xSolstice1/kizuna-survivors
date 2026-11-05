//! Stage 1 (Neon Shotengai) prop layout and collision.
//! The layout is a pure function of world coordinates: infinite, identical every run, and
//! shared by the sim (collision) and the client (drawing).

use crate::Vec2;

pub const TILE: f32 = 32.0;
pub const CHUNK: f32 = 192.0;
/// Every 14th tile column is a tactile-paving path that stays clear of props.
pub const PATH_EVERY: i32 = 14;

pub fn hash(x: i32, y: i32, salt: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77) ^ salt.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^ (h >> 15)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropKind {
    Lantern,
    Vending,
    Stall,
    NeonSign,
    Crates,
}

#[derive(Clone, Copy, Debug)]
pub struct Prop {
    pub kind: PropKind,
    /// Anchor: the sprite's bottom edge sits at `pos.y + 6`, centred on `pos.x`.
    pub pos: Vec2,
}

impl Prop {
    /// Solid footprint (the base of the sprite) as (min, max).
    pub fn footprint(&self) -> (Vec2, Vec2) {
        let (half_w, depth) = match self.kind {
            PropKind::Lantern => (3.0, 4.0),
            PropKind::Vending => (8.0, 10.0),
            PropKind::Stall => (16.0, 12.0),
            PropKind::NeonSign => (13.0, 6.0),
            PropKind::Crates => (7.0, 8.0),
        };
        let bottom = self.pos.y + 6.0;
        (Vec2::new(self.pos.x - half_w, bottom - depth), Vec2::new(self.pos.x + half_w, bottom))
    }
}

/// Props in one chunk (at most two). Footprints never cross the chunk's bounds.
pub fn chunk_props(cx: i32, cy: i32) -> ([Prop; 2], usize) {
    let none = Prop { kind: PropKind::Crates, pos: Vec2::ZERO };
    let mut out = [none; 2];
    let mut n = 0;
    let h = hash(cx, cy, 2);
    for k in 0..(h % 3) {
        let hk = hash(cx, cy, 10 + k);
        let px = cx as f32 * CHUNK + (hk % 160) as f32 + 16.0;
        let py = cy as f32 * CHUNK + ((hk >> 8) % 160) as f32 + 16.0;
        let path_x = ((px / (TILE * PATH_EVERY as f32)).round() * TILE * PATH_EVERY as f32) + TILE / 2.0;
        if (px - path_x).abs() < 40.0 || out[..n].iter().any(|q| (q.pos - Vec2::new(px, py)).length() < 56.0) {
            continue;
        }
        let kind = match (hk >> 16) % 10 {
            0..=2 => PropKind::Lantern,
            3 => PropKind::Vending,
            4..=5 => PropKind::Stall,
            6..=7 => PropKind::NeonSign,
            _ => PropKind::Crates,
        };
        out[n] = Prop { kind, pos: Vec2::new(px, py) };
        n += 1;
    }
    (out, n)
}

pub fn chunk_of(p: Vec2) -> (i32, i32) {
    ((p.x / CHUNK).floor() as i32, (p.y / CHUNK).floor() as i32)
}

/// Pushes a circle out of one box. Returns the push applied (zero if not touching).
fn push_out_box(c: Vec2, r: f32, (min, max): (Vec2, Vec2)) -> Vec2 {
    let q = Vec2::new(c.x.clamp(min.x, max.x), c.y.clamp(min.y, max.y));
    let d = c - q;
    let d2 = d.length_sq();
    if d2 >= r * r {
        return Vec2::ZERO;
    }
    if d2 > 1e-8 {
        let len = d2.sqrt();
        return d * ((r - len) / len);
    }
    // Centre inside the box: leave along the shallowest side.
    let (l, rt, t, b) = (c.x - min.x, max.x - c.x, c.y - min.y, max.y - c.y);
    let m = l.min(rt).min(t).min(b);
    if m == l {
        Vec2::new(-(l + r), 0.0)
    } else if m == rt {
        Vec2::new(rt + r, 0.0)
    } else if m == t {
        Vec2::new(0.0, -(t + r))
    } else {
        Vec2::new(0.0, b + r)
    }
}

/// Resolves a circle against props in its own chunk only (cheap; for crowds).
pub fn resolve_local(c: Vec2, r: f32) -> Vec2 {
    let (cx, cy) = chunk_of(c);
    let (props, n) = chunk_props(cx, cy);
    let mut c = c;
    for p in &props[..n] {
        c += push_out_box(c, r, p.footprint());
    }
    c
}

/// Resolves a circle against props in the surrounding 3x3 chunks (exact; for the player).
pub fn resolve(c: Vec2, r: f32) -> Vec2 {
    let (cx, cy) = chunk_of(c);
    let mut c = c;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (props, n) = chunk_props(cx + dx, cy + dy);
            for p in &props[..n] {
                c += push_out_box(c, r, p.footprint());
            }
        }
    }
    c
}

/// True if a circle overlaps any prop (3x3 chunk check).
pub fn blocked(c: Vec2, r: f32) -> bool {
    (resolve(c, r) - c).length_sq() > 1e-6
}

#[cfg(test)]
mod tests {
    use super::*;

    fn some_prop() -> Prop {
        for cy in 0..50 {
            for cx in 0..50 {
                let (p, n) = chunk_props(cx, cy);
                if n > 0 && p[0].kind == PropKind::Stall {
                    return p[0];
                }
            }
        }
        panic!("no stall found");
    }

    #[test]
    fn circles_are_pushed_out_of_props() {
        let p = some_prop();
        let (min, max) = p.footprint();
        let centre = Vec2::new((min.x + max.x) / 2.0, (min.y + max.y) / 2.0);
        let out = resolve(centre, 6.0);
        assert!(!blocked(out, 5.9), "still inside after resolve");
        assert!((out - centre).length() > 6.0);
    }

    #[test]
    fn footprints_stay_inside_their_chunk() {
        for cy in -20..20 {
            for cx in -20..20 {
                let (props, n) = chunk_props(cx, cy);
                for p in &props[..n] {
                    let (min, max) = p.footprint();
                    assert_eq!(chunk_of(min), (cx, cy));
                    assert_eq!(chunk_of(Vec2::new(max.x - 0.01, max.y - 0.01)), (cx, cy));
                }
            }
        }
    }
}
