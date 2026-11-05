//! Pure game simulation: no rendering, no platform code. Deterministic for a given seed.

pub mod bot;
pub mod content;
pub mod grid;
pub mod math;
pub mod meta;
pub mod rng;
pub mod stage;
pub mod world;

pub use math::Vec2;
pub use world::{Event, Input, RunState, Stress, World, WorldConfig};
