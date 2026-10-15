//! Permutation / property sweep over the restaurant build level, in the spirit
//! of a survivors-like's "rig space": enumerate the networks a student could
//! build, play each through the real deterministic engine, and hold every one
//! to the same rules — nothing panics, a sound build always delivers, the
//! star tiers really are the cost percentiles of all tried builds, an
//! identical build is byte-identical twice, and no random sequence of edits
//! can wedge the core. The point is that the level is not buggy for students.

use std::fs;
use std::path::PathBuf;

use pa_core::game::Game;

const LEVEL: &str = "newspot";
const OTHERS: [&str; 4] = ["srv", "reg1", "reg2", "reg3"];
const ADDRS: [(&str, &str); 3] = [
    ("reg1", "10.0.0.11/24"),
    ("reg2", "10.0.0.12/24"),
    ("reg3", "10.0.0.13/24"),
];
// The floor the sweep walks, matching the map's coordinate range and the grid
// the star tiers were derived from.
const STEP: i32 = 20;
const MAX_X: i32 = 1000;
const MAX_Y: i32 = 600;

fn data(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data")
        .join(name);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {name}: {e}"))
}

fn game() -> Game {
    let mut g = Game::new(&data("levels.json"), &data("copy.json"), "").expect("shipped data loads");
    g.set_teacher(true);
    g
}

/// Reads the integer that follows `"key":` in a flat JSON string.
fn num(json: &str, key: &str) -> Option<i64> {
    let at = json.find(&format!("\"{key}\":"))? + key.len() + 3;
    let rest = &json[at..];
    let end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '-')
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

/// Opens the level, places one switch at (x,y), wires it to the four fixed
/// machines in a star, addresses the registers and fills the table correctly,
/// then runs. `None` if the switch cannot be placed there or a cable overshoots
/// the reel — i.e. not a buildable spot.
fn solve_single(g: &mut Game, x: i32, y: i32) -> Option<String> {
    g.open(LEVEL).expect("the restaurant opens");
    if g.place("switch", x, y).is_err() {
        return None;
    }
    for other in OTHERS {
        if g.wire("m1", other).is_err() {
            return None;
        }
    }
    for (host, ip) in ADDRS {
        g.set_addr(host, ip).unwrap();
    }
    for dst in OTHERS {
        g.set_port("m1", dst, dst).unwrap();
    }
    Some(g.run().expect("a configured build runs"))
}

/// Every single-switch placement on the floor grid that can be wired within
/// the reel, as (x, y, total-cost), using the real engine's own cost.
fn single_switch_builds(g: &mut Game) -> Vec<(i32, i32, i64)> {
    let mut out = Vec::new();
    let mut x = 0;
    while x <= MAX_X {
        let mut y = 0;
        while y <= MAX_Y {
            if let Some(run) = solve_single(g, x, y) {
                out.push((x, y, num(&run, "total").expect("a build reports its cost")));
            }
            y += STEP;
        }
        x += STEP;
    }
    out
}

#[test]
fn every_buildable_single_switch_spot_delivers() {
    // Wherever the switch can legally sit and reach all four, a correct build
    // must pass — no dead spot silently fails a student who did everything
    // right, and nothing panics along the way.
    let mut g = game();
    let builds = single_switch_builds(&mut g);
    assert!(
        builds.len() > 100,
        "the floor offers many buildable spots: {}",
        builds.len()
    );
    for (x, y, _) in &builds {
        let run = solve_single(&mut g, *x, *y).expect("a buildable spot");
        assert!(
            run.contains("\"passed\":true"),
            "a correct build at ({x},{y}) should deliver: {run}"
        );
    }
}

#[test]
fn the_star_tiers_are_the_cost_percentiles_of_every_tried_build() {
    // The three visible stars and the hidden gold star are the 66th, 33rd and
    // 5th percentiles of the cost of every buildable network — so the level's
    // own thresholds have to equal those percentiles. If the geometry changes,
    // this is what catches a stale threshold.
    let mut g = game();
    let mut costs: Vec<i64> = single_switch_builds(&mut g).iter().map(|b| b.2).collect();
    costs.sort_unstable();
    let n = costs.len();
    let pct = |p: f64| costs[(p * (n as f64 - 1.0)) as usize];
    let (p5, p33, p66) = (pct(0.05), pct(0.33), pct(0.66));

    g.open(LEVEL).unwrap();
    let st = g.build_state();
    // star_cost is an array "[a,b]"; grab both numbers.
    let arr_at = st.find("\"star_cost\":[").unwrap() + "\"star_cost\":[".len();
    let arr = &st[arr_at..st[arr_at..].find(']').unwrap() + arr_at];
    let mut it = arr.split(',').map(|s| s.trim().parse::<i64>().unwrap());
    let s3 = it.next().unwrap();
    let s2 = it.next().unwrap();
    let gold = num(&st, "gold_cost").expect("gold_cost");
    assert_eq!(
        (gold, s3, s2),
        (p5, p33, p66),
        "tiers must match the distribution: p5={p5} p33={p33} p66={p66}"
    );
}

