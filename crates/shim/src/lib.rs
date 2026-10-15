//! pa_shim: forwards values between the page and `pa_core::api`.
//!
//! Builder requirement: the shim decides nothing. Each function is one call
//! into the core. `crates/core/tests/boundary.rs` fails if a branch, a loop or
//! the `?` operator appears here.

use pa_core::api::Api;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Game {
    inner: Api,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(levels: &str, copy: &str, saved: &str) -> Game {
        Game {
            inner: Api::new(levels, copy, saved),
        }
    }
    pub fn home(&mut self) -> String {
        self.inner.home()
    }
    pub fn save(&mut self) -> String {
        self.inner.save()
    }
    pub fn set_teacher(&mut self, on: bool) -> String {
        self.inner.set_teacher(on)
    }
    pub fn reset(&mut self) -> String {
        self.inner.reset()
    }
    pub fn open(&mut self, id: &str) -> String {
        self.inner.open(id)
    }
    pub fn choose(&mut self, edit: u32, option: u32) -> String {
        self.inner.choose(edit, option)
    }
    pub fn inspect(&mut self, id: &str) -> String {
        self.inner.inspect(id)
    }
    pub fn predict(&mut self, option: u32) -> String {
        self.inner.predict(option)
    }
    pub fn practice(&mut self) -> String {
        self.inner.practice()
    }
    pub fn run(&mut self) -> String {
        self.inner.run()
    }
    pub fn answer(&mut self, option: u32) -> String {
        self.inner.answer(option)
    }
    pub fn frame(&mut self, ms: u32, ms_per_tick: u32, log_from: u32) -> String {
        self.inner.frame(ms, ms_per_tick, log_from)
    }
    pub fn packet(&mut self, id: u32) -> String {
        self.inner.packet(id)
    }
    pub fn node(&mut self, i: u32, ms: u32, ms_per_tick: u32) -> String {
        self.inner.node(i, ms, ms_per_tick)
    }
}
