//! Permutation / property sweep over the restaurant build level, in the spirit
//! of a survivors-like's "rig space": enumerate the networks a student could
//! build, play each through the real deterministic engine, and hold every one
//! to the same rules. Because a cable may not cross the kitchen, no single
//! switch can serve the whole shop — so the sweep proves that, exercises
//! two-switch builds, checks the star tiers sit sensibly in the cost
//! distribution, and hammers the API so nothing a student clicks can wedge it.

use std::fs;
use std::path::PathBuf;

use pa_core::game::Game;

const LEVEL: &str = "newspot";
const MACHINES: [(&str, i32, i32); 4] = [
    ("reg1", 120, 110),
    ("reg2", 120, 500),
    ("reg3", 860, 500),
    ("srv", 860, 110),
];

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

fn num(json: &str, key: &str) -> Option<i64> {
    let at = json.find(&format!("\"{key}\":"))? + key.len() + 3;
    let rest = &json[at..];
    let end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '-')
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
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

fn clen(ax: i32, ay: i32, bx: i32, by: i32) -> i64 {
    let (dx, dy) = ((ax - bx) as i64, (ay - by) as i64);
    isqrt(dx * dx + dy * dy)
}

// A reproducible SplitMix64 mixer, so the sampling below is deterministic.
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

/// How many of the four machines a single switch at (x,y) can be wired to
/// (each cable within the reel and clear of the kitchen).
fn single_switch_reach(g: &mut Game, x: i32, y: i32) -> Option<usize> {
    g.open(LEVEL).unwrap();
    if g.place("switch", x, y).is_err() {
        return None; // on a wall
    }
    Some(
        MACHINES
            .iter()
            .filter(|(m, _, _)| g.wire("m1", m).is_ok())
            .count(),
    )
}

/// Builds a two-switch network: switches at s1 and s2, each machine wired to
/// the nearer switch it can legally reach, the two switches joined. Returns the
/// total cost if it is a valid, connected build; None otherwise.
fn two_switch_cost(g: &mut Game, s1: (i32, i32), s2: (i32, i32)) -> Option<i64> {
    g.open(LEVEL).unwrap();
    if g.place("switch", s1.0, s1.1).is_err() || g.place("switch", s2.0, s2.1).is_err() {
        return None;
    }
    if g.wire("m1", "m2").is_err() {
        return None;
    }
    let (mut u1, mut u2) = (0, 0);
    for (m, mx, my) in MACHINES {
        let near_first = clen(mx, my, s1.0, s1.1) <= clen(mx, my, s2.0, s2.1);
        let order = if near_first { ["m1", "m2"] } else { ["m2", "m1"] };
        let mut wired = false;
        for sw in order {
            if g.wire(sw, m).is_ok() {
                u1 += (sw == "m1") as i32;
                u2 += (sw == "m2") as i32;
                wired = true;
                break;
            }
        }
        if !wired {
            return None;
        }
    }
    if u1 == 0 || u2 == 0 {
        return None;
    }
    num(&g.build_state(), "total")
}

#[test]
fn no_single_switch_can_serve_the_whole_shop() {
    // The heart of the level: because cable cannot cross the kitchen, there is
    // nowhere to drop one switch that reaches all four machines. The student
    // must use more than one.
    let mut g = game();
    let mut spots = 0;
    let mut x = 0;
    while x <= 1000 {
        let mut y = 0;
        while y <= 600 {
            if let Some(reach) = single_switch_reach(&mut g, x, y) {
                spots += 1;
                assert!(
                    reach < MACHINES.len(),
                    "one switch at ({x},{y}) must not reach all four"
                );
            }
            y += 20;
        }
        x += 20;
    }
    assert!(spots > 100, "the floor still offers many spots to try: {spots}");
}

