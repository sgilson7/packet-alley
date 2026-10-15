//! The one door the shim calls. Every function returns a JSON string:
//! `{"ok": ...}` on success or `{"error": "..."}` with a message the page
//! shows as it is.

use crate::game::Game;

fn wrap(r: Result<String, String>) -> String {
    match r {
        Ok(s) => format!("{{\"ok\":{s}}}"),
        Err(e) => serde_json::json!({ "error": e }).to_string(),
    }
}

pub struct Api {
    game: Option<Game>,
    error: String,
}

impl Api {
    /// A game from the level file, the copy and a save (which may be empty or
    /// unreadable: then progress starts fresh).
    pub fn new(levels: &str, copy: &str, saved: &str) -> Api {
        match Game::new(levels, copy, saved) {
            Ok(g) => Api {
                game: Some(g),
                error: String::new(),
            },
            Err(e) => Api { game: None, error: e },
        }
    }

    fn with<F: FnOnce(&mut Game) -> Result<String, String>>(&mut self, f: F) -> String {
        match self.game.as_mut() {
            Some(g) => wrap(f(g)),
            None => wrap(Err(self.error.clone())),
        }
    }

    pub fn home(&mut self) -> String {
        self.with(|g| Ok(g.home()))
    }
    pub fn save(&mut self) -> String {
        self.with(|g| Ok(serde_json::json!(g.save()).to_string()))
    }
    pub fn set_teacher(&mut self, on: bool) -> String {
        self.with(|g| Ok(g.set_teacher(on)))
    }
    pub fn reset(&mut self) -> String {
        self.with(|g| Ok(g.reset()))
    }
    pub fn open(&mut self, id: &str) -> String {
        self.with(|g| g.open(id))
    }
    pub fn choose(&mut self, edit: u32, option: u32) -> String {
        self.with(|g| g.choose(edit as usize, option as usize))
    }
    pub fn scene(&mut self, chapter: &str) -> String {
        self.with(|g| g.scene(chapter))
    }
    pub fn scene_done(&mut self, chapter: &str) -> String {
        self.with(|g| Ok(g.scene_done(chapter)))
    }
    pub fn inspect(&mut self, id: &str) -> String {
        self.with(|g| g.inspect(id))
    }
    pub fn predict(&mut self, option: u32) -> String {
        self.with(|g| g.predict(option as usize))
    }
    pub fn practice(&mut self) -> String {
        self.with(|g| g.practice())
    }
    pub fn run(&mut self) -> String {
        self.with(|g| g.run())
    }
    pub fn answer(&mut self, option: u32) -> String {
        self.with(|g| g.answer(option as usize))
    }
    pub fn frame(&mut self, ms: u32, ms_per_tick: u32, log_from: u32) -> String {
        self.with(|g| g.frame(ms, ms_per_tick, log_from as usize))
    }
    pub fn packet(&mut self, id: u32) -> String {
        self.with(|g| g.packet(id as usize))
    }
    pub fn node(&mut self, i: u32, ms: u32, ms_per_tick: u32) -> String {
        self.with(|g| g.node(i as usize, ms, ms_per_tick))
    }

    // Build levels (end-of-zone "super" levels).
    pub fn build_state(&mut self) -> String {
        self.with(|g| Ok(g.build_state()))
    }
    pub fn place(&mut self, kind: &str, x: i32, y: i32) -> String {
        self.with(|g| g.place(kind, x, y))
    }
    pub fn unplace(&mut self, id: &str) -> String {
        self.with(|g| g.unplace(id))
    }
    pub fn wire(&mut self, a: &str, b: &str) -> String {
        self.with(|g| g.wire(a, b))
    }
    pub fn unwire(&mut self, a: &str, b: &str) -> String {
        self.with(|g| g.unwire(a, b))
    }
    pub fn set_addr(&mut self, node: &str, ip: &str) -> String {
        self.with(|g| g.set_addr(node, ip))
    }
    pub fn set_port(&mut self, switch: &str, dst: &str, via: &str) -> String {
        self.with(|g| g.set_port(switch, dst, via))
    }
}
