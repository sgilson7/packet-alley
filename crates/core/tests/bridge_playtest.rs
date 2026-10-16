//! Playtest probes for "The Seven Bridges": not pass/fail assertions so much as
//! measurements of how forgiving the level is, to surface spots that would
//! block or confuse a student. Run with `--nocapture` to read the numbers.

use std::fs;
use std::path::PathBuf;

use pa_core::game::Game;

fn data(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data")
        .join(name);
    fs::read_to_string(&p).unwrap()
}
fn game() -> Game {
    let mut g = Game::new(&data("levels.json"), &data("copy.json"), "").unwrap();
    g.set_teacher(true);
    g.open("newbridge").unwrap();
    g
}

/// How wide a window of router x-positions lets the North router still wire over
/// a bridge to the Exchange router? A narrow window means a student must place
/// very precisely or get a confusing refusal.
#[test]
fn placement_tolerance_for_a_bridge_crossing() {
    let mut lo = i32::MAX;
    let mut hi = i32::MIN;
    for x in 100..=900 {
        let mut g = game();
        g.place("router", x, 120).unwrap(); // north router at varying x
        g.place("router", 435, 300).unwrap(); // exchange router, fixed
                                              // can we wire the two over the band of water between them?
        if g.wire("m1", "m2").is_ok() {
            lo = lo.min(x);
            hi = hi.max(x);
        }
    }
    println!(
        "north-router x window that can bridge to the Exchange router: [{lo}..{hi}] = {} units wide (icon is 100 wide)",
        hi - lo
    );
    assert!(hi >= lo, "there is at least one working spot");
}

/// If a student skips the north-bank switch and wires both north machines
/// straight to the router, do both still reach across — or does one silently
/// break (two interfaces on one block)?
#[test]
fn skipping_the_switch_and_wiring_two_hosts_to_one_router() {
    let mut g = game();
    g.place("router", 430, 120).unwrap(); // m1 north
    g.wire("regN", "m1")
        .expect("the first north host wires to the router");
    // the second machine on the same block should be refused with a teaching
    // message, not quietly misdeliver later.
    let err = g.wire("anl", "m1").unwrap_err();
    println!("second same-block host wired straight to a router → refused: {err}");
    assert!(err.contains("switch"), "the refusal points at a switch: {err}");
}

/// What does a student see if they wire everything correctly but never set the
/// hop count? (It defaults to the lowest offered.)
#[test]
fn forgetting_the_hop_count() {
    let mut g = game();
    place_correct(&mut g, false); // don't set ttl
    let out = g.run().unwrap();
    println!("TTL never set — passed={}", out.contains("\"passed\":true"));
    if let Some(i) = out.find("\"diagnosis\":") {
        println!("  diagnosis: {}", &out[i..(i + 300).min(out.len())]);
    }
}

fn place_correct(g: &mut Game, set_ttl: bool) {
    g.place("router", 430, 120).unwrap();
    g.place("router", 430, 480).unwrap();
    g.place("router", 700, 300).unwrap();
    g.place("router", 435, 300).unwrap();
    g.place("switch", 300, 300).unwrap();
    g.place("switch", 620, 110).unwrap();
    for (a, b) in [
        ("regN", "m6"),
        ("anl", "m6"),
        ("m6", "m1"),
        ("m1", "m4"),
        ("regS", "m2"),
        ("m2", "m4"),
        ("regL", "m3"),
        ("m3", "m4"),
        ("hq", "m5"),
        ("desk", "m5"),
        ("m5", "m4"),
    ] {
        g.wire(a, b).unwrap();
    }
    for r in ["m1", "m2", "m3"] {
        for blk in ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24", "10.0.9.0/24"] {
            g.set_route(r, blk, "m4").unwrap();
        }
    }
    g.set_route("m4", "10.0.1.0/24", "m1").unwrap();
    g.set_route("m4", "10.0.2.0/24", "m2").unwrap();
    g.set_route("m4", "10.0.3.0/24", "m3").unwrap();
    if set_ttl {
        g.set_ttl(6).unwrap();
    }
}
