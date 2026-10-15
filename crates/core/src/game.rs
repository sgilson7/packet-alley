//! The door the page calls through. A `Game` holds the level file, the copy,
//! the player's progress and the level being played; every method returns
//! JSON for the page to draw. Every reader-visible sentence comes from
//! `data/copy.json` or `data/levels.json`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::ip::{show_ip, show_mac, BROADCAST_MAC};
use crate::level::{self, Cond, Level, LevelFile};
use crate::sim::{Body, Proto, Record, Sim};

/// The map's size in its own units; node coordinates in the level file are
/// in these units.
pub const MAP_W: i32 = 1000;
pub const MAP_H: i32 = 600;
/// A node's icon, in map units, and the gaps between icon and its two labels.
pub const ICON: i32 = 76;
const LABEL_GAP: i32 = 16;
const LINE_GAP: i32 = 18;
/// How far a frame rides to the right of its cable, so the two directions of
/// one cable do not overlap.
const LANE: i64 = 11;
/// How far a backup cable bows away from the cable it shares ends with, so two
/// cables between the same pair of boxes read as two, not one.
const BOW: i64 = 72;
/// Playback time runs in thousandths of a tick.
const MILLI: i64 = 1000;
const SAVE_VERSION: u32 = 1;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Progress {
    pub v: u32,
    /// Per level: passed, the quiz answered right, the bonus met.
    pub stars: BTreeMap<String, [bool; 3]>,
    pub teacher: bool,
}

struct Play {
    level: usize,
    choice: Vec<usize>,
    record: Option<Record>,
    passed: bool,
}

pub struct Game {
    file: LevelFile,
    copy: BTreeMap<String, String>,
    progress: Progress,
    play: Option<Play>,
}

/// Every copy key the simulation's events use. `new` refuses a copy file
/// that lacks one.
pub const EVENT_KEYS: &[&str] = &[
    "ev_queue_full",
    "ev_no_gateway",
    "ev_arp_fail",
    "ev_arp_fail_gw",
    "ev_gave_up",
    "ev_retry",
    "ev_timeout",
    "ev_collision",
    "ev_collision_gave_up",
    "ev_moved",
    "ev_learn",
    "ev_ttl",
    "ev_no_route",
    "ev_arp_fail_router",
    "ev_route",
    "ev_ignore",
    "ev_arp_answer",
    "ev_arp_learned",
    "ev_not_mine",
    "ev_delivered",
    "ev_ping",
    "ev_pong",
    "ev_transfer_done",
    "ev_out_of_order",
    "ev_synack",
    "ev_established",
    "ev_bad_ack",
    "ev_request",
    "ev_reset",
    "ev_was_reset",
    "ev_served",
    "ev_lost",
];

/// The packet classes the page draws. Each has a letter and a name in the
/// copy, so no class is told apart by colour alone.
pub const CLASSES: &[&str] = &[
    "data",
    "ping",
    "reply",
    "broadcast",
    "arp",
    "syn",
    "synack",
    "ack",
    "seg",
    "rst",
];

/// The map effects, each with a symbol in the copy (`fx_*`).
pub const EFFECTS: &[&str] = &["deliver", "drop", "collision", "ignore", "learn"];

const OTHER_KEYS: &[&str] = &[
    "f_frame",
    "f_src_mac",
    "f_dst_mac",
    "f_broadcast",
    "f_arp",
    "f_arp_request",
    "f_arp_reply",
    "f_op",
    "f_sender",
    "f_target",
    "f_ip",
    "f_src_ip",
    "f_dst_ip",
    "f_ttl",
    "f_tcp",
    "f_ports",
    "f_flags",
    "f_seq",
    "f_ack",
    "f_data",
    "f_udp",
    "f_ping",
    "f_ping_request",
    "f_ping_reply",
    "f_label",
    "f_hop",
    "f_lost",
    "f_kind",
    "hud",
    "level_n",
    "stars_text",
    "total_stars",
    "ticks_used",
    "sum_frames",
    "sum_drops",
    "sum_collisions",
    "sum_retransmits",
    "passed",
    "not_yet",
    "quiz_right",
    "quiz_wrong",
    "why_nothing_lost",
    "log_tick",
    "err_no_level",
    "err_locked",
    "err_bad_choice",
    "err_not_run",
    "err_quiz_first",
];

