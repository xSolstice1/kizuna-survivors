//! Foliage that moves: grass tufts that sway in the wind and part around the hero.
//!
//! Grass is laid out by a hash of its grid cell (no state): one tuft per cell at most, off
//! roads and away from cliff edges, so the same field appears every time you pass. Each tuft
//! is a quad whose top bends with gusts of wind rolling across the field and away from the
//! hero within reach; tufts are batched per texture into one mesh. They're drawn on the ground
//! before the light map, so lanterns light them. Art: tools/ai/foliage_art.py.
use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::prelude::*;
use sim::campaign::StageId;
use sim::terrain::{height, road_distance, HIGH_ABOVE, LOW_BELOW};

use crate::sprites;

/// World units per grass cell; each holds at most one tuft.
const CELL: f32 = 20.0;
/// How far the hero parts the grass.
const PART: f32 = 20.0;

/// A world's grass: textures, how many cells hold a tuft, and tuft height (world units).
pub struct Grass {
    tex: Vec<Texture2D>,
    density: f32,
    height: f32,
}

impl Grass {
    pub fn load(stage: StageId) -> Option<Grass> {
        let (dir, density, height) = match stage {
            StageId::ShrineValley => ("w1", 0.30, 13.0),
            StageId::EmberLeaf => ("w3", 0.22, 13.0),
            _ => return None,
        };
        let tex: Vec<Texture2D> = (0..4).map(|i| format!("{dir}/grass_{i}")).filter(|p| sprites::has_art(p)).map(|p| sprites::hi_art(&p)).collect();
        (!tex.is_empty()).then_some(Grass { tex, density, height })
    }

    /// Draws every tuft in `view`. `hero` parts the grass; `time` drives the wind.
    pub fn draw(&self, stage: StageId, view: Rect, hero: Vec2, time: f64) {
        let t = time as f32;
        // Low quality: half the grass.
        let density = if crate::post::quality() == crate::post::Quality::Low { self.density * 0.5 } else { self.density };
        let mut batches: Vec<(Vec<Vertex>, Vec<u16>)> = vec![(Vec::new(), Vec::new()); self.tex.len()];
        let (cx0, cy0) = ((view.x / CELL).floor() as i32 - 1, (view.y / CELL).floor() as i32 - 1);
        let (cx1, cy1) = (((view.x + view.w) / CELL).ceil() as i32 + 1, ((view.y + view.h) / CELL).ceil() as i32 + 1);
        for cy in cy0..=cy1 {
            for cx in cx0..=cx1 {
                let h = sim::stage::hash(cx, cy, 0x6A55);
                if (h & 0xFFFF) as f32 / 65535.0 > density {
                    continue;
                }
                let j = |s: u32| ((h >> s) & 0xFF) as f32 / 255.0;
                let p = vec2((cx as f32 + 0.15 + 0.7 * j(8)) * CELL, (cy as f32 + 0.15 + 0.7 * j(16)) * CELL);
                let sp = sim::Vec2::new(p.x, p.y);
                // Not on roads, and not on or near a cliff edge (its face or lip).
                if road_distance(stage, sp) < 2.1 {
                    continue;
                }
                let lv = height(stage, sp);
                if (lv - LOW_BELOW).abs() < 0.03 || (lv - HIGH_ABOVE).abs() < 0.03 {
                    continue;
                }
                // Nor on a rock face: the strip below any edge with higher ground to the north.
                let level = |h: f32| (h > LOW_BELOW) as u8 + (h > HIGH_ABOVE) as u8;
                if (1..=5).any(|k| level(height(stage, sim::Vec2::new(p.x, p.y - 8.0 * k as f32))) > level(lv)) {
                    continue;
                }
                let tex = (h >> 24) as usize % self.tex.len();
                let tx = &self.tex[tex];
                let hgt = self.height * (0.75 + 0.5 * j(4));
                let w = hgt * tx.width() / tx.height();
                // Wind: slow gusts rolling across the field, and a quicker flutter per tuft.
                let gust = crate::wind::gust(p, t);
                let mut bend = (gust * 0.24 * crate::wind::DIR.x.signum() + 0.04 * (t * 3.1 + j(12) * 6.0).sin()) * hgt;
                // The hero parts the grass: tops lean away, more the closer.
                let d = p - hero;
                let near = (1.0 - d.length() / PART).max(0.0);
                bend += d.x.signum() * near * near * hgt * 0.7;
                let squash = 1.0 - near * 0.35;
                let flip = j(20) > 0.5;
                let (u0, u1) = if flip { (1.0, 0.0) } else { (0.0, 1.0) };
                let top = p.y - hgt * squash;
                let (vs, is) = &mut batches[tex];
                let b = vs.len() as u16;
                if b > 60000 {
                    continue;
                }
                vs.push(Vertex::new(p.x - w / 2.0 + bend, top, 0.0, u0, 0.0, WHITE));
                vs.push(Vertex::new(p.x + w / 2.0 + bend, top, 0.0, u1, 0.0, WHITE));
                vs.push(Vertex::new(p.x + w / 2.0, p.y, 0.0, u1, 1.0, WHITE));
                vs.push(Vertex::new(p.x - w / 2.0, p.y, 0.0, u0, 1.0, WHITE));
                is.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
            }
        }
        for (i, (vertices, indices)) in batches.into_iter().enumerate() {
            if !vertices.is_empty() {
                draw_mesh(&Mesh { vertices, indices, texture: Some(self.tex[i].clone()) });
            }
        }
    }
}