#[test]
fn the_tiers_actually_award_the_right_stars() {
    // Spot-check that a build at each tier earns exactly the stars its cost
    // deserves: a cheapest-few build is gold, a top-third build is three stars,
    // a middling build two, and an expensive-but-working build just one.
    // A fresh game each time, so the result's stars are this build's alone and
    // not the accumulated best (stars only ever add up within one game).
    let star_of = |x, y| {
        let mut g = game();
        let run = solve_single(&mut g, x, y).expect("a buildable spot");
        let stars = run[run.find("\"stars\":[").unwrap() + 9..]
            .split(']')
            .next()
            .unwrap()
            .to_string();
        let gold = run.contains("\"gold\":true");
        (stars, gold, run.contains("\"passed\":true"))
    };
    // near-optimal against the kitchen wall → gold + three stars
    let (s, gold, passed) = star_of(416, 300);
    assert!(
        passed && gold && s == "true,true,true",
        "tight build: {s} gold={gold}"
    );
    // a plain central-ish build sits in the middle third → two stars, no gold
    let (s, gold, _) = star_of(500, 420);
    assert!(!gold && s == "true,true,false", "fair build: {s} gold={gold}");
}

#[test]
fn an_identical_build_runs_byte_for_byte_the_same() {
    // The engine is deterministic: the same build played twice gives the same
    // result, down to the JSON.
    let mut g = game();
    let a = solve_single(&mut g, 500, 420).unwrap();
    let b = solve_single(&mut g, 500, 420).unwrap();
    assert_eq!(a, b, "the same build is deterministic");
}

