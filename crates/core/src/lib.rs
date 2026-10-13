//! pa_core: every rule of Packet Alley.
//!
//! Builder requirement: this crate performs no input or output. It takes
//! values and returns values, so `cargo test` reaches every rule without a
//! browser. `tests/boundary.rs` enforces that, along with the dependency
//! allowlist and the reference ban on floats, hash maps and clocks. It is
//! also why the simulated network cannot reach a real one: a packet here is a
//! value in a list, and nothing in this crate can send one anywhere.

pub mod api;
pub mod game;
pub mod ip;
pub mod level;
pub mod rng;
pub mod sim;