fn fill(template: &str, args: &[String]) -> String {
    let mut out = template.to_string();
    for (key, a) in ["{a}", "{b}", "{c}", "{d}"].iter().zip(args) {
        out = out.replace(key, a);
    }
    out
}

fn isqrt(v: i64) -> i64 {
    if v <= 0 {
        return 0;
    }
    let mut x = v;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + v / x) / 2;
    }
    x
}

/// The control point of a cable's curve. Cables that share the same pair of
/// ends — the spare "backup" cables of the loop levels — would lie on top of
/// one another, so each is bowed to its own side and the student can tell them
/// apart. A lone cable's control point is its midpoint, which makes the
/// quadratic a straight line, so every other level draws exactly as before.
fn link_control(l: &Level, idx: usize) -> (i64, i64) {
    let k = &l.links[idx];
    let a = l.nodes.iter().find(|n| n.id == k.a).expect("validated");
    let b = l.nodes.iter().find(|n| n.id == k.b).expect("validated");
    let (ax, ay, bx, by) = (a.x as i64, a.y as i64, b.x as i64, b.y as i64);
    let (mx, my) = ((ax + bx) / 2, (ay + by) / 2);
    let same: Vec<usize> = l
        .links
        .iter()
        .enumerate()
        .filter(|(_, o)| (o.a == k.a && o.b == k.b) || (o.a == k.b && o.b == k.a))
        .map(|(j, _)| j)
        .collect();
    let count = same.len() as i64;
    if count <= 1 {
        return (mx, my);
    }
    let pos = same.iter().position(|&j| j == idx).expect("self is in the group") as i64;
    let bow = BOW * (2 * pos - (count - 1));
    let (dx, dy) = (bx - ax, by - ay);
    let len = isqrt(dx * dx + dy * dy).max(1);
    (mx - dy * bow / len, my + dx * bow / len)
}

/// Every level must build at its starting options, and with each option of
/// each edit in turn; ids, chapters and the quiz must line up.
pub fn validate(file: &LevelFile) -> Result<(), String> {
    for (i, l) in file.levels.iter().enumerate() {
        if file.levels[..i].iter().any(|o| o.id == l.id) {
            return Err(format!("two levels are called {}", l.id));
        }
        if !file.chapters.iter().any(|c| c.id == l.chapter) {
            return Err(format!("{}: no chapter {}", l.id, l.chapter));
        }
        if l.quiz.answer >= l.quiz.options.len() {
            return Err(format!("{}: the quiz answer is not one of its options", l.id));
        }
        if l.goal.is_empty() {
            return Err(format!("{}: a level needs a goal", l.id));
        }
        for c in l.goal.iter().chain(std::iter::once(&l.bonus)) {
            check_cond(l, c)?;
        }
        let mut start = l.clone();
        for e in &l.edits {
            if e.start >= e.options.len() {
                return Err(format!("{}: edit {} starts on a missing option", l.id, e.id));
            }
            level::apply(&mut start, &e.target, &e.options[e.start].value)
                .map_err(|m| format!("{}: {m}", l.id))?;
        }
        Sim::build(&start).map_err(|m| format!("{}: {m}", l.id))?;
        for e in &l.edits {
            for o in &e.options {
                let mut one = start.clone();
                level::apply(&mut one, &e.target, &o.value).map_err(|m| format!("{}: {m}", l.id))?;
                Sim::build(&one).map_err(|m| format!("{} with {} = {}: {m}", l.id, e.id, o.value))?;
            }
        }
    }
    Ok(())
}

fn check_cond(l: &Level, c: &Cond) -> Result<(), String> {
    let needs_app = matches!(c.kind.as_str(), "delivered" | "replies" | "transfer");
    let known = matches!(
        c.kind.as_str(),
        "delivered"
            | "replies"
            | "served"
            | "transfer"
            | "max_frames"
            | "no_drops"
            | "no_collisions"
            | "max_ticks"
            | "max_retransmits"
            | "observe"
    );
    if !known {
        return Err(format!("{}: unknown goal {}", l.id, c.kind));
    }
    if needs_app && !l.apps.iter().any(|a| a.id == c.app) {
        return Err(format!("{}: goal {} names no program {}", l.id, c.kind, c.app));
    }
    Ok(())
}

