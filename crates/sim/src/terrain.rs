//! Terrain: high ground, low ground and the cliffs between them, for the campaign worlds.
//!
//! The map is endless and a pure function of (stage, cell): every run of a stage has the same
//! hills and valleys. Each 16-unit cell has a level (0 low, 1 ground, 2 high) from smooth noise
//! flattened into plateaus. Where the level changes there is a **cliff**: a wall for players
//! and enemies alike (nobody falls off; projectiles fly over). In the top-down view a drop to
//! the south shows its rock face below the edge, so that wall is the lower cell under the
//! face; drops to the other sides are walled on the higher cell (its rim). Either way every
//! pair of side-by-side cells at different levels has a wall between them. The only way between levels is a
//! **road**: wavy roads run every `ROAD_EVERY` cells in both directions and ramp over every
//! level change they cross. The start is a clearing on a road crossing, so the player is always
//! connected to the road network and can never be sealed in. Plateaus too small for a road are
//! simply obstacles.
//!
//! The world keeps a cache of cells around the player (cheap lookups for thousands of enemies)
//! and a flow field: the walking distance from the player to every cell nearby, so enemies go
//! around cliffs instead of piling against them.

use std::collections::VecDeque;

use crate::campaign::StageId;
use crate::stage::{chunk_props as props_raw, hash, Prop, PropKind, CHUNK};

/// Props per chunk with terrain: the stage's scatter (up to 2) plus trailside lanterns.
pub const PROPS: usize = 4;
pub type ChunkProps = ([Prop; PROPS], usize);
use crate::Vec2;

pub const CELL: f32 = 16.0;
pub const ROAD_EVERY: i32 = 24;
/// Roads are this many cells wide on each side of their centre line (3 cells = 48 units).
const ROAD_HALF: i32 = 1;
/// The start clearing (cells): never a cliff.
const CLEAR: f32 = 10.0;

/// Cell flags (low 2 bits: the level).
pub const LEVEL: u8 = 0b11;
pub const WALL: u8 = 1 << 2;
pub const ROAD: u8 = 1 << 3;
/// A road cell where the level changes (drawn as stairs / a slope).
pub const RAMP: u8 = 1 << 4;

const CACHE: i32 = 256;
const FLOW: i32 = 112;
const UNREACHED: u16 = u16::MAX;

