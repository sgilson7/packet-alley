//! The door the page calls through. A `Game` holds the level file, the copy,
//! the player's progress and the level being played; every method returns
//! JSON for the page to draw. Every reader-visible sentence comes from
//! `data/copy.json` or `data/levels.json`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::ip::{mask, parse_cidr, show_ip, show_mac, Cidr, BROADCAST_MAC};
use crate::level::{self, Claim, Cond, Level, LevelFile};
use crate::sim::{Body, FactVal, Proto, Record, Sim};

/// The map's size in its own units; node coordinates in the level file are
/// in these units.
pub const MAP_W: i32 = 1000;
pub const MAP_H: i32 = 600;
/// A node's icon, in map units, and the gaps between icon and its two labels.
pub const ICON: i32 = 100;
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
    /// Which chapters' demonstration scenes have been watched (Phase 2).
    #[serde(default)]
    pub scenes_seen: BTreeMap<String, bool>,
    /// Which build levels earned the hidden "gold" (cheapest few per cent) star.
    #[serde(default)]
    pub gold: BTreeMap<String, bool>,
}

struct Play {
    level: usize,
    choice: Vec<usize>,
    record: Option<Record>,
    passed: bool,
    /// Evidence-first (Phase 2): node/cable ids the student has inspected, the
    /// outcome they committed to, and which fixed practice variant is active
    /// (`None` = the scored base case). A level with no `predict` ignores all
    /// three and plays exactly as before.
    evidence_seen: Vec<String>,
    prediction: Option<usize>,
    variant: Option<usize>,
    /// A build level's working state (`None` on an ordinary level): the
    /// machines the student has placed, the cables drawn, the addresses
    /// assigned, and each switch's hand-filled table.
    build: Option<BuildState>,
}

/// What the student has built so far on a build level.
#[derive(Default)]
struct BuildState {
    placed: Vec<Placed>,
    next: u32,
    /// Each cable as a pair of node ids (fixed or placed).
    cables: Vec<(String, String)>,
    /// Host id → the CIDR address the student chose for it.
    addrs: BTreeMap<String, String>,
    /// Switch id → its table: (destination host id, the neighbour to send toward).
    tables: BTreeMap<String, Vec<(String, String)>>,
    /// Router id → its forwarding table: (destination block prefix, the
    /// neighbouring router to send toward). The routed-build analogue of a
    /// switch's MAC table — the student fills every row.
    routes: BTreeMap<String, Vec<(String, String)>>,
    /// The hop count (TTL) the student chose for the terminals; 0 = not yet set.
    ttl: u32,
}

struct Placed {
    id: String,
    kind: String,
    name: String,
    x: i32,
    y: i32,
}

/// The addressing a routed build derives from the student's placement and
/// wiring: every router interface's address, every host's gateway, and, for
/// each router, its directly-connected neighbour routers, the address to send
/// toward each of them, and the remote blocks it still needs a signpost for.
/// The student never types an address — they choose the box, the cable and the
/// next-hop neighbour; the engine numbers the network underneath.
#[derive(Default)]
struct BuildNet {
    /// router id → its interfaces: (cable id, address as CIDR).
    iface: BTreeMap<String, Vec<(String, String)>>,
    /// host/server id → its gateway address (the router on its own block).
    gw: BTreeMap<String, String>,
    /// router id → the router ids it shares a point-to-point link with.
    rneigh: BTreeMap<String, Vec<String>>,
    /// (router id, neighbour router id) → the neighbour's address on their link
    /// (the next hop this router forwards toward).
    via_ip: BTreeMap<(String, String), String>,
    /// router id → the blocks it is not directly on, each needing a signpost:
    /// (prefix, friendly name).
    remote: BTreeMap<String, Vec<(String, String)>>,
}

/// The base address the engine numbers point-to-point router links from:
/// 10.0.254.0, stepping by four (a /30) per link. Chosen to sit clear of the
/// neighbourhood blocks a level declares.
const P2P_BASE: u32 = 0x0A00_FE00;

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
    "ev_no_entry",
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
    "err_inspect_first",
    "err_predict_first",
    "tbl_ip",
    "tbl_mac",
    "tbl_gateway",
    "tbl_none",
    "tbl_ports",
    "tbl_open_conns",
    "tbl_waiting",
    "tbl_mac_table",
    "tbl_mac_table_empty",
    "tbl_blocked",
    "tbl_ifaces",
    "tbl_routes",
    "tbl_routes_empty",
    "tbl_col_mac",
    "tbl_col_port",
    "tbl_col_ip",
    "tbl_col_block",
    "tbl_col_dest",
    "tbl_col_via",
    "err_build_max",
    "err_build_obstacle",
    "err_build_too_long",
    "err_build_wired",
    "err_build_crosses",
];

/// The action category a log line belongs to, so the page can filter the log
/// by what happened (a forward, a loss, a collision…). Most follow the event's
/// map effect (`fx`); a few keys are split out into their own category so a
/// student can pick them apart.
fn log_kind(key: &str, fx: &str) -> &'static str {
    match key {
        "ev_route" => "forward",
        "ev_retry" | "ev_timeout" | "ev_synack" => "retry",
        _ => match fx {
            "deliver" => "deliver",
            "drop" => "drop",
            "collision" => "collision",
            "ignore" => "ignore",
            "learn" => "learn",
            _ => "other",
        },
    }
}

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
        // A build level starts empty (the student lays the cables), so it is
        // not built from its starting edits; its own rules are checked instead.
        if let Some(b) = &l.build {
            check_build(l, b)?;
            continue;
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
        // Phase 2: every prediction claim names a real device or program,
        // every evidence id a real device or cable, and every fixed practice
        // variant builds with the level's starting edits.
        if let Some(pd) = &l.predict {
            for o in &pd.options {
                check_claim(l, &o.claim)?;
            }
        }
        for id in &l.evidence {
            let known = l.nodes.iter().any(|n| &n.id == id) || l.links.iter().any(|k| &k.id == id);
            if !known {
                return Err(format!("{}: evidence names no device or cable {id}", l.id));
            }
        }
        for v in &l.variants {
            let mut vl = l.clone();
            vl.seed = v.seed;
            for o in &v.overrides {
                level::apply(&mut vl, &o.target, &o.value)
                    .map_err(|m| format!("{}: a variant: {m}", l.id))?;
            }
            // the edits as the variant presents them, with its option overrides
            let mut edits = l.edits.clone();
            for ov in &v.edits {
                let e = edits
                    .iter_mut()
                    .find(|e| e.id == ov.edit)
                    .ok_or_else(|| format!("{}: a variant names no edit {}", l.id, ov.edit))?;
                if ov.options.len() != e.options.len() {
                    return Err(format!(
                        "{}: a variant gives edit {} {} options, not {}",
                        l.id,
                        ov.edit,
                        ov.options.len(),
                        e.options.len()
                    ));
                }
                e.options = ov.options.clone();
            }
            let mut start = vl.clone();
            for e in &edits {
                level::apply(&mut start, &e.target, &e.options[e.start].value)
                    .map_err(|m| format!("{}: a variant: {m}", l.id))?;
            }
            Sim::build(&start).map_err(|m| format!("{}: a variant: {m}", l.id))?;
            // every option of a variant edit must build too
            for e in &edits {
                for o in &e.options {
                    let mut one = start.clone();
                    level::apply(&mut one, &e.target, &o.value)
                        .map_err(|m| format!("{}: a variant: {m}", l.id))?;
                    Sim::build(&one)
                        .map_err(|m| format!("{}: a variant {} = {}: {m}", l.id, e.id, o.value))?;
                }
            }
        }
    }
    Ok(())
}