fn app_index(l: &Level, id: &str) -> usize {
    l.apps.iter().position(|a| a.id == id).unwrap_or(0)
}

/// Whether one condition holds for a finished run. `max_ticks` asks that
/// every other goal condition already held by that tick.
pub fn met(l: &Level, c: &Cond, rec: &Record) -> bool {
    let s = rec.stats();
    match c.kind.as_str() {
        "max_ticks" => {
            let others: Vec<&Cond> = l.goal.iter().filter(|g| g.kind != "max_ticks").collect();
            (0..rec.stats_at.len())
                .any(|t| t as u32 <= c.n && others.iter().all(|g| holds(l, g, &rec.stats_at[t], rec.ticks)))
        }
        _ => holds(l, c, s, rec.ticks),
    }
}

fn holds(l: &Level, c: &Cond, s: &crate::sim::Stats, _ticks: u32) -> bool {
    let a = app_index(l, &c.app);
    match c.kind.as_str() {
        "delivered" => s.delivered[a] >= c.n,
        "replies" => s.replies[a] >= c.n,
        "served" => {
            if c.app.is_empty() {
                s.served.iter().filter(|b| **b).count() as u32 >= c.n
            } else {
                s.served[a]
            }
        }
        "transfer" => s.transferred[a],
        "max_frames" => s.frames <= c.n,
        "no_drops" => s.drops == 0,
        "no_collisions" => s.collisions == 0,
        "max_retransmits" => s.retransmits <= c.n,
        // An explainer level is passed by watching it; there is nothing to
        // get right, so the goal always holds once the run has played.
        "observe" => true,
        _ => true,
    }
}

fn load_progress(saved: &str, file: &LevelFile) -> Progress {
    let mut p: Progress = serde_json::from_str(saved).unwrap_or_default();
    if p.v != SAVE_VERSION {
        p = Progress::default();
    }
    p.v = SAVE_VERSION;
    p.stars.retain(|id, _| file.levels.iter().any(|l| &l.id == id));
    p
}

impl Game {
    pub fn new(levels_json: &str, copy_json: &str, saved: &str) -> Result<Game, String> {
        let file = level::parse(levels_json)?;
        let copy: BTreeMap<String, String> =
            serde_json::from_str(copy_json).map_err(|e| format!("copy.json: {e}"))?;
        for k in EVENT_KEYS.iter().chain(OTHER_KEYS) {
            if !copy.contains_key(*k) {
                return Err(format!("copy.json has no {k}"));
            }
        }
        for c in CLASSES {
            if !copy.contains_key(&format!("pkt_{c}")) || !copy.contains_key(&format!("pkt_{c}_name")) {
                return Err(format!(
                    "copy.json has no letter or name for packets of class {c}"
                ));
            }
        }
        for f in EFFECTS {
            if !copy.contains_key(&format!("fx_{f}")) {
                return Err(format!("copy.json has no symbol for the effect {f}"));
            }
        }
        validate(&file)?;
        let progress = load_progress(saved, &file);
        Ok(Game {
            file,
            copy,
            progress,
            play: None,
        })
    }

    fn t(&self, key: &str) -> String {
        self.copy.get(key).cloned().unwrap_or_default()
    }

    fn unlocked(&self, i: usize) -> bool {
        if self.progress.teacher || i == 0 {
            return true;
        }
        let prev = &self.file.levels[i - 1].id;
        self.progress.stars.get(prev).map(|s| s[0]).unwrap_or(false)
    }

    fn stars_of(&self, id: &str) -> [bool; 3] {
        self.progress.stars.get(id).copied().unwrap_or([false; 3])
    }

    fn stars_text(&self, s: [bool; 3]) -> String {
        let got = s.iter().filter(|b| **b).count();
        fill(&self.t("stars_text"), &[got.to_string(), s.len().to_string()])
    }

    pub fn save(&self) -> String {
        serde_json::to_string(&self.progress).expect("progress serialises")
    }