/// Whether a stage has terrain (the prologue streets stay flat).
pub fn enabled(stage: StageId) -> bool {
    stage != StageId::Shotengai
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Value noise in 0..1 on a lattice of size 1.
pub fn noise(x: f32, y: f32, salt: u32) -> f32 {
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (smooth(x - xi as f32), smooth(y - yi as f32));
    let v = |dx: i32, dy: i32| (hash(xi + dx, yi + dy, salt) & 0xFFFF) as f32 / 65535.0;
    let top = v(0, 0) + (v(1, 0) - v(0, 0)) * fx;
    let bot = v(0, 1) + (v(1, 1) - v(0, 1)) * fx;
    top + (bot - top) * fy
}

fn salt(stage: StageId) -> u32 {
    0x5EED_0000 + stage.index() as u32 * 977
}

/// Level thresholds on the height field.
pub const LOW_BELOW: f32 = 0.34;
pub const HIGH_ABOVE: f32 = 0.62;

/// The continuous height field at a point in cell units (cells sample it at their corner).
/// The client draws its contour lines; the sim rounds it to cells.
pub fn height_cells(stage: StageId, x: f32, y: f32) -> f32 {
    let s = salt(stage);
    let n = noise(x / 15.0, y / 15.0, s) * 0.7 + noise(x / 6.0, y / 6.0, s + 1) * 0.3;
    // Flat around the start.
    let d = (x * x + y * y).sqrt();
    if d < CLEAR + 8.0 {
        let k = smooth(((d - CLEAR) / 8.0).clamp(0.0, 1.0));
        0.5 + (n - 0.5) * k
    } else {
        n
    }
}

/// The height field at a world position.
pub fn height(stage: StageId, p: Vec2) -> f32 {
    height_cells(stage, p.x / CELL, p.y / CELL)
}

/// Distance (cells, continuous) from a world position to the nearest road centre line.
pub fn road_distance(stage: StageId, p: Vec2) -> f32 {
    let s = salt(stage);
    let (x, y) = (p.x / CELL, p.y / CELL);
    let mut best = f32::MAX;
    for (c, t, off) in [(x, y, 11u32), (y, x, 23u32)] {
        let k0 = (c / ROAD_EVERY as f32).round();
        for dk in -1..=1 {
            let k = k0 + dk as f32;
            let wob = (noise(t / 18.0, k * 3.7, s + off) - 0.5) * 9.0;
            best = best.min((c - (k * ROAD_EVERY as f32 + wob + 0.5)).abs());
        }
    }
    best
}

/// The raw level of a cell, before cliffs and roads.
fn level_raw(stage: StageId, cx: i32, cy: i32) -> u8 {
    if !enabled(stage) {
        return 1;
    }
    // Sampled at the cell's centre, so cells match the drawn contours.
    let n = height_cells(stage, cx as f32 + 0.5, cy as f32 + 0.5);
    if n > HIGH_ABOVE {
        2
    } else if n < LOW_BELOW {
        0
    } else {
        1
    }
}

/// Sideways offset (cells) of road `k` at position `t` along it.
fn wobble(stage: StageId, k: i32, t: i32, salt_off: u32) -> i32 {
    ((noise(t as f32 / 18.0, k as f32 * 3.7, salt(stage) + salt_off) - 0.5) * 9.0).round() as i32
}

fn is_road(stage: StageId, cx: i32, cy: i32) -> bool {
    if !enabled(stage) {
        return false;
    }
    let near = |c: i32, t: i32, off: u32| {
        let k = (c as f32 / ROAD_EVERY as f32).round() as i32;
        (k - 1..=k + 1).any(|k| (c - (k * ROAD_EVERY + wobble(stage, k, t, off))).abs() <= ROAD_HALF)
    };
    near(cx, cy, 11) || near(cy, cx, 23)
}

fn clear(cx: i32, cy: i32) -> bool {
    ((cx * cx + cy * cy) as f32) < CLEAR * CLEAR
}

/// A cell's flags, computed from scratch (see `Terrain::cell` for the cached version).
pub fn cell_raw(stage: StageId, cx: i32, cy: i32) -> u8 {
    let mut n = [[0u8; 3]; 3];
    for (j, row) in n.iter_mut().enumerate() {
        for (i, v) in row.iter_mut().enumerate() {
            *v = level_raw(stage, cx + i as i32 - 1, cy + j as i32 - 1);
        }
    }
    flags(n, is_road(stage, cx, cy), clear(cx, cy))
}

/// A cell's flags from its 3x3 neighbourhood of levels (`n[1][1]` is the cell; `n[0]` is the
/// row above, i.e. north).
fn flags(n: [[u8; 3]; 3], road: bool, clear: bool) -> u8 {
    let me = n[1][1];
    let mut f = me;
    // Rim: lower ground to the north, west or east (or diagonally above).
    let rim = [n[0][0], n[0][1], n[0][2], n[1][0], n[1][2]].iter().any(|&l| l < me);
    // Under a south-facing rock face: higher ground right above.
    let under_face = n[0][1] > me;
    let change = n.iter().flatten().any(|&l| l != me);
    if road {
        f |= ROAD;
        if change {
            f |= RAMP;
        }
    } else if (rim || under_face) && !clear {
        f |= WALL;
    }
    f
}

pub fn cell_of(p: Vec2) -> (i32, i32) {
    ((p.x / CELL).floor() as i32, (p.y / CELL).floor() as i32)
}

/// Terrain state kept by the world: cached cells around the player and the flow field.
#[derive(Clone)]
pub struct Terrain {
    pub stage: StageId,
    pub on: bool,
    origin: (i32, i32),
    cells: Vec<u8>,
    flow_origin: (i32, i32),
    flow: Vec<u16>,
    /// Per flow cell: a straight line from its centre to the goal is clear of cliffs.
    sight: Vec<bool>,
    sight_goal: Vec2,
    sight_row: i32,
    /// Props per chunk over the cached window, minus any that would sit on a cliff or road.
    props_origin: (i32, i32),
    props_n: i32,
    props: Vec<ChunkProps>,
}

impl Terrain {
    pub fn new(stage: StageId) -> Self {
        let mut t = Terrain { stage, on: enabled(stage), origin: (0, 0), cells: Vec::new(), flow_origin: (0, 0), flow: Vec::new(), sight: Vec::new(), sight_goal: Vec2::ZERO, sight_row: FLOW, props_origin: (0, 0), props_n: 0, props: Vec::new() };
        t.ensure(Vec2::ZERO);
        t.update_flow(Vec2::ZERO);
        t.step_sight(FLOW);
        t
    }

    /// Keeps the cache centred near `centre` (refills it when the player wanders far).
    pub fn ensure(&mut self, centre: Vec2) {
        if !self.on {
            return;
        }
        let (cx, cy) = cell_of(centre);
        let (ox, oy) = (self.origin.0 + CACHE / 2, self.origin.1 + CACHE / 2);
        if !self.cells.is_empty() && (cx - ox).abs() < CACHE / 4 && (cy - oy).abs() < CACHE / 4 {
            return;
        }
        self.origin = (cx - CACHE / 2, cy - CACHE / 2);
        // Levels first (with a 1-cell border), then flags from neighbours: one noise read per cell.
        let n = (CACHE + 2) as usize;
        let mut lv = vec![0u8; n * n];
        let mut road = vec![false; n * n];
        for j in 0..n {
            for i in 0..n {
                let (x, y) = (self.origin.0 + i as i32 - 1, self.origin.1 + j as i32 - 1);
                lv[j * n + i] = level_raw(self.stage, x, y);
                road[j * n + i] = is_road(self.stage, x, y);
            }
        }
        self.cells = vec![0; (CACHE * CACHE) as usize];
        for j in 1..n - 1 {
            for i in 1..n - 1 {
                let mut nb = [[0u8; 3]; 3];
                for (dj, row) in nb.iter_mut().enumerate() {
                    for (di, v) in row.iter_mut().enumerate() {
                        *v = lv[(j + dj - 1) * n + (i + di - 1)];
                    }
                }
                let (x, y) = (self.origin.0 + i as i32 - 1, self.origin.1 + j as i32 - 1);
                self.cells[(j - 1) * CACHE as usize + (i - 1)] = flags(nb, road[j * n + i], clear(x, y));
            }
        }
        // Props for every chunk fully inside the window.
        let c0 = ((self.origin.0 as f32 * CELL / CHUNK).ceil() as i32, (self.origin.1 as f32 * CELL / CHUNK).ceil() as i32);
        let n = ((CACHE as f32 * CELL / CHUNK).floor() as i32 - 1).max(0);
        self.props_origin = c0;
        self.props_n = n;
        self.props = (0..n * n).map(|k| self.filtered_props(c0.0 + k % n, c0.1 + k / n)).collect();
    }

    /// A chunk's props: the stage's scatter minus any whose footprint (plus a cell of margin)
    /// touches a cliff or a road, then up to two lanterns (prop A) standing beside a trail.
    fn filtered_props(&self, cx: i32, cy: i32) -> ChunkProps {
        let (ps, n) = props_raw(self.stage, cx, cy);
        let mut out = [ps[0]; PROPS];
        if !self.on {
            out[..n].copy_from_slice(&ps[..n]);
            return (out, n);
        }
        let clear_of = |p: &Prop, flags: u8| {
            let (min, max) = p.footprint();
            let (x0, y0) = cell_of(min);
            let (x1, y1) = cell_of(max);
            (y0 - 1..=y1 + 1).all(|y| (x0 - 1..=x1 + 1).all(|x| self.cell(x, y) & flags == 0))
        };
        let mut m = 0;
        for p in &ps[..n] {
            if clear_of(p, WALL | ROAD) {
                out[m] = *p;
                m += 1;
            }
        }
        // Trailside lanterns: candidate spots on a coarse grid, kept if a road is 2 cells away
        // and the spot itself is clear. Footprints stay inside the chunk.
        let per = (CHUNK / CELL) as i32;
        let (bx, by) = (cx * per, cy * per);
        let mut placed = 0;
        let start = (hash(cx, cy, 61) % 9) as i32;
        for k in 0..9 {
            if placed == 2 || m == PROPS {
                break;
            }
            let s = (start + k * 4) % 9;
            let (gx, gy) = (bx + 2 + (s % 3) * 4, by + 3 + (s / 3) * 4);
            let near_road = [(-2, 0), (2, 0), (0, -2), (0, 2)].iter().any(|(dx, dy)| self.cell(gx + dx, gy + dy) & ROAD != 0);
            if !near_road {
                continue;
            }
            let p = Prop { kind: PropKind::A, pos: Vec2::new(gx as f32 * CELL + CELL / 2.0, gy as f32 * CELL + CELL / 2.0) };
            let too_close = out[..m].iter().any(|q| (q.pos - p.pos).length() < 40.0);
            if !too_close && clear_of(&p, WALL | ROAD) {
                out[m] = p;
                m += 1;
                placed += 1;
            }
        }
        (out, m)
    }

    /// A chunk's props (cached when inside the window).
    pub fn chunk_props(&self, cx: i32, cy: i32) -> ChunkProps {
        if !self.on {
            let (ps, n) = props_raw(self.stage, cx, cy);
            let mut out = [ps[0]; PROPS];
            out[..n].copy_from_slice(&ps[..n]);
            return (out, n);
        }
        let (i, j) = (cx - self.props_origin.0, cy - self.props_origin.1);
        if i >= 0 && j >= 0 && i < self.props_n && j < self.props_n {
            self.props[(j * self.props_n + i) as usize]
        } else {
            self.filtered_props(cx, cy)
        }
    }

    /// A cell's flags (from the cache when it covers the cell).
    pub fn cell(&self, cx: i32, cy: i32) -> u8 {
        if !self.on {
            return 1;
        }
        let (i, j) = (cx - self.origin.0, cy - self.origin.1);
        if i >= 0 && j >= 0 && i < CACHE && j < CACHE && !self.cells.is_empty() {
            self.cells[(j * CACHE + i) as usize]
        } else {
            cell_raw(self.stage, cx, cy)
        }
    }

    pub fn wall(&self, cx: i32, cy: i32) -> bool {
        self.cell(cx, cy) & WALL != 0
    }

    pub fn wall_at(&self, p: Vec2) -> bool {
        let (cx, cy) = cell_of(p);
        self.wall(cx, cy)
    }

    /// Pushes a circle out of every cliff cell it overlaps.
    pub fn resolve(&self, c: Vec2, r: f32) -> Vec2 {
        if !self.on {
            return c;
        }
        let mut c = c;
        let (x0, y0) = cell_of(Vec2::new(c.x - r, c.y - r));
        let (x1, y1) = cell_of(Vec2::new(c.x + r, c.y + r));
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                if self.wall(cx, cy) {
                    let min = Vec2::new(cx as f32 * CELL, cy as f32 * CELL);
                    c += crate::stage::push_out_box(c, r, (min, min + Vec2::new(CELL, CELL)));
                }
            }
        }
        c
    }

    /// Rebuilds the flow field: walking distance (in cells) from `goal` to every cell nearby.
    pub fn update_flow(&mut self, goal: Vec2) {
        if !self.on {
            return;
        }
        let (gx, gy) = cell_of(goal);
        self.flow_origin = (gx - FLOW / 2, gy - FLOW / 2);
        self.flow.clear();
        self.flow.resize((FLOW * FLOW) as usize, UNREACHED);
        let idx = |x: i32, y: i32| (y * FLOW + x) as usize;
        let mut q = VecDeque::with_capacity(4096);
        let start = (FLOW / 2, FLOW / 2);
        self.flow[idx(start.0, start.1)] = 0;
        q.push_back(start);
        while let Some((x, y)) = q.pop_front() {
            let d = self.flow[idx(x, y)];
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= FLOW || ny >= FLOW || self.flow[idx(nx, ny)] != UNREACHED {
                    continue;
                }
                if self.wall(self.flow_origin.0 + nx, self.flow_origin.1 + ny) {
                    continue;
                }
                self.flow[idx(nx, ny)] = d + 1;
                q.push_back((nx, ny));
            }
        }
        // Line of sight is refreshed a few rows per tick (`step_sight`), not all at once.
        if self.sight.len() != (FLOW * FLOW) as usize {
            self.sight = vec![false; (FLOW * FLOW) as usize];
        }
        self.sight_goal = goal;
        self.sight_row = 0;
    }

    /// Refreshes `rows` rows of the line-of-sight map (call every tick; a full pass takes
    /// FLOW / rows ticks), so enemies don't each trace a line every tick.
    pub fn step_sight(&mut self, rows: i32) {
        if !self.on || self.flow.is_empty() {
            return;
        }
        let end = (self.sight_row + rows).min(FLOW);
        for j in self.sight_row..end {
            for i in 0..FLOW {
                let k = (j * FLOW + i) as usize;
                self.sight[k] = self.flow[k] != UNREACHED && {
                    let c = Vec2::new((self.flow_origin.0 + i) as f32 * CELL + CELL / 2.0, (self.flow_origin.1 + j) as f32 * CELL + CELL / 2.0);
                    self.clear_line(c, self.sight_goal, 7.0)
                };
            }
        }
        self.sight_row = end;
    }

    /// Walking distance (cells) from the flow field's goal; None outside it or unreachable.
    pub fn distance(&self, p: Vec2) -> Option<u16> {
        let (cx, cy) = cell_of(p);
        let (i, j) = (cx - self.flow_origin.0, cy - self.flow_origin.1);
        if self.flow.is_empty() || i < 0 || j < 0 || i >= FLOW || j >= FLOW {
            return None;
        }
        let d = self.flow[(j * FLOW + i) as usize];
        (d != UNREACHED).then_some(d)
    }

    /// Direction to walk from `p` towards the flow goal, or None when the straight line is as
    /// good (no terrain in the way) or `p` is outside the field.
    pub fn steer(&self, p: Vec2, goal: Vec2) -> Option<Vec2> {
        if !self.on {
            return None;
        }
        let d = self.distance(p)?;
        let (cx, cy) = cell_of(p);
        let (i, j) = (cx - self.flow_origin.0, cy - self.flow_origin.1);
        if self.sight[(j * FLOW + i) as usize] {
            return None;
        }
        let _ = goal;
        // Downhill on the distance field (8 neighbours, so diagonals smooth the path).
        let mut best = (d, Vec2::ZERO);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if (dx, dy) == (0, 0) {
                    continue;
                }
                let q = Vec2::new(p.x + dx as f32 * CELL, p.y + dy as f32 * CELL);
                if let Some(nd) = self.distance(q) {
                    // A diagonal step only if both side cells are open (no corner cutting).
                    if dx != 0 && dy != 0 && (self.wall(cx + dx, cy) || self.wall(cx, cy + dy)) {
                        continue;
                    }
                    if nd < best.0 {
                        best = (nd, Vec2::new(dx as f32, dy as f32));
                    }
                }
            }
        }
        // Head for the centre of the next cell (not just its direction), so bodies wider than
        // a corner don't snag on it.
        (best.1 != Vec2::ZERO).then(|| {
            let target = Vec2::new((cx as f32 + best.1.x + 0.5) * CELL, (cy as f32 + best.1.y + 0.5) * CELL);
            (target - p).normalize_or_zero()
        })
    }

    /// True if a body of radius `r` can go straight from `a` to `b` without touching a cliff
    /// (sampled every half cell, at the centre line and both shoulders).
    pub fn clear_line(&self, a: Vec2, b: Vec2, r: f32) -> bool {
        let d = b - a;
        let len = d.length();
        if len < 1e-3 {
            return true;
        }
        let side = Vec2::new(-d.y, d.x) * (r / len);
        let steps = (len / (CELL * 0.5)).ceil() as i32;
        (0..=steps).all(|k| {
            let q = a + d * (k as f32 / steps as f32);
            !self.wall_at(q) && !self.wall_at(q + side) && !self.wall_at(q - side)
        })
    }

    /// The nearest open cell to `p` that can walk to the player (within `search` cells), or None.
    pub fn spawnable(&self, p: Vec2, search: i32) -> Option<Vec2> {
        if !self.on {
            return Some(p);
        }
        // Reachable if the flow field knows the cell (or there is no field yet).
        let ok = |q: Vec2| !self.wall_at(q) && (self.flow.is_empty() || self.distance(q).is_some());
        if ok(p) {
            return Some(p);
        }
        let (cx, cy) = cell_of(p);
        for r in 1..=search {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r {
                        continue;
                    }
                    let q = Vec2::new((cx + dx) as f32 * CELL + CELL / 2.0, (cy + dy) as f32 * CELL + CELL / 2.0);
                    if ok(q) {
                        return Some(q);
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const W1: StageId = StageId::ShrineValley;

    #[test]
    fn prologue_is_flat() {
        let t = Terrain::new(StageId::Shotengai);
        for y in -60..60 {
            for x in -60..60 {
                assert!(!t.wall(x, y));
            }
        }
    }

    #[test]
    fn cache_matches_raw() {
        let mut t = Terrain::new(W1);
        t.ensure(Vec2::new(3000.0, -1200.0));
        let (cx, cy) = cell_of(Vec2::new(3000.0, -1200.0));
        for y in cy - 40..cy + 40 {
            for x in cx - 40..cx + 40 {
                assert_eq!(t.cell(x, y), cell_raw(W1, x, y), "cell {x},{y}");
            }
        }
    }

    #[test]
    fn has_cliffs_and_open_start() {
        let t = Terrain::new(W1);
        let mut walls = 0;
        for y in -100..100 {
            for x in -100..100 {
                walls += t.wall(x, y) as u32;
            }
        }
        let share = walls as f32 / 40000.0;
        assert!(share > 0.04 && share < 0.35, "cliff share {share}");
        for y in -8..8 {
            for x in -8..8 {
                assert!(!t.wall(x, y), "start clearing blocked at {x},{y}");
            }
        }
    }

    #[test]
    fn roads_are_reachable_from_the_start() {
        let mut t = Terrain::new(W1);
        t.update_flow(Vec2::ZERO);
        // Every road cell in the flow window (except tiny cut-offs at its edge) is reachable.
        let (mut road, mut reached) = (0, 0);
        for y in -40..40 {
            for x in -40..40 {
                if t.cell(x, y) & ROAD != 0 {
                    road += 1;
                    reached += t.distance(Vec2::new(x as f32 * CELL + 1.0, y as f32 * CELL + 1.0)).is_some() as u32;
                }
            }
        }
        assert!(road > 100 && reached as f32 > road as f32 * 0.97, "{reached}/{road} road cells reachable");
    }

    #[test]
    fn walls_push_circles_out() {
        let t = Terrain::new(W1);
        let mut found = None;
        'outer: for y in -80..80 {
            for x in -80..80 {
                if t.wall(x, y) {
                    found = Some((x, y));
                    break 'outer;
                }
            }
        }
        let (x, y) = found.expect("a cliff");
        let inside = Vec2::new(x as f32 * CELL + 8.0, y as f32 * CELL + 8.0);
        let out = t.resolve(inside, 6.0);
        assert!((out - inside).length() > 1.0);
    }

    /// Writes each world's cell flags around the start, for debugging the layout:
    /// `KZ_TERRAIN_DUMP=/tmp cargo test -p sim dump_terrain -- --ignored`
    #[test]
    #[ignore]
    fn dump_terrain() {
        let dir = std::env::var("KZ_TERRAIN_DUMP").unwrap_or_else(|_| "/tmp".into());
        for stage in crate::campaign::ALL_STAGES.into_iter().filter(|s| enabled(*s)) {
            let t = Terrain::new(stage);
            let mut s = String::from("-80 -45 160 90\n");
            for y in -45..45 {
                let row: Vec<String> = (-80..80).map(|x| t.cell(x, y).to_string()).collect();
                s += &row.join(" ");
                s.push('\n');
            }
            std::fs::write(format!("{dir}/terrain_w{}.txt", stage.index()), s).unwrap();
        }
    }

    /// Prints a map of the start area (cargo test -p sim terrain::tests::map -- --nocapture).
    #[test]
    fn map() {
        let t = Terrain::new(W1);
        let mut s = String::new();
        for y in -30..30 {
            for x in -60..60 {
                let f = t.cell(x, y);
                s.push(if (x, y) == (0, 0) {
                    '@'
                } else if f & WALL != 0 {
                    '#'
                } else if f & RAMP != 0 {
                    '/'
                } else if f & ROAD != 0 {
                    '='
                } else {
                    [',', '.', '^'][(f & LEVEL) as usize]
                });
            }
            s.push('\n');
        }
        println!("{s}");
    }
}