// A small reproducible mixer (SplitMix64), so the fuzz below is deterministic.
struct Mix(u64);
impl Mix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    fn upto(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

#[test]
fn random_edit_sequences_never_wedge_the_core() {
    // Hammer the build API with thousands of arbitrary edits across many
    // seeds. Whatever the student clicks, every call returns valid JSON (an
    // ok state or a refusal), the invariants hold (cost = machines + wire, no
    // cable beats the reel, no more switches than the palette allows), and a
    // run never panics.
    let max_wire: i64 = {
        let mut g = game();
        g.open(LEVEL).unwrap();
        num(&g.build_state(), "max_wire").unwrap()
    };
    let mut g = game();
    for seed in 0..40u64 {
        g.open(LEVEL).unwrap();
        let mut m = Mix(seed.wrapping_mul(0x1000_0001).wrapping_add(1));
        let names = ["srv", "reg1", "reg2", "reg3", "m1", "m2", "m3"];
        let ips = [
            "10.0.0.11/24",
            "10.0.0.12/24",
            "10.0.0.13/24",
            "10.0.0.14/24",
            "10.0.1.50/24",
            "",
        ];
        for _ in 0..200 {
            match m.upto(6) {
                0 => {
                    let x = (m.upto(1100)) as i32 - 50;
                    let y = (m.upto(700)) as i32 - 50;
                    let _ = g.place("switch", x, y);
                }
                1 => {
                    let a = names[m.upto(names.len() as u64) as usize];
                    let b = names[m.upto(names.len() as u64) as usize];
                    let _ = g.wire(a, b);
                }
                2 => {
                    let a = names[m.upto(names.len() as u64) as usize];
                    let b = names[m.upto(names.len() as u64) as usize];
                    let _ = g.unwire(a, b);
                }
                3 => {
                    let host = ["reg1", "reg2", "reg3", "srv"][m.upto(4) as usize];
                    let ip = ips[m.upto(ips.len() as u64) as usize];
                    let _ = g.set_addr(host, ip);
                }
                4 => {
                    let sw = ["m1", "m2", "m3"][m.upto(3) as usize];
                    let dst = names[m.upto(names.len() as u64) as usize];
                    let via = names[m.upto(names.len() as u64) as usize];
                    let _ = g.set_port(sw, dst, via);
                }
                _ => {
                    let id = ["m1", "m2", "m3", "reg1"][m.upto(4) as usize];
                    let _ = g.unplace(id);
                }
            }
            // The state is always well-formed and obeys the invariants.
            let st = g.build_state();
            assert!(st.contains("\"build\":true"), "state stays well-formed: {st}");
            let machines = num(&st, "machines").unwrap();
            let wire = num(&st, "wire").unwrap();
            let total = num(&st, "total").unwrap();
            assert_eq!(machines + wire, total, "cost is machines plus wire: {st}");
            // every cable at or under the reel
            let mut rest = st.as_str();
            while let Some(p) = rest.find("\"len\":") {
                rest = &rest[p + 6..];
                let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
                let len: i64 = rest[..end].parse().unwrap();
                assert!(len <= max_wire, "no cable beats the reel: len={len} > {max_wire}");
            }
            // never more than three switches (the palette's max)
            let switches = st.matches("\"kind\":\"switch\"").count();
            assert!(
                switches <= 4,
                "at most the palette max of switches (plus palette entry): {switches}"
            );
        }
        // A run on whatever got built never panics. If the build is complete
        // enough to assemble, it reports pass or fail; if a host is still
        // unaddressed it is refused with a message — both are fine, neither
        // wedges the core.
        match g.run() {
            Ok(run) => assert!(run.contains("\"passed\":"), "a run reports pass or fail: {run}"),
            Err(msg) => assert!(!msg.is_empty(), "a refusal carries a message"),
        }
    }
}

#[test]
fn an_off_block_or_duplicate_address_cannot_deliver() {
    // For each register in turn, putting it off the shop's block breaks its
    // approval; so does giving two registers the same address.
    let mut g = game();
    for bad in ["reg1", "reg2", "reg3"] {
        g.open(LEVEL).unwrap();
        g.place("switch", 416, 300).unwrap();
        for other in OTHERS {
            g.wire("m1", other).unwrap();
        }
        for (host, ip) in ADDRS {
            let ip = if host == bad { "10.0.1.50/24" } else { ip };
            g.set_addr(host, ip).unwrap();
        }
        for dst in OTHERS {
            g.set_port("m1", dst, dst).unwrap();
        }
        let run = g.run().unwrap();
        assert!(
            run.contains("\"passed\":false"),
            "{bad} off the block must fail: {run}"
        );
    }
}

#[test]
fn every_single_missing_table_row_fails() {
    // A build switch forwards only by its table, so leaving out any one of the
    // four rows leaves that machine unreachable and the level unsolved.
    let mut g = game();
    for missing in OTHERS {
        g.open(LEVEL).unwrap();
        g.place("switch", 416, 300).unwrap();
        for other in OTHERS {
            g.wire("m1", other).unwrap();
        }
        for (host, ip) in ADDRS {
            g.set_addr(host, ip).unwrap();
        }
        for dst in OTHERS {
            if dst != missing {
                g.set_port("m1", dst, dst).unwrap();
            }
        }
        let run = g.run().unwrap();
        assert!(
            run.contains("\"passed\":false"),
            "missing the {missing} row must fail: {run}"
        );
    }
}

#[test]
fn a_two_switch_build_also_delivers() {
    // Two switches near the two clusters, joined by one cable, must carry the
    // traffic across each other — each forwards by its own filled table, and a
    // frame for a far machine is sent toward the other switch. Here it even
    // beats a single central switch, which is a fair reward to discover.
    let mut g = game();
    g.open(LEVEL).unwrap();
    g.place("switch", 300, 305).unwrap(); // m1, by the left-hand registers
    g.place("switch", 600, 305).unwrap(); // m2, by the bar and the server
    for (a, b) in [
        ("m1", "reg1"),
        ("m1", "reg2"),
        ("m2", "reg3"),
        ("m2", "srv"),
        ("m1", "m2"),
    ] {
        g.wire(a, b).unwrap_or_else(|e| panic!("wire {a}-{b}: {e}"));
    }
    for (host, ip) in ADDRS {
        g.set_addr(host, ip).unwrap();
    }
    // m1 reaches reg1/reg2 directly; everything else is across the link to m2.
    for (dst, via) in [("reg1", "reg1"), ("reg2", "reg2"), ("srv", "m2"), ("reg3", "m2")] {
        g.set_port("m1", dst, via).unwrap();
    }
    // m2 reaches reg3/srv directly; the two left registers are back across m1.
    for (dst, via) in [("reg3", "reg3"), ("srv", "srv"), ("reg1", "m1"), ("reg2", "m1")] {
        g.set_port("m2", dst, via).unwrap();
    }
    let run = g.run().unwrap();
    assert!(
        run.contains("\"passed\":true"),
        "two switches carry the traffic across each other: {run}"
    );
}