    pub fn home(&self) -> String {
        let mut total = 0;
        let chapters: Vec<Value> = self
            .file
            .chapters
            .iter()
            .map(|c| {
                let levels: Vec<Value> = self
                    .file
                    .levels
                    .iter()
                    .enumerate()
                    .filter(|(_, l)| l.chapter == c.id)
                    .map(|(i, l)| {
                        let s = self.stars_of(&l.id);
                        total += s.iter().filter(|b| **b).count();
                        json!({
                            "id": l.id,
                            "title": l.title,
                            "number": fill(&self.t("level_n"), &[(i + 1).to_string()]),
                            "stars": s,
                            "stars_text": self.stars_text(s),
                            "locked": !self.unlocked(i),
                            "passed": s[0],
                        })
                    })
                    .collect();
                json!({"id": c.id, "title": c.title, "layer": c.layer, "blurb": c.blurb,
                       "backdrop": c.backdrop, "levels": levels})
            })
            .collect();
        let max = self.file.levels.len() * 3;
        json!({
            "chapters": chapters,
            "stars": fill(&self.t("total_stars"), &[total.to_string(), max.to_string()]),
            "teacher": self.progress.teacher,
        })
        .to_string()
    }

    pub fn set_teacher(&mut self, on: bool) -> String {
        self.progress.teacher = on;
        self.home()
    }

    pub fn reset(&mut self) -> String {
        self.progress = Progress {
            v: SAVE_VERSION,
            ..Progress::default()
        };
        self.play = None;
        self.home()
    }

    /// Opens a level with every edit at its starting option.
    pub fn open(&mut self, id: &str) -> Result<String, String> {
        let i = self
            .file
            .levels
            .iter()
            .position(|l| l.id == id)
            .ok_or_else(|| self.t("err_no_level"))?;
        if !self.unlocked(i) {
            return Err(self.t("err_locked"));
        }
        let choice = self.file.levels[i].edits.iter().map(|e| e.start).collect();
        self.play = Some(Play {
            level: i,
            choice,
            record: None,
            passed: false,
        });
        Ok(self.view())
    }

    /// Picks option `option` for edit `edit`. The last run is forgotten.
    pub fn choose(&mut self, edit: usize, option: usize) -> Result<String, String> {
        let i = self.play.as_ref().ok_or_else(|| self.t("err_no_level"))?.level;
        let ok = self.file.levels[i]
            .edits
            .get(edit)
            .map(|e| option < e.options.len())
            .unwrap_or(false);
        if !ok {
            return Err(self.t("err_bad_choice"));
        }
        let play = self.play.as_mut().expect("checked above");
        play.choice[edit] = option;
        play.record = None;
        play.passed = false;
        Ok(self.view())
    }

    fn edited(&self) -> Level {
        let play = self.play.as_ref().expect("a level is open");
        let base = &self.file.levels[play.level];
        let mut l = base.clone();
        for (e, &c) in base.edits.iter().zip(&play.choice) {
            level::apply(&mut l, &e.target, &e.options[c].value).expect("validated at load");
        }
        l
    }

