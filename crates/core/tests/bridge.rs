//! The network-layer super level, "The Seven Bridges": the student places a
//! mix of boxes (routers, a switch), wires them over the bridges (never the
//! water), and fills every router's forwarding table by hand — the routed twin
//! of the restaurant's switch tables. These tests drive the shipped level
//! (`newbridge`) through the core exactly as the page would.

use std::fs;
use std::path::PathBuf;

use pa_core::game::Game;

fn data(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data")
        .join(name);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {name}: {e}"))
}

fn game() -> Game {
    let mut g = Game::new(&data("levels.json"), &data("copy.json"), "").expect("shipped data loads");
    g.set_teacher(true);
    g.open("newbridge").expect("the bridges open");
    g
}

fn has(j: &str, needle: &str) -> bool {
    j.contains(needle)
}

/// A star-on-the-island gold layout: a router by each branch (m1 north, m2
/// south, m3 island) all homing to the Exchange router (m4) over a bridge. A
/// switch (m5) joins the card-auth HQ and the desk on the central island; a
/// second switch (m6) joins the register and the analytics logger on the north
/// bank — two multi-host segments that each need a switch, not a hub.
fn star(g: &mut Game) {
    star_with(g, "switch");
}

/// The same layout, but the Exchange's aggregator is `exch` — a switch in the
/// gold build, a hub to show the wrong box loses the servers' replies.
fn star_with(g: &mut Game, exch: &str) {
    g.place("router", 430, 120).expect("north router"); // m1
    g.place("router", 430, 480).expect("south router"); // m2
    g.place("router", 700, 300).expect("island router"); // m3
    g.place("router", 435, 300).expect("exchange router"); // m4
    g.place(exch, 300, 300).expect("exchange box"); // m5
    g.place("switch", 620, 110).expect("north switch"); // m6
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
        g.wire(a, b).unwrap_or_else(|e| panic!("wire {a}-{b}: {e}"));
    }
    // each branch router sends everything far toward the Exchange router
    for r in ["m1", "m2", "m3"] {
        for blk in ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24", "10.0.9.0/24"] {
            // skip the router's own block (the branch routers' own blocks differ,
            // but set_route of a directly-connected block is simply harmless)
            g.set_route(r, blk, "m4").unwrap();
        }
    }
    // the Exchange router points each branch block down its own bridge
    g.set_route("m4", "10.0.1.0/24", "m1").unwrap();
    g.set_route("m4", "10.0.2.0/24", "m2").unwrap();
    g.set_route("m4", "10.0.3.0/24", "m3").unwrap();
    g.set_ttl(6).unwrap();
}

#[test]
fn the_seven_bridges_can_be_wired_and_passes() {
    let mut g = game();
    star(&mut g);
    let out = g.run().expect("the bridges run");
    assert!(
        has(&out, "\"passed\":true"),
        "a correct routed build delivers: {out}"
    );
}

#[test]
fn a_correct_build_earns_the_gold() {
    let mut g = game();
    star(&mut g);
    let out = g.run().expect("runs");
    // 4 routers ($300) + 2 switches ($90) = $1,380 of gear.
    assert!(
        has(&out, "\"machines\":1380"),
        "gear is four routers and two switches: {out}"
    );
    assert!(
        has(&out, "\"gold\":true"),
        "the star-on-the-island build is cheapest-few: {out}"
    );
}

#[test]
fn a_wrong_signpost_misdelivers() {
    let mut g = game();
    star(&mut g);
    // point the Exchange router's road to the north block at the island router
    g.set_route("m4", "10.0.1.0/24", "m3").unwrap();
    let out = g.run().unwrap();
    assert!(
        has(&out, "\"passed\":false"),
        "a wrong signpost strands the north branch: {out}"
    );
}

#[test]
fn a_missing_signpost_is_a_black_hole() {
    let mut g = game();
    star(&mut g);
    // clear the Exchange router's road to the south block
    g.set_route("m4", "10.0.2.0/24", "").unwrap();
    let out = g.run().unwrap();
    assert!(
        has(&out, "\"passed\":false"),
        "an unfilled row drops the south branch: {out}"
    );
}

#[test]
fn too_few_hops_expire_in_transit() {
    let mut g = game();
    star(&mut g);
    g.set_ttl(2).unwrap(); // two hops can't cross two routers
    let out = g.run().unwrap();
    assert!(
        has(&out, "\"passed\":false"),
        "a low hop count dies before the Exchange: {out}"
    );
}

#[test]
fn a_hub_at_the_exchange_loses_replies() {
    // The card-auth HQ and the desk answer on the same segment; a hub repeats
    // every signal to every port, so concurrent replies collide and are lost.
    let mut g = game();
    star_with(&mut g, "hub");
    let out = g.run().unwrap();
    assert!(
        has(&out, "\"passed\":false"),
        "a hub where several machines answer at once drops replies: {out}"
    );
}

#[test]
fn a_cable_cannot_cross_the_water() {
    let mut g = game();
    // a router on the north bank, over open water (not a bridge gap) from the HQ
    g.place("router", 120, 80).unwrap(); // m1
    let err = g.wire("m1", "hq").unwrap_err();
    assert!(
        err.contains("water") || err.contains("bridge"),
        "a cable over open water is refused, and the message names the water, not the kitchen: {err}"
    );
}