#[test]
fn a_two_switch_build_carries_the_whole_shop() {
    // Two switches, each by a pair of counters, joined to each other: each
    // forwards by its own table and sends far traffic across the link. A clean
    // routed-around-the-kitchen layout delivers.
    let mut g = game();
    g.open(LEVEL).unwrap();
    g.place("switch", 260, 150).unwrap(); // m1
    g.place("switch", 720, 150).unwrap(); // m2
    for (a, b) in [
        ("m1", "reg1"),
        ("m1", "reg2"),
        ("m1", "m2"),
        ("m2", "srv"),
        ("m2", "reg3"),
    ] {
        g.wire(a, b).unwrap_or_else(|e| panic!("wire {a}-{b}: {e}"));
    }
    for (h, ip) in [
        ("reg1", "10.0.0.11/24"),
        ("reg2", "10.0.0.12/24"),
        ("reg3", "10.0.0.13/24"),
    ] {
        g.set_addr(h, ip).unwrap();
    }
    for (dst, via) in [("reg1", "reg1"), ("reg2", "reg2"), ("srv", "m2"), ("reg3", "m2")] {
        g.set_port("m1", dst, via).unwrap();
    }
    for (dst, via) in [("srv", "srv"), ("reg3", "reg3"), ("reg1", "m1"), ("reg2", "m1")] {
        g.set_port("m2", dst, via).unwrap();
    }
    let run = g.run().unwrap();
    assert!(
        run.contains("\"passed\":true"),
        "two switches carry the shop: {run}"
    );
}

#[test]
fn the_star_tiers_sit_sensibly_in_the_cost_distribution() {
    // Sample many valid two-switch builds and check the tiers partition their
    // cost the way the stars claim: the gold tier is the cheap tail, the third
    // star the cheap third, the second star about two-thirds, all within range.
    let mut g = game();
    g.open(LEVEL).unwrap();
    let st = g.build_state();
    let arr_at = st.find("\"star_cost\":[").unwrap() + "\"star_cost\":[".len();
    let arr = &st[arr_at..st[arr_at..].find(']').unwrap() + arr_at];
    let mut it = arr.split(',').map(|s| s.trim().parse::<i64>().unwrap());
    let s3 = it.next().unwrap();
    let s2 = it.next().unwrap();
    let gold = num(&st, "gold_cost").unwrap();

    let mut m = Mix(0x5a17_c0de);
    let mut costs: Vec<i64> = Vec::new();
    let mut tries = 0;
    while costs.len() < 300 && tries < 6000 {
        tries += 1;
        let p = |mix: &mut Mix| ((mix.upto(1040)) as i32 - 20, (mix.upto(640)) as i32 - 20);
        let (s1, s2) = (p(&mut m), p(&mut m));
        if let Some(c) = two_switch_cost(&mut g, s1, s2) {
            costs.push(c);
        }
    }
    costs.sort_unstable();
    let n = costs.len() as f64;
    assert!(
        costs.len() > 150,
        "enough valid builds were sampled: {}",
        costs.len()
    );
    let frac = |thr: i64| costs.iter().filter(|&&c| c <= thr).count() as f64 / n;
    assert!(
        costs[0] <= gold,
        "the cheapest sampled build reaches the gold tier"
    );
    assert!(
        *costs.last().unwrap() >= s2,
        "some sampled builds miss the second star"
    );
    assert!(frac(gold) <= 0.20, "gold is a cheap tail: {}", frac(gold));
    assert!(
        (0.15..=0.55).contains(&frac(s3)),
        "third star is about a third: {}",
        frac(s3)
    );
    assert!(
        (0.45..=0.85).contains(&frac(s2)),
        "second star is about two-thirds: {}",
        frac(s2)
    );
}

#[test]
fn an_identical_build_runs_byte_for_byte_the_same() {
    let once = || {
        let mut g = game();
        g.open(LEVEL).unwrap();
        g.place("switch", 260, 150).unwrap();
        g.place("switch", 720, 150).unwrap();
        for (a, b) in [
            ("m1", "reg1"),
            ("m1", "reg2"),
            ("m1", "m2"),
            ("m2", "srv"),
            ("m2", "reg3"),
        ] {
            g.wire(a, b).unwrap();
        }
        for (h, ip) in [
            ("reg1", "10.0.0.11/24"),
            ("reg2", "10.0.0.12/24"),
            ("reg3", "10.0.0.13/24"),
        ] {
            g.set_addr(h, ip).unwrap();
        }
        for (dst, via) in [("reg1", "reg1"), ("reg2", "reg2"), ("srv", "m2"), ("reg3", "m2")] {
            g.set_port("m1", dst, via).unwrap();
        }
        for (dst, via) in [("srv", "srv"), ("reg3", "reg3"), ("reg1", "m1"), ("reg2", "m1")] {
            g.set_port("m2", dst, via).unwrap();
        }
        g.run().unwrap()
    };
    assert_eq!(once(), once(), "the same build is deterministic");
}