fn check_claim(l: &Level, claim: &Claim) -> Result<(), String> {
    let node = |id: &String| -> Result<(), String> {
        if l.nodes.iter().any(|n| &n.id == id) {
            Ok(())
        } else {
            Err(format!("{}: a prediction names no device {id}", l.id))
        }
    };
    let app = |id: &String| -> Result<(), String> {
        if l.apps.iter().any(|a| &a.id == id) {
            Ok(())
        } else {
            Err(format!("{}: a prediction names no program {id}", l.id))
        }
    };
    match claim {
        Claim::Delivered { app: a }
        | Claim::NotDelivered { app: a }
        | Claim::Served { app: a }
        | Claim::Replies { app: a }
        | Claim::Transferred { app: a } => app(a),
        Claim::DeliveredTo { node: n }
        | Claim::FailsAt { node: n }
        | Claim::ExpiresAt { node: n }
        | Claim::QueueFull { node: n } => node(n),
        Claim::NoLoss => Ok(()),
    }
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

/// Does the segment (ax,ay)-(bx,by) touch the axis-aligned obstacle rectangle?
/// All integer: the cross product orders points around a turn, and two
/// segments cross when each straddles the other (plus collinear-overlap cases).
fn seg_hits_rect(ax: i32, ay: i32, bx: i32, by: i32, o: &level::Obstacle) -> bool {
    let (ox, oy, rx, ry) = (o.x as i64, o.y as i64, (o.x + o.w) as i64, (o.y + o.h) as i64);
    let inside = |x: i64, y: i64| x >= ox && x <= rx && y >= oy && y <= ry;
    let (ax, ay, bx, by) = (ax as i64, ay as i64, bx as i64, by as i64);
    if inside(ax, ay) || inside(bx, by) {
        return true;
    }
    let orient =
        |px: i64, py: i64, qx: i64, qy: i64, cx: i64, cy: i64| (qx - px) * (cy - py) - (qy - py) * (cx - px);
    let on = |px: i64, py: i64, qx: i64, qy: i64, cx: i64, cy: i64| {
        cx >= px.min(qx) && cx <= px.max(qx) && cy >= py.min(qy) && cy <= py.max(qy)
    };
    let cross = |cx: i64, cy: i64, dx: i64, dy: i64| {
        let o1 = orient(ax, ay, bx, by, cx, cy);
        let o2 = orient(ax, ay, bx, by, dx, dy);
        let o3 = orient(cx, cy, dx, dy, ax, ay);
        let o4 = orient(cx, cy, dx, dy, bx, by);
        if (o1 > 0) != (o2 > 0) && (o3 > 0) != (o4 > 0) {
            return true;
        }
        (o1 == 0 && on(ax, ay, bx, by, cx, cy))
            || (o2 == 0 && on(ax, ay, bx, by, dx, dy))
            || (o3 == 0 && on(cx, cy, dx, dy, ax, ay))
            || (o4 == 0 && on(cx, cy, dx, dy, bx, by))
    };
    let c = [(ox, oy), (rx, oy), (rx, ry), (ox, ry)];
    (0..c.len()).any(|i| {
        let (cx, cy) = c[i];
        let (dx, dy) = c[(i + 1) % c.len()];
        cross(cx, cy, dx, dy)
    })
}

/// Checks a build level's own rules: the subnet and every offered address
/// parse and sit on that block, the palette and the costs name real machine
/// kinds, the machines that need an address exist, and the budget thresholds
/// go cheapest first.
fn check_build(l: &Level, b: &level::BuildDef) -> Result<(), String> {
    let kinds = ["host", "printer", "server", "hub", "switch", "router"];
    parse_cidr(&b.subnet).ok_or_else(|| format!("{}: bad subnet {}", l.id, b.subnet))?;
    // Choices may sit off the block on purpose (a distractor that teaches the
    // one-subnet rule), so only that they parse is checked here.
    for a in &b.addr_choices {
        parse_cidr(a).ok_or_else(|| format!("{}: bad address {a}", l.id))?;
    }
    for p in &b.palette {
        if !kinds.contains(&p.kind.as_str()) {
            return Err(format!("{}: palette has unknown kind {}", l.id, p.kind));
        }
    }
    // Every machine kind that can appear (fixed endpoint or placed) needs a price.
    for kind in l
        .nodes
        .iter()
        .map(|n| &n.kind)
        .chain(b.palette.iter().map(|p| &p.kind))
    {
        if !b.costs.iter().any(|c| &c.kind == kind) {
            return Err(format!("{}: no cost for a {kind}", l.id));
        }
    }
    for id in &b.needs_addr {
        if !l.nodes.iter().any(|n| &n.id == id) {
            return Err(format!("{}: needs_addr names no machine {id}", l.id));
        }
    }
    if b.star_cost.windows(2).any(|w| w[0] > w[1]) {
        return Err(format!("{}: star_cost must go cheapest first", l.id));
    }
    Ok(())
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

/// Whether a committed prediction came true. The claim is checked against the
/// run's recorded events and stats — never a stored answer index — so a correct
/// prediction means the student read the network, not that they guessed an
/// option position (Phase 2, PA8).
pub fn graded(l: &Level, claim: &Claim, rec: &Record) -> bool {
    let s = rec.stats();
    let idx = |id: &str| l.nodes.iter().position(|n| n.id == id);
    let at = |id: &str, pred: &dyn Fn(&crate::sim::Event) -> bool| match idx(id) {
        Some(i) => rec.events.iter().any(|e| e.node == i && pred(e)),
        None => false,
    };
    match claim {
        Claim::Delivered { app } => s.delivered[app_index(l, app)] >= 1,
        Claim::NotDelivered { app } => s.delivered[app_index(l, app)] == 0,
        Claim::Served { app } => s.served[app_index(l, app)],
        Claim::Replies { app } => s.replies[app_index(l, app)] >= 1,
        Claim::Transferred { app } => s.transferred[app_index(l, app)],
        Claim::DeliveredTo { node } => at(node, &|e| e.key == "ev_delivered"),
        Claim::FailsAt { node } => at(node, &|e| e.fx == "drop" || e.fx == "collision"),
        Claim::ExpiresAt { node } => at(node, &|e| e.key == "ev_ttl"),
        Claim::QueueFull { node } => at(node, &|e| e.key == "ev_queue_full"),
        Claim::NoLoss => s.drops == 0 && s.collisions == 0,
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
        let l = &self.file.levels[i];
        // A "super" build level opens once its whole zone is passed, wherever
        // it sits in the file; an ordinary level opens after the one before it.
        if l.build.is_some() {
            return self
                .file
                .levels
                .iter()
                .filter(|o| o.chapter == l.chapter && !o.scene && o.build.is_none())
                .all(|o| self.stars_of(&o.id)[0]);
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
                    .filter(|(_, l)| l.chapter == c.id && !l.scene && l.build.is_none())
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
                // The chapter's demonstration scene, if any, and whether it has
                // been watched (Phase 2).
                let scene = self.file.levels.iter().find(|l| l.scene && l.chapter == c.id);
                let scene_info = json!({
                    "id": scene.map(|l| l.id.clone()),
                    "seen": self.progress.scenes_seen.get(&c.id).copied().unwrap_or(false),
                });
                // The end-of-zone "super" build level, if the chapter has one.
                let sup = self
                    .file
                    .levels
                    .iter()
                    .position(|l| l.build.is_some() && l.chapter == c.id);
                let super_info = sup.map(|i| {
                    let l = &self.file.levels[i];
                    let s = self.stars_of(&l.id);
                    json!({"id": l.id, "title": l.title, "stars": s,
                           "stars_text": self.stars_text(s),
                           "locked": !self.unlocked(i), "passed": s[0],
                           "gold": self.progress.gold.get(&l.id).copied().unwrap_or(false)})
                });
                json!({"id": c.id, "title": c.title, "layer": c.layer, "blurb": c.blurb,
                       "backdrop": c.backdrop, "levels": levels, "scene": scene_info,
                       "super": super_info})
            })
            .collect();
        let max = self
            .file
            .levels
            .iter()
            .filter(|l| !l.scene && l.build.is_none())
            .count()
            * 3;
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
        let build = self.file.levels[i].build.as_ref().map(|_| BuildState {
            next: 1,
            ..BuildState::default()
        });
        self.play = Some(Play {
            level: i,
            choice,
            record: None,
            passed: false,
            evidence_seen: Vec::new(),
            prediction: None,
            variant: None,
            build,
        });
        Ok(self.view())
    }

    /// Opens a chapter's demonstration scene (watch-only, not scored). The
    /// scene is a level flagged `scene`; it bypasses the level lock.
    pub fn scene(&mut self, chapter: &str) -> Result<String, String> {
        let i = self
            .file
            .levels
            .iter()
            .position(|l| l.scene && l.chapter == chapter)
            .ok_or_else(|| self.t("err_no_level"))?;
        let choice = self.file.levels[i].edits.iter().map(|e| e.start).collect();
        self.play = Some(Play {
            level: i,
            choice,
            record: None,
            passed: false,
            evidence_seen: Vec::new(),
            prediction: None,
            variant: None,
            build: None,
        });
        Ok(self.view())
    }

    /// Marks a chapter's scene as watched, so it is not shown again.
    pub fn scene_done(&mut self, chapter: &str) -> String {
        self.progress.scenes_seen.insert(chapter.to_string(), true);
        self.home()
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

    /// The edits as the active practice variant presents them: the base edits,
    /// with any option list the variant replaces (so labels and values match
    /// the variant's addresses). On the scored case this is just the base edits.
    fn effective_edits(&self) -> Vec<level::EditDef> {
        let play = self.play.as_ref().expect("a level is open");
        let base = &self.file.levels[play.level];
        let mut edits = base.edits.clone();
        if let Some(v) = play.variant.and_then(|vi| base.variants.get(vi)) {
            for ov in &v.edits {
                if let Some(e) = edits.iter_mut().find(|e| e.id == ov.edit) {
                    e.options = ov.options.clone();
                }
            }
        }
        edits
    }

    /// The concrete level the record was built from: an assembled build level
    /// if this is one, otherwise the edited level. Playback (`frame`, `node`)
    /// uses it so node indices and positions line up with the record.
    fn current_level(&self) -> Level {
        let play = self.play.as_ref().expect("a level is open");
        if self.file.levels[play.level].build.is_some() {
            self.assembled()
                .unwrap_or_else(|_| self.file.levels[play.level].clone())
        } else {
            self.edited()
        }
    }

    fn edited(&self) -> Level {
        let play = self.play.as_ref().expect("a level is open");
        let base = &self.file.levels[play.level];
        let mut l = base.clone();
        // A fixed practice variant changes the numbers before the student
        // configures: its field overrides go on first, then its seed, then the
        // chosen edits (whose option values the variant may also have replaced).
        if let Some(v) = play.variant.and_then(|vi| base.variants.get(vi)) {
            l.seed = v.seed;
            for o in &v.overrides {
                level::apply(&mut l, &o.target, &o.value).expect("variant validated at load");
            }
        }
        for (e, &c) in self.effective_edits().iter().zip(&play.choice) {
            level::apply(&mut l, &e.target, &e.options[c].value).expect("validated at load");
        }
        l
    }

    /// Records that the student inspected a device or cable (the evidence step).
    pub fn inspect(&mut self, id: &str) -> Result<String, String> {
        if self.play.is_none() {
            return Err(self.t("err_no_level"));
        }
        let play = self.play.as_mut().expect("checked");
        if !play.evidence_seen.iter().any(|e| e == id) {
            play.evidence_seen.push(id.to_string());
        }
        Ok(self.view())
    }

    /// Commits the outcome the student predicts, before the run.
    pub fn predict(&mut self, option: usize) -> Result<String, String> {
        let i = self.play.as_ref().ok_or_else(|| self.t("err_no_level"))?.level;
        let ok = self.file.levels[i]
            .predict
            .as_ref()
            .map(|p| option < p.options.len())
            .unwrap_or(false);
        if !ok {
            return Err(self.t("err_bad_choice"));
        }
        self.play.as_mut().expect("checked").prediction = Some(option);
        Ok(self.view())
    }

    /// Switches to the next fixed practice case after a miss. A practice run is
    /// for learning and never re-awards a star.
    pub fn practice(&mut self) -> Result<String, String> {
        let i = self.play.as_ref().ok_or_else(|| self.t("err_no_level"))?.level;
        let n = self.file.levels[i].variants.len();
        if n == 0 {
            return Err(self.t("err_bad_choice"));
        }
        let choice: Vec<usize> = self.file.levels[i].edits.iter().map(|e| e.start).collect();
        let play = self.play.as_mut().expect("checked");
        play.variant = Some(match play.variant {
            None => 0,
            Some(v) => (v + 1) % n,
        });
        play.choice = choice;
        play.record = None;
        play.passed = false;
        play.prediction = None;
        play.evidence_seen.clear();
        Ok(self.view())
    }

    // ---------- build levels (end-of-zone "super" levels) ----------

    fn build_def(&self) -> Result<&level::BuildDef, String> {
        let play = self.play.as_ref().ok_or_else(|| self.t("err_no_level"))?;
        self.file.levels[play.level]
            .build
            .as_ref()
            .ok_or_else(|| self.t("err_bad_choice"))
    }

    /// Every node id on a build level, fixed endpoints then placed machines,
    /// with its kind, name and position.
    fn build_nodes(&self) -> Vec<(String, String, String, i32, i32)> {
        let play = self.play.as_ref().expect("a level is open");
        let base = &self.file.levels[play.level];
        let mut out: Vec<(String, String, String, i32, i32)> = base
            .nodes
            .iter()
            .map(|n| (n.id.clone(), n.kind.clone(), n.name.clone(), n.x, n.y))
            .collect();
        if let Some(b) = play.build.as_ref() {
            for p in &b.placed {
                out.push((p.id.clone(), p.kind.clone(), p.name.clone(), p.x, p.y));
            }
        }
        out
    }

    fn cost_of_kind(&self, def: &level::BuildDef, kind: &str) -> u32 {
        def.costs
            .iter()
            .find(|c| c.kind == kind)
            .map(|c| c.cost)
            .unwrap_or(0)
    }

    /// The length of a cable between two node ids (integer, like the map).
    fn cable_len(&self, a: &str, b: &str) -> u32 {
        let nodes = self.build_nodes();
        let find = |id: &str| nodes.iter().find(|n| n.0 == id).map(|n| (n.3, n.4));
        match (find(a), find(b)) {
            (Some((ax, ay)), Some((bx, by))) => {
                let (dx, dy) = ((ax - bx) as i64, (ay - by) as i64);
                isqrt(dx * dx + dy * dy) as u32
            }
            _ => 0,
        }
    }

    /// The running bill: machines bought, cable laid, and the sum.
    fn build_cost(&self) -> (u32, u32, u32) {
        let def = match self.build_def() {
            Ok(d) => d,
            Err(_) => return (0, 0, 0),
        };
        let machines: u32 = self
            .build_nodes()
            .iter()
            .map(|n| self.cost_of_kind(def, &n.1))
            .sum();
        let play = self.play.as_ref().expect("level open");
        let wire: u32 = play
            .build
            .as_ref()
            .map(|b| {
                b.cables
                    .iter()
                    .map(|(a, c)| self.cable_len(a, c) * def.wire_cost)
                    .sum()
            })
            .unwrap_or(0);
        (machines, wire, machines + wire)
    }

    fn in_obstacle(def: &level::BuildDef, x: i32, y: i32) -> bool {
        def.obstacles
            .iter()
            .any(|o| x >= o.x && x <= o.x + o.w && y >= o.y && y <= o.y + o.h)
    }

    /// Whether the straight cable between two points runs into any fixture — a
    /// cable may not cross the kitchen, which forces the layout around it. True
    /// if the segment touches an obstacle's rectangle at all.
    fn cable_blocked(def: &level::BuildDef, ax: i32, ay: i32, bx: i32, by: i32) -> bool {
        def.obstacles.iter().any(|o| seg_hits_rect(ax, ay, bx, by, o))
    }

    /// Works out the addressing of a routed build from the student's placement
    /// and wiring: each router interface's address (a `/24` toward a
    /// neighbourhood, a `/30` between two routers), every host's gateway, each
    /// router's directly-connected neighbour routers and the address to send
    /// toward them, and the blocks each router still needs a signpost for. The
    /// student chooses boxes, cables and next-hop neighbours; the numbers are
    /// derived here so the lesson is the forwarding table, not address typing.
    fn build_net(&self) -> BuildNet {
        let mut wiring = BuildNet::default();
        let def = match self.build_def() {
            Ok(d) => d,
            Err(_) => return wiring,
        };
        let play = self.play.as_ref().expect("level open");
        let base = &self.file.levels[play.level];
        let st = match play.build.as_ref() {
            Some(s) => s,
            None => return wiring,
        };
        let nodes = self.build_nodes();
        let pos = |id: &str| nodes.iter().position(|n| n.0 == id);
        let kind = |id: &str| {
            nodes
                .iter()
                .find(|n| n.0 == id)
                .map(|n| n.1.clone())
                .unwrap_or_default()
        };
        let is_router = |id: &str| kind(id) == "router";
        let cidr_of = |id: &str| -> Option<Cidr> {
            base.nodes
                .iter()
                .find(|n| n.id == id && !n.ip.is_empty())
                .and_then(|n| parse_cidr(&n.ip))
        };
        let cables = &st.cables;
        let neighbours = |id: &str| -> Vec<String> {
            cables
                .iter()
                .filter_map(|(a, b)| {
                    if a == id {
                        Some(b.clone())
                    } else if b == id {
                        Some(a.clone())
                    } else {
                        None
                    }
                })
                .collect()
        };
        // L2 segments: non-router nodes joined without crossing a router. Each is
        // one block, named by any fixed host or server on it.
        let mut comp: BTreeMap<String, usize> = BTreeMap::new();
        let mut comp_block: Vec<Option<Cidr>> = Vec::new();
        for n in nodes.iter().filter(|n| n.1 != "router") {
            if comp.contains_key(&n.0) {
                continue;
            }
            let c = comp_block.len();
            let mut block: Option<Cidr> = None;
            let mut stack = vec![n.0.clone()];
            comp.insert(n.0.clone(), c);
            while let Some(u) = stack.pop() {
                if let Some(cd) = cidr_of(&u) {
                    block = Some(cd.network());
                }
                for v in neighbours(&u) {
                    if !is_router(&v) && !comp.contains_key(&v) {
                        comp.insert(v.clone(), c);
                        stack.push(v);
                    }
                }
            }
            comp_block.push(block);
        }
        // Router interfaces toward a neighbourhood, and the gateway every host on
        // that segment uses (the router's address on the block, <network>.1).
        for n in nodes.iter().filter(|n| n.1 == "router") {
            let r = &n.0;
            for (ci, (a, b)) in cables.iter().enumerate() {
                let other = if a == r {
                    b
                } else if b == r {
                    a
                } else {
                    continue;
                };
                if is_router(other) {
                    continue;
                }
                if let Some(&c) = comp.get(other) {
                    if let Some(block) = comp_block[c] {
                        let ip = (block.ip & mask(block.len)) + 1;
                        wiring
                            .iface
                            .entry(r.clone())
                            .or_default()
                            .push((format!("c{}", ci + 1), format!("{}/{}", show_ip(ip), block.len)));
                        for (m, &mc) in comp.iter() {
                            if mc == c && matches!(kind(m).as_str(), "host" | "server" | "printer") {
                                wiring.gw.entry(m.clone()).or_insert_with(|| show_ip(ip));
                            }
                        }
                    }
                }
            }
        }
        // Point-to-point /30 links between two routers, numbered in a fixed order.
        let mut rr: Vec<(usize, String, String)> = cables
            .iter()
            .enumerate()
            .filter(|(_, (a, b))| is_router(a) && is_router(b))
            .map(|(i, (a, b))| (i, a.clone(), b.clone()))
            .collect();
        rr.sort_by_key(|(_, a, b)| {
            let (pa, pb) = (pos(a).unwrap_or(0), pos(b).unwrap_or(0));
            (pa.min(pb), pa.max(pb))
        });
        for (k, (ci, a, b)) in rr.iter().enumerate() {
            let net30 = P2P_BASE + 4 * k as u32;
            let (pa, pb) = (pos(a).unwrap_or(0), pos(b).unwrap_or(0));
            let (lo, hi) = if pa <= pb { (a, b) } else { (b, a) };
            let cable = format!("c{}", ci + 1);
            wiring
                .iface
                .entry(lo.clone())
                .or_default()
                .push((cable.clone(), format!("{}/30", show_ip(net30 + 1))));
            wiring
                .iface
                .entry(hi.clone())
                .or_default()
                .push((cable, format!("{}/30", show_ip(net30 + 2))));
            wiring.rneigh.entry(lo.clone()).or_default().push(hi.clone());
            wiring.rneigh.entry(hi.clone()).or_default().push(lo.clone());
            wiring.via_ip.insert((lo.clone(), hi.clone()), show_ip(net30 + 2));
            wiring.via_ip.insert((hi.clone(), lo.clone()), show_ip(net30 + 1));
        }
        // The blocks each router is not directly on — the signposts to fill.
        let norm = |p: &str| parse_cidr(p).map(|c| c.network().show());
        for n in nodes.iter().filter(|n| n.1 == "router") {
            let r = &n.0;
            let mut here: Vec<String> = Vec::new();
            for (a, b) in cables.iter() {
                let other = if a == r {
                    b
                } else if b == r {
                    a
                } else {
                    continue;
                };
                if is_router(other) {
                    continue;
                }
                if let Some(&c) = comp.get(other) {
                    if let Some(block) = comp_block[c] {
                        here.push(block.show());
                    }
                }
            }
            let remote: Vec<(String, String)> = def
                .blocks
                .iter()
                .filter(|bl| match norm(&bl.prefix) {
                    Some(p) => !here.contains(&p),
                    None => true,
                })
                .map(|bl| (bl.prefix.clone(), bl.name.clone()))
                .collect();
            wiring.remote.insert(r.clone(), remote);
        }
        wiring
    }

    /// Places one machine from the palette at a point on the floor.
    pub fn place(&mut self, kind: &str, x: i32, y: i32) -> Result<String, String> {
        let def = self.build_def()?.clone();
        let item = def
            .palette
            .iter()
            .find(|p| p.kind == kind)
            .ok_or_else(|| self.t("err_bad_choice"))?;
        let play = self.play.as_mut().expect("level open");
        let b = play.build.as_mut().expect("build level");
        let have = b.placed.iter().filter(|p| p.kind == kind).count() as u32;
        if have >= item.max {
            return Err(self.t("err_build_max"));
        }
        if Self::in_obstacle(&def, x, y) {
            return Err(self.t("err_build_obstacle"));
        }
        let n = b.placed.iter().filter(|p| p.kind == kind).count() as u32 + 1;
        b.placed.push(Placed {
            id: format!("m{}", b.next),
            kind: kind.to_string(),
            name: format!("{} {}", item.name, n),
            x,
            y,
        });
        b.next += 1;
        Ok(self.build_state())
    }

    /// Removes a placed machine and anything that depended on it.
    pub fn unplace(&mut self, id: &str) -> Result<String, String> {
        self.build_def()?;
        let play = self.play.as_mut().expect("level open");
        let b = play.build.as_mut().expect("build level");
        if !b.placed.iter().any(|p| p.id == id) {
            return Err(self.t("err_bad_choice"));
        }
        b.placed.retain(|p| p.id != id);
        b.cables.retain(|(a, c)| a != id && c != id);
        b.tables.remove(id);
        for rows in b.tables.values_mut() {
            rows.retain(|(dst, via)| dst != id && via != id);
        }
        b.routes.remove(id);
        for rows in b.routes.values_mut() {
            rows.retain(|(_, via)| via != id);
        }
        Ok(self.build_state())
    }

    /// Lays a cable between two machines.
    pub fn wire(&mut self, a: &str, b: &str) -> Result<String, String> {
        let def = self.build_def()?.clone();
        if a == b {
            return Err(self.t("err_bad_choice"));
        }
        let nodes = self.build_nodes();
        let known = |id: &str| nodes.iter().any(|n| n.0 == id);
        if !known(a) || !known(b) {
            return Err(self.t("err_bad_choice"));
        }
        if def.max_wire > 0 && self.cable_len(a, b) > def.max_wire {
            return Err(self.t("err_build_too_long"));
        }
        let ends = |id: &str| nodes.iter().find(|n| n.0 == id).map(|n| (n.3, n.4));
        if let (Some((ax, ay)), Some((bx, by))) = (ends(a), ends(b)) {
            if Self::cable_blocked(&def, ax, ay, bx, by) {
                return Err(self.t("err_build_crosses"));
            }
        }
        let play = self.play.as_mut().expect("level open");
        let st = play.build.as_mut().expect("build level");
        let exists = st
            .cables
            .iter()
            .any(|(x, y)| (x == a && y == b) || (x == b && y == a));
        if exists {
            return Err(self.t("err_build_wired"));
        }
        st.cables.push((a.to_string(), b.to_string()));
        Ok(self.build_state())
    }

    /// Removes a cable between two machines.
    pub fn unwire(&mut self, a: &str, b: &str) -> Result<String, String> {
        self.build_def()?;
        let play = self.play.as_mut().expect("level open");
        let st = play.build.as_mut().expect("build level");
        let before = st.cables.len();
        st.cables
            .retain(|(x, y)| !((x == a && y == b) || (x == b && y == a)));
        if st.cables.len() == before {
            return Err(self.t("err_bad_choice"));
        }
        Ok(self.build_state())
    }

    /// Gives a host an address (empty clears it).
    pub fn set_addr(&mut self, node: &str, ip: &str) -> Result<String, String> {
        let def = self.build_def()?.clone();
        if !def.needs_addr.iter().any(|n| n == node) {
            return Err(self.t("err_bad_choice"));
        }
        if !ip.is_empty() && !def.addr_choices.iter().any(|c| c == ip) {
            return Err(self.t("err_bad_choice"));
        }
        let play = self.play.as_mut().expect("level open");
        let st = play.build.as_mut().expect("build level");
        if ip.is_empty() {
            st.addrs.remove(node);
        } else {
            st.addrs.insert(node.to_string(), ip.to_string());
        }
        Ok(self.build_state())
    }

    /// Sets one row of a switch's table: to reach `dst`, send toward `via`
    /// (empty `via` clears the row).
    pub fn set_port(&mut self, switch: &str, dst: &str, via: &str) -> Result<String, String> {
        self.build_def()?;
        let play = self.play.as_mut().expect("level open");
        let st = play.build.as_mut().expect("build level");
        if !st.placed.iter().any(|p| p.id == switch && p.kind == "switch") {
            return Err(self.t("err_bad_choice"));
        }
        let rows = st.tables.entry(switch.to_string()).or_default();
        rows.retain(|(d, _)| d != dst);
        if !via.is_empty() {
            rows.push((dst.to_string(), via.to_string()));
        }
        Ok(self.build_state())
    }

    /// Sets one row of a router's forwarding table: to reach the block `prefix`,
    /// send toward the neighbour router `via` (empty `via` clears the row). The
    /// routed-build twin of `set_port` — the student fills every router's table
    /// the way they fill a switch's.
    pub fn set_route(&mut self, router: &str, prefix: &str, via: &str) -> Result<String, String> {
        self.build_def()?;
        let play = self.play.as_mut().expect("level open");
        let st = play.build.as_mut().expect("build level");
        if !st.placed.iter().any(|p| p.id == router && p.kind == "router") {
            return Err(self.t("err_bad_choice"));
        }
        let rows = st.routes.entry(router.to_string()).or_default();
        rows.retain(|(p, _)| p != prefix);
        if !via.is_empty() {
            rows.push((prefix.to_string(), via.to_string()));
        }
        Ok(self.build_state())
    }

    /// Sets the hop count (TTL) the terminals send with; must be one of the
    /// level's offered choices.
    pub fn set_ttl(&mut self, value: u32) -> Result<String, String> {
        let def = self.build_def()?.clone();
        if !def.ttl_choices.contains(&value) {
            return Err(self.t("err_bad_choice"));
        }
        let play = self.play.as_mut().expect("level open");
        play.build.as_mut().expect("build level").ttl = value;
        Ok(self.build_state())
    }

    /// Assembles the student's work into a concrete level the simulation runs.
    fn assembled(&self) -> Result<Level, String> {
        let play = self.play.as_ref().expect("level open");
        let def = self.build_def()?;
        let mut l = self.file.levels[play.level].clone();
        l.edits = Vec::new();
        let st = play.build.as_ref().expect("build level");
        // The addressing the wiring implies: router interfaces, gateways, next hops.
        let wiring = self.build_net();
        // The hop count the student chose (or the lowest offered, so an untouched
        // build dies on the longest route and the TTL decision bites).
        let ttl = if st.ttl > 0 {
            st.ttl
        } else {
            def.ttl_choices.first().copied().unwrap_or(0)
        };
        // Addresses, gateways and hop counts onto the fixed hosts/servers.
        for n in l.nodes.iter_mut() {
            if let Some(ip) = st.addrs.get(&n.id) {
                n.ip = ip.clone();
            }
            if let Some(gw) = wiring.gw.get(&n.id) {
                n.gateway = gw.clone();
            }
            if ttl > 0 && matches!(n.kind.as_str(), "host" | "server" | "printer") {
                n.ttl = ttl;
            }
        }
        // Placed machines. A routed build's switch is an ordinary learning switch
        // (the student's judgement is choosing it over a hub); the restaurant's is
        // a static table the student fills. A router is given the interfaces the
        // wiring implies and the forwarding table the student filled.
        for p in &st.placed {
            let mut node = level::NodeDef {
                id: p.id.clone(),
                kind: p.kind.clone(),
                name: p.name.clone(),
                x: p.x,
                y: p.y,
                ip: st.addrs.get(&p.id).cloned().unwrap_or_default(),
                gateway: String::new(),
                ifaces: Vec::new(),
                routes: Vec::new(),
                block: String::new(),
                ttl: 0,
                static_fwd: false,
                mac_hints: Vec::new(),
            };
            match p.kind.as_str() {
                "switch" if def.static_switch => {
                    node.static_fwd = true;
                    if let Some(rows) = st.tables.get(&p.id) {
                        node.mac_hints = rows
                            .iter()
                            .map(|(dst, via)| level::MacHint {
                                dst: dst.clone(),
                                via: via.clone(),
                            })
                            .collect();
                    }
                }
                "router" => {
                    if let Some(ifs) = wiring.iface.get(&p.id) {
                        node.ifaces = ifs
                            .iter()
                            .map(|(link, ip)| level::IfaceDef {
                                link: link.clone(),
                                ip: ip.clone(),
                            })
                            .collect();
                    }
                    if let Some(rows) = st.routes.get(&p.id) {
                        node.routes =
                            rows.iter()
                                .filter_map(|(prefix, via)| {
                                    wiring.via_ip.get(&(p.id.clone(), via.clone())).map(|vip| {
                                        level::RouteDef {
                                            prefix: prefix.clone(),
                                            via: vip.clone(),
                                        }
                                    })
                                })
                                .collect();
                    }
                }
                _ => {}
            }
            l.nodes.push(node);
        }
        // Cables.
        l.links = st
            .cables
            .iter()
            .enumerate()
            .map(|(i, (a, b))| level::LinkDef {
                id: format!("c{}", i + 1),
                a: a.clone(),
                b: b.clone(),
                down: false,
                lose_every: 0,
                delay: 1,
            })
            .collect();
        Ok(l)
    }

    /// The whole build canvas as JSON: the bill so far, the palette with how
    /// many of each the student has placed, the fixed endpoints and the placed
    /// machines, the cables with their length and price, and the addresses and
    /// switch tables filled in.
    pub fn build_state(&self) -> String {
        let def = match self.build_def() {
            Ok(d) => d.clone(),
            Err(e) => return json!({"error": e}).to_string(),
        };
        let play = self.play.as_ref().expect("level open");
        let base = &self.file.levels[play.level];
        let st = play.build.as_ref().expect("build level");
        let (machines, wire, total) = self.build_cost();
        let placed_count = |kind: &str| st.placed.iter().filter(|p| p.kind == kind).count() as u32;
        let palette: Vec<Value> = def
            .palette
            .iter()
            .map(|p| {
                json!({"kind": p.kind, "name": p.name, "max": p.max, "ports": p.ports,
                       "placed": placed_count(&p.kind), "cost": self.cost_of_kind(&def, &p.kind)})
            })
            .collect();
        // A machine ready to draw: its centre is its map point; the icon and
        // the labels hang off it by the same constants the ordinary map uses.
        // `idx` is its position once assembled (fixed first, then placed), which
        // is what fixes its MAC — so the student can see every machine's MAC,
        // even before a run.
        let machine = |idx: usize, id: &str, kind: &str, name: &str, x: i32, y: i32, fixed: bool| {
            json!({"id": id, "kind": kind, "name": name, "fixed": fixed,
                   "mac": show_mac(idx as u32 + 0x10),
                   "cost": self.cost_of_kind(&def, kind),
                   "addr": st.addrs.get(id).cloned().or_else(|| {
                       base.nodes
                           .iter()
                           .find(|n| n.id == id && !n.ip.is_empty())
                           .map(|n| n.ip.clone())
                   }),
                   "needs_addr": def.needs_addr.iter().any(|n| n == id),
                   "x": x - ICON / 2, "y": y - ICON / 2, "size": ICON,
                   "cx": x, "cy": y,
                   "ly": y + ICON / 2 + LABEL_GAP,
                   "ly2": y + ICON / 2 + LABEL_GAP + LINE_GAP})
        };
        let fixed: Vec<Value> = base
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| machine(i, &n.id, &n.kind, &n.name, n.x, n.y, true))
            .collect();
        let placed: Vec<Value> = st
            .placed
            .iter()
            .enumerate()
            .map(|(j, p)| machine(base.nodes.len() + j, &p.id, &p.kind, &p.name, p.x, p.y, false))
            .collect();
        let nodes = self.build_nodes();
        let center = |id: &str| nodes.iter().find(|n| n.0 == id).map(|n| (n.3, n.4));
        let cables: Vec<Value> = st
            .cables
            .iter()
            .filter_map(|(a, b)| {
                let (ax, ay) = center(a)?;
                let (bx, by) = center(b)?;
                let len = self.cable_len(a, b);
                Some(json!({"a": a, "b": b, "ax": ax, "ay": ay, "bx": bx, "by": by,
                            "mx": (ax + bx) / 2, "my": (ay + by) / 2,
                            "len": len, "cost": len * def.wire_cost}))
            })
            .collect();
        let tables: BTreeMap<&String, Vec<Value>> = st
            .tables
            .iter()
            .map(|(sw, rows)| {
                (
                    sw,
                    rows.iter()
                        .map(|(dst, via)| json!({"dst": dst, "via": via}))
                        .collect(),
                )
            })
            .collect();
        // The routed-build extras: each placed router's signposts to fill (the
        // remote blocks, the neighbour routers it can send toward, and the
        // student's current pick for each), the hop-count choice, and the blocks.
        let wiring = self.build_net();
        let name_of = |id: &str| {
            nodes
                .iter()
                .find(|n| n.0 == id)
                .map(|n| n.2.clone())
                .unwrap_or_default()
        };
        let routers: Vec<Value> = st
            .placed
            .iter()
            .filter(|p| p.kind == "router")
            .map(|p| {
                let rows = st.routes.get(&p.id);
                let via_of = |prefix: &str| {
                    rows.and_then(|rs| rs.iter().find(|(pp, _)| pp == prefix))
                        .map(|(_, v)| v.clone())
                        .unwrap_or_default()
                };
                let blocks: Vec<Value> = wiring
                    .remote
                    .get(&p.id)
                    .map(|v| {
                        v.iter()
                            .map(|(prefix, name)| {
                                json!({"prefix": prefix, "name": name, "via": via_of(prefix)})
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let neighbours: Vec<Value> = wiring
                    .rneigh
                    .get(&p.id)
                    .map(|v| {
                        v.iter()
                            .map(|nid| json!({"id": nid, "name": name_of(nid)}))
                            .collect()
                    })
                    .unwrap_or_default();
                json!({"id": p.id, "name": p.name, "blocks": blocks, "neighbours": neighbours})
            })
            .collect();
        json!({
            "build": true,
            "viewbox": format!("0 0 {MAP_W} {MAP_H}"),
            "subnet": def.subnet,
            "wire_cost": def.wire_cost,
            "max_wire": def.max_wire,
            "cost": {"machines": machines, "wire": wire, "total": total},
            "star_cost": def.star_cost,
            "gold_cost": def.gold_cost,
            "palette": palette,
            "obstacles": def.obstacles.iter().map(|o| json!({"x": o.x, "y": o.y, "w": o.w, "h": o.h,
                "cx": o.x + o.w / 2, "cy": o.y + o.h / 2, "label": o.label})).collect::<Vec<_>>(),
            "fixed": fixed,
            "placed": placed,
            "cables": cables,
            "needs_addr": def.needs_addr,
            "addr_choices": def.addr_choices,
            "addrs": st.addrs,
            "tables": tables,
            "concept": base.concept,
            "blocks": def.blocks.iter().map(|b| json!({"prefix": b.prefix, "name": b.name})).collect::<Vec<_>>(),
            "ttl": {"choices": def.ttl_choices, "current": st.ttl},
            "routers": routers,
            "gateways": wiring.gw,
            "static_switch": def.static_switch,
        })
        .to_string()
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
        let eff_edits = self.effective_edits();
        let edits: Vec<Value> = eff_edits
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
        // Phase 2: the prediction choices (graded against the run, so no correct
        // option is marked here) and the evidence the student should inspect.
        let predict = base.predict.as_ref().map(|pd| {
            json!({
                "prompt": pd.prompt,
                "options": pd.options.iter().map(|o| json!({"label": o.label})).collect::<Vec<_>>(),
                "chosen": play.prediction,
            })
        });
        json!({
            "id": base.id, "title": base.title,
            "number": fill(&self.t("level_n"), &[(play.level + 1).to_string()]),
            "chapter": chapter.title, "layer": chapter.layer, "backdrop": chapter.backdrop,
            "story": base.story, "concept": base.concept, "task": base.task,
            "goals": goals, "bonus": base.bonus.text,
            "nodes": nodes, "links": links, "edits": edits,
            "viewbox": format!("0 0 {MAP_W} {MAP_H}"),
            "stars": s, "stars_text": self.stars_text(s), "next": next, "legend": legend,
            "predict": predict,
            "evidence": base.evidence,
            "evidence_seen": play.evidence_seen,
            "variant": play.variant,
            "scene": base.scene,
            "build": base.build.is_some(),
            "note": base.note,
            "textbook": chapter.textbook,
            "textbook_src": chapter.textbook_src,
        })
        .to_string()
    }

    /// Runs the open level with the chosen edits, scores it, and keeps the
    /// record for playback.
    pub fn run(&mut self) -> Result<String, String> {
        let i = self.play.as_ref().ok_or_else(|| self.t("err_no_level"))?.level;
        // An evidence-first level asks for a committed prediction before a
        // scored run (inspecting the glowing evidence is encouraged by the
        // prompt, but never a hard blocker). A level without a `predict` block
        // runs on demand as before.
        if self.file.levels[i].predict.is_some()
            && self.play.as_ref().expect("level open").prediction.is_none()
        {
            return Err(self.t("err_predict_first"));
        }
        // A build level assembles the student's placed machines, cables,
        // addresses and switch tables into the network that runs; an ordinary
        // level applies its edits.
        let is_build = self.file.levels[i].build.is_some();
        let build_cost = if is_build { self.build_cost() } else { (0, 0, 0) };
        let l = if is_build {
            self.assembled()?
        } else {
            self.edited()
        };
        let rec = Sim::build(&l)?.run(l.ticks);
        let goals: Vec<(String, bool)> = l
            .goal
            .iter()
            .map(|g| (g.text.clone(), met(&l, g, &rec)))
            .collect();
        let passed = goals.iter().all(|g| g.1);
        let bonus = passed && met(&l, &l.bonus, &rec);
        let has_predict = self.file.levels[i].predict.is_some();
        let predict_ok = self.file.levels[i]
            .predict
            .as_ref()
            .and_then(|pd| {
                self.play
                    .as_ref()
                    .and_then(|p| p.prediction)
                    .and_then(|pi| pd.options.get(pi))
            })
            .map(|o| graded(&l, &o.claim, &rec))
            .unwrap_or(false);
        // A build level earns its bonus stars by coming in under budget: the
        // middle star for the looser threshold, the final star for the tighter.
        let under = |k: usize| {
            self.file.levels[i]
                .build
                .as_ref()
                .and_then(|b| b.star_cost.get(k))
                .map(|&t| passed && build_cost.2 <= t)
                .unwrap_or(false)
        };
        let (cheap_enough, cheapest) = (under(1), under(0));
        // The hidden gold star: a build in the cheapest few per cent of all tried.
        let gold = is_build
            && passed
            && self.file.levels[i]
                .build
                .as_ref()
                .map(|b| b.gold_cost > 0 && build_cost.2 <= b.gold_cost)
                .unwrap_or(false);
        let on_variant = self.play.as_ref().map(|p| p.variant.is_some()).unwrap_or(false);
        let is_scene = self.file.levels[i].scene;
        if on_variant || is_scene {
            // A practice case or a demonstration scene teaches; it awards nothing.
        } else {
            let entry = self.progress.stars.entry(l.id.clone()).or_insert([false; 3]);
            entry[0] |= passed;
            if is_build {
                entry[1] |= cheap_enough;
                entry[2] |= cheapest;
                if gold {
                    self.progress.gold.insert(l.id.clone(), true);
                }
            } else if has_predict {
                // The prediction gates the final star — earned only when the
                // level is passed and the committed prediction came true, so a
                // correct guess about an unfixed network earns nothing. The
                // no-loss bonus takes the middle star. (No prediction: quiz.)
                entry[1] |= bonus;
                entry[2] |= passed && predict_ok;
            } else {
                entry[2] |= bonus;
            }
        }
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
        let quiz = if passed && base.predict.is_none() && base.build.is_none() {
            json!({"question": base.quiz.question, "options": base.quiz.options})
        } else {
            Value::Null
        };
        let predict = if has_predict {
            json!({"shown": true, "right": predict_ok})
        } else {
            Value::Null
        };
        let can_practice = !base.variants.is_empty();
        // The next scored level (scenes are skipped), for the result's Next button.
        let next = self.file.levels[i + 1..]
            .iter()
            .find(|l| !l.scene)
            .map(|l| l.id.clone());
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
            "predict": predict,
            "can_practice": can_practice,
            "on_variant": on_variant,
            "next": next,
            "concept": base.concept,
            "stars": stars,
            "stars_text": self.stars_text(stars),
            "ticks": rec.ticks,
            "cost": if is_build { json!({"machines": build_cost.0, "wire": build_cost.1, "total": build_cost.2}) } else { Value::Null },
            "gold": gold,
            "gold_cost": self.file.levels[i].build.as_ref().map(|b| b.gold_cost),
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
        let l = self.current_level();
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
                       "fx": e.fx, "symbol": self.t(&format!("fx_{}", e.fx)),
                       "node": l.nodes[e.node].name.clone(), "kind": log_kind(e.key, e.fx)})
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
        let l = self.current_level();
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
        let l = self.current_level();
        let n = l.nodes.get(i).ok_or_else(|| self.t("err_bad_choice"))?;
        let owned;
        let info = match &play.record {
            Some(rec) => {
                let tick = (ms as i64 / ms_per_tick.max(1) as i64) as usize;
                &rec.tables[tick.min(rec.tables.len() - 1)][i]
            }
            None => {
                owned = Sim::build(&l)?.run(0);
                &owned.tables[0][i]
            }
        };
        let facts: Vec<Value> = info
            .facts
            .iter()
            .map(|(k, v)| {
                let val = match v {
                    FactVal::Text(s) => s.clone(),
                    FactVal::Copy(k) => self.t(k),
                };
                json!([self.t(k), val])
            })
            .collect();
        let tables: Vec<Value> = info
            .tables
            .iter()
            .map(|tbl| {
                let cols: Vec<String> = tbl.cols.iter().map(|c| self.t(c)).collect();
                json!({
                    "title": self.t(tbl.title),
                    "cols": cols,
                    "rows": tbl.rows,
                    "empty": tbl.empty.map(|e| self.t(e)),
                })
            })
            .collect();
        Ok(
            json!({"name": n.name, "kind": n.kind, "kind_name": self.t(&format!("kind_{}", n.kind)),
                  "blurb": self.t(&format!("about_{}", n.kind)), "facts": facts, "tables": tables})
            .to_string(),
        )
    }
}