    fn view(&self) -> String {
        let play = self.play.as_ref().expect("a level is open");
        let base = &self.file.levels[play.level];
        let l = self.edited();
        let chapter = self
            .file
            .chapters
            .iter()
            .find(|c| c.id == base.chapter)
            .expect("validated");
        let nodes: Vec<Value> = l
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| {
                let addr = if !n.ip.is_empty() {
                    n.ip.clone()
                } else {
                    let ips: Vec<&str> = n
                        .ifaces
                        .iter()
                        .map(|f| f.ip.split('/').next().unwrap_or(""))
                        .collect();
                    ips.join(" · ")
                };
                json!({
                    "i": i, "id": n.id, "kind": n.kind, "name": n.name,
                    "kind_name": self.t(&format!("kind_{}", n.kind)),
                    "x": n.x - ICON / 2, "y": n.y - ICON / 2, "size": ICON,
                    "cx": n.x, "cy": n.y,
                    "ly": n.y + ICON / 2 + LABEL_GAP,
                    "ly2": n.y + ICON / 2 + LABEL_GAP + LINE_GAP,
                    "addr": addr,
                })
            })
            .collect();
        let links: Vec<Value> = l
            .links
            .iter()
            .enumerate()
            .map(|(li, k)| {
                let a = l.nodes.iter().find(|n| n.id == k.a).expect("validated");
                let b = l.nodes.iter().find(|n| n.id == k.b).expect("validated");
                let blocked = l.nodes.iter().any(|n| n.block == k.id);
                let (cpx, cpy) = link_control(&l, li);
                json!({"id": k.id, "x1": a.x, "y1": a.y, "x2": b.x, "y2": b.y,
                       "cpx": cpx, "cpy": cpy,
                       "down": k.down, "lossy": k.lose_every > 0, "blocked": blocked,
                       "mx": (a.x + b.x) / 2, "my": (a.y + b.y) / 2})
            })
            .collect();
        let edits: Vec<Value> = base
            .edits
            .iter()
            .enumerate()
            .map(|(ei, e)| {
                // Which on-map control the edit becomes: a device palette, a
                // "point at a machine" pick, a number, a handshake flag, or a
                // cable/port. The page draws the right chooser from this.
                let field = e.target.field.as_str();
                let ui = match field {
                    "kind" => "palette",
                    "to" | "gateway" | "route" => "device",
                    "ttl" | "window" | "rto" => "number",
                    "first_flags" | "third_ack" => "flag",
                    "block" | "down" => "port",
                    _ => "menu",
                };
                // Where the control sits on the map: the device for a node
                // edit, the sender for a packet edit, the cable's middle for a
                // cable edit.
                let (ax, ay) = if !e.target.app.is_empty() {
                    l.apps
                        .iter()
                        .find(|a| a.id == e.target.app)
                        .and_then(|a| l.nodes.iter().find(|n| n.id == a.from))
                        .map(|n| (n.x, n.y))
                        .unwrap_or((0, 0))
                } else if !e.target.link.is_empty() {
                    l.links
                        .iter()
                        .find(|k| k.id == e.target.link)
                        .and_then(|k| {
                            let a = l.nodes.iter().find(|n| n.id == k.a)?;
                            let b = l.nodes.iter().find(|n| n.id == k.b)?;
                            Some(((a.x + b.x) / 2, (a.y + b.y) / 2))
                        })
                        .unwrap_or((0, 0))
                } else {
                    l.nodes
                        .iter()
                        .find(|n| n.id == e.target.node)
                        .map(|n| (n.x, n.y))
                        .unwrap_or((0, 0))
                };
                let opts: Vec<Value> = e
                    .options
                    .iter()
                    .enumerate()
                    .map(|(oi, o)| {
                        let mut m = json!({"label": o.label, "chosen": play.choice[ei] == oi});
                        let obj = m.as_object_mut().expect("object");
                        match ui {
                            // A device kind for the palette icons.
                            "palette" => {
                                obj.insert("nkind".into(), json!(o.value));
                            }
                            // The machine (or next-hop router) this address
                            // points at, so the student picks it by pointing.
                            "device" => {
                                let ip = o.value.split('/').next().unwrap_or(&o.value);
                                if !o.value.is_empty() {
                                    if let Some(n) = l.nodes.iter().find(|n| {
                                        n.ip.split('/').next() == Some(ip)
                                            || n.ifaces.iter().any(|f| f.ip.split('/').next() == Some(ip))
                                    }) {
                                        obj.insert("node".into(), json!(n.id));
                                        obj.insert("nkind".into(), json!(n.kind));
                                        obj.insert("nname".into(), json!(n.name));
                                    }
                                }
                            }
                            // Which cable the port edit blocks.
                            "port" => {
                                obj.insert("link".into(), json!(o.value));
                            }
                            _ => {}
                        }
                        m
                    })
                    .collect();
                json!({"prompt": e.prompt, "ui": ui, "field": field, "ax": ax, "ay": ay, "options": opts})
            })
            .collect();
        let goals: Vec<String> = base.goal.iter().map(|g| g.text.clone()).collect();
        let next = self.file.levels.get(play.level + 1).map(|n| n.id.clone());
        let legend: Vec<Value> = CLASSES
            .iter()
            .map(|c| json!({"cls": c, "letter": self.t(&format!("pkt_{c}")), "name": self.t(&format!("pkt_{c}_name"))}))
            .collect();
        let s = self.stars_of(&base.id);
        json!({
            "id": base.id, "title": base.title,
            "number": fill(&self.t("level_n"), &[(play.level + 1).to_string()]),
            "chapter": chapter.title, "layer": chapter.layer, "backdrop": chapter.backdrop,
            "story": base.story, "concept": base.concept, "task": base.task,
            "goals": goals, "bonus": base.bonus.text,
            "nodes": nodes, "links": links, "edits": edits,
            "viewbox": format!("0 0 {MAP_W} {MAP_H}"),
            "stars": s, "stars_text": self.stars_text(s), "next": next, "legend": legend,
        })
        .to_string()
    }

    /// Runs the open level with the chosen edits, scores it, and keeps the
    /// record for playback.
    pub fn run(&mut self) -> Result<String, String> {
        let i = self.play.as_ref().ok_or_else(|| self.t("err_no_level"))?.level;
        let l = self.edited();
        let rec = Sim::build(&l)?.run(l.ticks);
        let goals: Vec<(String, bool)> = l
            .goal
            .iter()
            .map(|g| (g.text.clone(), met(&l, g, &rec)))
            .collect();
        let passed = goals.iter().all(|g| g.1);
        let bonus = passed && met(&l, &l.bonus, &rec);
        let entry = self.progress.stars.entry(l.id.clone()).or_insert([false; 3]);
        entry[0] |= passed;
        entry[2] |= bonus;
        let diagnosis = if passed { Vec::new() } else { self.diagnose(&rec) };
        let s = rec.stats();
        let summary = vec![
            fill(&self.t("ticks_used"), &[rec.ticks.to_string()]),
            fill(&self.t("sum_frames"), &[s.frames.to_string()]),
            fill(&self.t("sum_drops"), &[s.drops.to_string()]),
            fill(&self.t("sum_collisions"), &[s.collisions.to_string()]),
            fill(&self.t("sum_retransmits"), &[s.retransmits.to_string()]),
        ];
        let base = &self.file.levels[i];
        let quiz = if passed {
            json!({"question": base.quiz.question, "options": base.quiz.options})
        } else {
            Value::Null
        };
        let stars = self.stars_of(&l.id);
        let out = json!({
            "passed": passed,
            "headline": self.t(if passed { "passed" } else { "not_yet" }),
            "goals": goals.iter().map(|g| json!({"text": g.0, "met": g.1})).collect::<Vec<_>>(),
            "bonus": {"text": l.bonus.text, "met": bonus},
            "summary": summary,
            "diagnosis": diagnosis,
            "debrief": if passed { base.debrief.clone() } else { String::new() },
            "quiz": quiz,
            "stars": stars,
            "stars_text": self.stars_text(stars),
            "ticks": rec.ticks,
        });
        let play = self.play.as_mut().expect("checked");
        play.record = Some(rec);
        play.passed = passed;
        Ok(out.to_string())
    }

    /// The first few different reasons frames were lost, each with what it
    /// usually means.
    fn diagnose(&self, rec: &Record) -> Vec<String> {
        let mut seen: Vec<&str> = Vec::new();
        let mut out = Vec::new();
        for e in rec
            .events
            .iter()
            .filter(|e| e.fx == "drop" || e.fx == "collision")
        {
            if seen.contains(&e.key) || seen.len() >= 3 {
                continue;
            }
            seen.push(e.key);
            let mut line = fill(&self.t(e.key), &e.args);
            let hint = self.t(&format!("why_{}", &e.key[3..]));
            if !hint.is_empty() {
                line = format!("{line} {hint}");
            }
            out.push(line);
        }
        if out.is_empty() {
            out.push(self.t("why_nothing_lost"));
        }
        out
    }

    pub fn answer(&mut self, option: usize) -> Result<String, String> {
        let play = self.play.as_ref().ok_or_else(|| self.t("err_no_level"))?;
        if !play.passed {
            return Err(self.t("err_quiz_first"));
        }
        let base = &self.file.levels[play.level];
        if option >= base.quiz.options.len() {
            return Err(self.t("err_bad_choice"));
        }
        let correct = option == base.quiz.answer;
        let id = base.id.clone();
        let why = base.quiz.why.clone();
        if correct {
            self.progress.stars.entry(id.clone()).or_insert([false; 3])[1] = true;
        }
        let stars = self.stars_of(&id);
        Ok(json!({
            "correct": correct,
            "headline": self.t(if correct { "quiz_right" } else { "quiz_wrong" }),
            "why": if correct { why } else { String::new() },
            "stars": stars,
            "stars_text": self.stars_text(stars),
        })
        .to_string())
    }

    /// The map at a moment of playback: `ms` since the run started, at
    /// `ms_per_tick`. Log lines from index `log_from` on that have happened.
    pub fn frame(&self, ms: u32, ms_per_tick: u32, log_from: usize) -> Result<String, String> {
        let play = self.play.as_ref().ok_or_else(|| self.t("err_no_level"))?;
        let rec = play.record.as_ref().ok_or_else(|| self.t("err_not_run"))?;
        let l = self.edited();
        let end = (rec.ticks as i64 + 1) * MILLI;
        let tt = (ms as i64 * MILLI / ms_per_tick.max(1) as i64).min(end);
        let tick = (tt / MILLI) as u32;
        let pos = |id: usize| (l.nodes[id].x as i64, l.nodes[id].y as i64);
        let mut packets = Vec::new();
        for (i, x) in rec.transits.iter().enumerate() {
            let (s, e) = (x.t0 as i64 * MILLI, x.t1 as i64 * MILLI);
            if tt < s || tt >= e {
                continue;
            }
            let k = (tt - s) * MILLI / (e - s);
            if x.lost && k >= MILLI / 2 {
                continue;
            }
            let (ax, ay) = pos(x.from);
            let (bx, by) = pos(x.to);
            let (dx, dy) = (bx - ax, by - ay);
            let len = isqrt(dx * dx + dy * dy).max(1);
            // Ride the cable's curve (a quadratic through its control point),
            // then step one lane to the right so the two directions do not sit
            // on top of each other. A straight cable's control point is its
            // midpoint, which makes the quadratic linear — the old path exactly.
            let (cx, cy) = link_control(&l, x.link);
            let (u, w) = (k, MILLI - k);
            let q = MILLI * MILLI;
            let px = (w * w * ax + 2 * w * u * cx + u * u * bx) / q - dy * LANE / len;
            let py = (w * w * ay + 2 * w * u * cy + u * u * by) / q + dx * LANE / len;
            let cls = x.frame.class();
            packets.push(
                json!({"id": i, "x": px, "y": py, "cls": cls, "letter": self.t(&format!("pkt_{cls}"))}),
            );
        }
        let mut effects = Vec::new();
        for e in rec.events.iter().filter(|e| e.t == tick && e.fx != "info") {
            let (x, y) = match e.transit {
                Some(ti) => {
                    let tr = &rec.transits[ti];
                    let (ax, ay) = pos(tr.from);
                    let (bx, by) = pos(tr.to);
                    ((ax + bx) / 2, (ay + by) / 2)
                }
                None => pos(e.node),
            };
            effects.push(json!({"x": x, "y": y, "fx": e.fx, "symbol": self.t(&format!("fx_{}", e.fx))}));
        }
        let log: Vec<Value> = rec
            .events
            .iter()
            .skip(log_from)
            .take_while(|e| e.t <= tick)
            .map(|e| {
                json!({"t": fill(&self.t("log_tick"), &[e.t.to_string()]), "text": fill(&self.t(e.key), &e.args),
                       "fx": e.fx, "symbol": self.t(&format!("fx_{}", e.fx))})
            })
            .collect();
        let s = &rec.stats_at[(tick as usize).min(rec.stats_at.len() - 1)];
        let hud = fill(
            &self.t("hud"),
            &[
                tick.min(rec.ticks).to_string(),
                s.frames.to_string(),
                s.drops.to_string(),
                s.collisions.to_string(),
            ],
        );
        Ok(json!({
            "tick": tick, "done": tt >= end, "packets": packets, "effects": effects,
            "log": log, "log_count": log_from + log.len(), "hud": hud,
        })
        .to_string())
    }

    /// The headers of one frame on one cable, layer by layer.
    pub fn packet(&self, id: usize) -> Result<String, String> {
        let play = self.play.as_ref().ok_or_else(|| self.t("err_no_level"))?;
        let rec = play.record.as_ref().ok_or_else(|| self.t("err_not_run"))?;
        let x = rec.transits.get(id).ok_or_else(|| self.t("err_bad_choice"))?;
        let l = self.edited();
        let f = &x.frame;
        let row = |k: &str, v: String| json!([self.t(k), v]);
        let cls = f.class();
        let mut layers = Vec::new();
        let dst = if f.dst_mac == BROADCAST_MAC {
            format!("{} {}", show_mac(f.dst_mac), self.t("f_broadcast"))
        } else {
            show_mac(f.dst_mac)
        };
        let hop = fill(
            &self.t("f_hop"),
            &[
                l.nodes[x.from].name.clone(),
                l.nodes[x.to].name.clone(),
                x.t0.to_string(),
            ],
        );
        let mut frame_rows = vec![row("f_kind", self.t(&format!("pkt_{cls}_name")))];
        frame_rows.push(json!(["", hop]));
        if x.lost {
            frame_rows.push(json!(["", self.t("f_lost")]));
        }
        frame_rows.push(row("f_src_mac", show_mac(f.src_mac)));
        frame_rows.push(row("f_dst_mac", dst));
        layers.push(json!({"name": self.t("f_frame"), "rows": frame_rows}));
        match &f.body {
            Body::Arp {
                reply,
                sender,
                target,
            } => {
                let op = self.t(if *reply { "f_arp_reply" } else { "f_arp_request" });
                layers.push(json!({"name": self.t("f_arp"), "rows": [
                    row("f_op", op), row("f_sender", show_ip(*sender)), row("f_target", show_ip(*target))]}));
            }
            Body::Ip(p) => {
                layers.push(json!({"name": self.t("f_ip"), "rows": [
                    row("f_src_ip", show_ip(p.src)), row("f_dst_ip", show_ip(p.dst)), row("f_ttl", p.ttl.to_string())]}));
                match &p.proto {
                    Proto::Udp => {
                        layers.push(
                            json!({"name": self.t("f_udp"), "rows": [row("f_label", p.label.clone())]}),
                        );
                    }
                    Proto::Ping { reply, n } => {
                        let k = self.t(if *reply { "f_ping_reply" } else { "f_ping_request" });
                        layers.push(
                            json!({"name": self.t("f_ping"), "rows": [row("f_op", format!("{k} #{n}"))]}),
                        );
                    }
                    Proto::Tcp {
                        flags,
                        sport,
                        dport,
                        seq,
                        ack,
                        data,
                    } => {
                        let mut rows = vec![
                            row("f_ports", format!("{sport} → {dport}")),
                            row("f_flags", flags.show()),
                            row("f_seq", seq.to_string()),
                            row("f_ack", ack.to_string()),
                        ];
                        if *data > 0 {
                            let what = if p.label.is_empty() {
                                data.to_string()
                            } else {
                                format!("{} #{data}", p.label)
                            };
                            rows.push(row("f_data", what));
                        }
                        layers.push(json!({"name": self.t("f_tcp"), "rows": rows}));
                    }
                }
            }
        }
        Ok(json!({"cls": cls, "letter": self.t(&format!("pkt_{cls}")), "layers": layers}).to_string())
    }

    /// What a node knows at a moment of playback (before a run: at the start).
    pub fn node(&self, i: usize, ms: u32, ms_per_tick: u32) -> Result<String, String> {
        let play = self.play.as_ref().ok_or_else(|| self.t("err_no_level"))?;
        let l = self.edited();
        let n = l.nodes.get(i).ok_or_else(|| self.t("err_bad_choice"))?;
        let lines = match &play.record {
            Some(rec) => {
                let tick = (ms as i64 / ms_per_tick.max(1) as i64) as usize;
                rec.tables[tick.min(rec.tables.len() - 1)][i].clone()
            }
            None => {
                let rec = Sim::build(&l)?.run(0);
                rec.tables[0][i].clone()
            }
        };
        Ok(
            json!({"name": n.name, "kind": n.kind, "kind_name": self.t(&format!("kind_{}", n.kind)),
                  "blurb": self.t(&format!("about_{}", n.kind)), "lines": lines})
            .to_string(),
        )
    }
}