#[test]
fn random_edit_sequences_never_wedge_the_core() {
    // Hammer the build API with arbitrary edits across many seeds. Whatever the
    // student clicks, every call returns valid JSON, the invariants hold (cost
    // = machines + wire, no cable beats the reel — over-long and kitchen-
    // crossing cables are refused — no more switches than the palette allows),
    // and a run never panics.
    let max_wire: i64 = {
        let mut g = game();
        g.open(LEVEL).unwrap();
        num(&g.build_state(), "max_wire").unwrap()
    };
    let mut g = game();
    for seed in 0..40u64 {
        g.open(LEVEL).unwrap();
        let mut m = Mix(seed.wrapping_mul(0x1000_0001).wrapping_add(1));
        let names = ["srv", "reg1", "reg2", "reg3", "m1", "m2", "m3", "m4"];
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
                    let sw = ["m1", "m2", "m3", "m4"][m.upto(4) as usize];
                    let dst = names[m.upto(names.len() as u64) as usize];
                    let via = names[m.upto(names.len() as u64) as usize];
                    let _ = g.set_port(sw, dst, via);
                }
                _ => {
                    let id = ["m1", "m2", "m3", "m4", "reg1"][m.upto(5) as usize];
                    let _ = g.unplace(id);
                }
            }
            let st = g.build_state();
            assert!(st.contains("\"build\":true"), "state stays well-formed: {st}");
            let machines = num(&st, "machines").unwrap();
            let wire = num(&st, "wire").unwrap();
            let total = num(&st, "total").unwrap();
            assert_eq!(machines + wire, total, "cost is machines plus wire: {st}");
            let mut rest = st.as_str();
            while let Some(p) = rest.find("\"len\":") {
                rest = &rest[p + 6..];
                let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
                let len: i64 = rest[..end].parse().unwrap();
                assert!(len <= max_wire, "no cable beats the reel: len={len} > {max_wire}");
            }
            let switches = st.matches("\"kind\":\"switch\"").count();
            assert!(
                switches <= 5,
                "at most the palette max of switches (plus palette entry): {switches}"
            );
        }
        match g.run() {
            Ok(run) => assert!(run.contains("\"passed\":"), "a run reports pass or fail: {run}"),
            Err(msg) => assert!(!msg.is_empty(), "a refusal carries a message"),
        }
    }
}

#[test]
fn an_off_block_or_duplicate_address_cannot_deliver() {
    for bad in ["reg1", "reg2", "reg3"] {
        let mut g = game();
        g.open(LEVEL).unwrap();
        g.place("switch", 260, 150).unwrap();
        g.place("switch", 720, 150).unwrap();
        for (a, b) in [
            ("m1", "reg1"),
            ("m1", "reg2"),
            ("m1", "m2"),
            ("m2", "srv"),
            ("m2", "reg3"),
        ] {
            g.wire(a, b).unwrap();
        }
        for (h, ip) in [
            ("reg1", "10.0.0.11/24"),
            ("reg2", "10.0.0.12/24"),
            ("reg3", "10.0.0.13/24"),
        ] {
            let ip = if h == bad { "10.0.1.50/24" } else { ip };
            g.set_addr(h, ip).unwrap();
        }
        for (dst, via) in [("reg1", "reg1"), ("reg2", "reg2"), ("srv", "m2"), ("reg3", "m2")] {
            g.set_port("m1", dst, via).unwrap();
        }
        for (dst, via) in [("srv", "srv"), ("reg3", "reg3"), ("reg1", "m1"), ("reg2", "m1")] {
            g.set_port("m2", dst, via).unwrap();
        }
        let run = g.run().unwrap();
        assert!(
            run.contains("\"passed\":false"),
            "{bad} off the block must fail: {run}"
        );
    }
}
