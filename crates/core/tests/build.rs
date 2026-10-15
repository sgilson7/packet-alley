//! The end-of-zone "super" build levels: the student places machines, draws
//! the cables, assigns the addresses and fills the switch tables from scratch,
//! under a cable budget — and a cable may not run through the kitchen, so one
//! switch in the middle cannot reach everyone. These tests drive the real
//! shipped restaurant level (`newspot`) through the core exactly as the page
//! would.

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
    g.open("newspot").expect("the restaurant opens");
    g
}

fn has(j: &str, needle: &str) -> bool {
    j.contains(needle)
}

/// Two switches above the kitchen, each by one pair of counters, joined to each
/// other: m1 by the left registers, m2 by the bar and the server. Every cable
/// routes around the kitchen. This is a near-optimal (gold) layout.
fn two_switch(g: &mut Game) {
    g.place("switch", 260, 150).expect("place switch 1"); // m1
    g.place("switch", 720, 150).expect("place switch 2"); // m2
    for (a, b) in [
        ("m1", "reg1"),
        ("m1", "reg2"),
        ("m1", "m2"),
        ("m2", "srv"),
        ("m2", "reg3"),
    ] {
        g.wire(a, b).unwrap_or_else(|e| panic!("wire {a}-{b}: {e}"));
    }
    g.set_addr("reg1", "10.0.0.11/24").unwrap();
    g.set_addr("reg2", "10.0.0.12/24").unwrap();
    g.set_addr("reg3", "10.0.0.13/24").unwrap();
    // m1 reaches the left registers directly, the rest across the link to m2.
    for (dst, via) in [("reg1", "reg1"), ("reg2", "reg2"), ("srv", "m2"), ("reg3", "m2")] {
        g.set_port("m1", dst, via).unwrap();
    }
    // m2 reaches the server and the bar directly, the left registers via m1.
    for (dst, via) in [("srv", "srv"), ("reg3", "reg3"), ("reg1", "m1"), ("reg2", "m1")] {
        g.set_port("m2", dst, via).unwrap();
    }
}

#[test]
fn the_restaurant_can_be_wired_and_passes() {
    let mut g = game();
    two_switch(&mut g);
    let out = g.run().expect("the restaurant runs");
    assert!(
        has(&out, "\"passed\":true"),
        "a correct two-switch build delivers: {out}"
    );
    assert!(has(&out, "\"met\":true"), "the goals are met: {out}");
}

#[test]
fn the_tight_layout_is_cheap_and_earns_the_gold() {
    let mut g = game();
    two_switch(&mut g);
    let out = g.run().expect("runs");
    // two switches ($90 each) = $180; cable 145+376+460+145+376 = 1502.
    assert!(has(&out, "\"machines\":180"), "two switches cost 180: {out}");
    assert!(
        has(&out, "\"wire\":1502"),
        "the cable bill matches the routes: {out}"
    );
    assert!(
        has(&out, "\"total\":1682"),
        "the total is machines plus cable: {out}"
    );
    assert!(
        has(&out, "\"stars\":[true,true,true]"),
        "a tight build earns every visible star: {out}"
    );
    assert!(has(&out, "\"gold\":true"), "and the hidden gold star: {out}");
}

#[test]
fn a_single_central_switch_cannot_reach_both_sides() {
    // One switch cannot serve everyone: a cable to the far side runs through the
    // kitchen and is refused. (That no spot at all works is proven in the sweep.)
    let mut g = game();
    g.place("switch", 300, 300).unwrap();
    g.wire("m1", "reg1").expect("the near register wires fine");
    let err = g.wire("m1", "srv").unwrap_err();
    assert!(
        !err.is_empty(),
        "a cable across the kitchen to the far side is refused"
    );
}

#[test]
fn a_cable_through_the_kitchen_is_refused() {
    // A cable whose straight line crosses the kitchen is rejected — even when it
    // is well within the reel, so this isolates the no-cross rule.
    let mut g = game();
    g.place("switch", 450, 150).unwrap(); // above the kitchen, within reach of reg2
    let err = g.wire("m1", "reg2").unwrap_err();
    assert!(
        err.contains("kitchen"),
        "a cable that cuts through the kitchen is refused: {err}"
    );
    // routing one around it (to a same-side register) is fine
    assert!(
        g.wire("m1", "reg1").is_ok(),
        "a cable that clears the kitchen is allowed"
    );
}

#[test]
fn a_cable_that_overshoots_the_reel_is_refused() {
    let mut g = game();
    g.place("switch", 20, 300).unwrap();
    // the far corner is well past the reel's reach
    assert!(g.wire("m1", "reg3").is_err(), "an over-long cable is refused");
}

#[test]
fn a_wrong_table_row_drops_the_frame() {
    let mut g = game();
    two_switch(&mut g);
    // point m1's "to reach the server" at a left register instead of m2
    g.set_port("m1", "srv", "reg1").unwrap();
    let out = g.run().unwrap();
    assert!(
        has(&out, "\"passed\":false"),
        "a wrong table row misdelivers: {out}"
    );
}

#[test]
fn a_missing_table_row_drops_the_frame() {
    let mut g = game();
    two_switch(&mut g);
    // clear m2's row for the server: the far requests can no longer be forwarded
    g.set_port("m2", "srv", "").unwrap();
    let out = g.run().unwrap();
    assert!(
        has(&out, "\"passed\":false"),
        "an unfilled row leaves a machine unreachable: {out}"
    );
}

#[test]
fn an_off_block_address_cannot_reach_the_server() {
    let mut g = game();
    two_switch(&mut g);
    g.set_addr("reg1", "10.0.1.50/24").unwrap(); // off the shop's block
    let out = g.run().unwrap();
    assert!(
        has(&out, "\"passed\":false"),
        "a register off the block cannot reach the server: {out}"
    );
}

#[test]
fn a_switch_inspector_shows_a_structured_mac_table_and_mac() {
    // Every build machine reports its MAC in build_state, and a placed switch's
    // inspector draws a titled MAC table.
    let mut g = game();
    two_switch(&mut g);
    let st = g.build_state();
    assert!(has(&st, "\"mac\":\"02:"), "machines report a MAC: {st}");
    // node index 4 is the first placed switch (reg1..srv are 0..3)
    let node = g.node(4, 0, 1).expect("the switch inspects");
    assert!(
        has(&node, "\"title\":\"MAC address table\""),
        "the switch shows a MAC table: {node}"
    );
}
