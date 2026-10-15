//! The end-of-zone "super" build levels: the student places machines, draws
//! the cables, assigns the addresses and fills the switch table from scratch,
//! under a cable budget. These tests drive the real shipped restaurant level
//! (`newspot`) through the core exactly as the page would, and prove it can be
//! solved, that the bill is what the geometry says, and that a wrong table or
//! an off-block address is caught.

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

/// Places one switch below the kitchen, wires the four machines to it in a
/// star, addresses the three registers, and fills the switch table.
fn solve(g: &mut Game) -> String {
    g.place("switch", 500, 420).expect("place the switch");
    // the switch is the only placed machine, so its id is m1
    for other in ["srv", "reg1", "reg2", "reg3"] {
        g.wire("m1", other)
            .unwrap_or_else(|e| panic!("wire m1-{other}: {e}"));
    }
    g.set_addr("reg1", "10.0.0.11/24").unwrap();
    g.set_addr("reg2", "10.0.0.12/24").unwrap();
    g.set_addr("reg3", "10.0.0.13/24").unwrap();
    // in a star every destination is the neighbour on its own cable
    for dst in ["srv", "reg1", "reg2", "reg3"] {
        g.set_port("m1", dst, dst)
            .unwrap_or_else(|e| panic!("table row {dst}: {e}"));
    }
    g.run().expect("the restaurant runs")
}

#[test]
fn the_restaurant_can_be_wired_and_passes() {
    let mut g = game();
    let out = solve(&mut g);
    assert!(has(&out, "\"passed\":true"), "a correct build delivers: {out}");
    // every register got its approval back
    assert!(has(&out, "\"met\":true"), "the goals are met: {out}");
}

#[test]
fn the_bill_is_what_the_geometry_costs() {
    let mut g = game();
    // before any wiring: the three registers + server are bought, no cable yet.
    let s0 = g.build_state();
    assert!(has(&s0, "\"wire\":0"), "no cable laid yet: {s0}");
    assert!(
        has(&s0, "\"machines\":1500"),
        "the fixed machines cost 1500: {s0}"
    );
    let out = solve(&mut g);
    // switch (90) + fixed (1500) = 1590 machines; cable 475+490+388+368 = 1721.
    assert!(has(&out, "\"machines\":1590"), "a switch adds 90: {out}");
    assert!(
        has(&out, "\"wire\":1721"),
        "the cable bill matches the distances: {out}"
    );
    assert!(
        has(&out, "\"total\":3311"),
        "the total is machines plus cable: {out}"
    );
    // 3311 sits in the middle third (<= 3325 but > 3293): two stars, no gold.
    assert!(
        has(&out, "\"stars\":[true,true,false]"),
        "a fair build earns two stars: {out}"
    );
    assert!(has(&out, "\"gold\":false"), "a fair build is not gold: {out}");
}

#[test]
fn a_tight_placement_earns_three_stars_and_the_gold() {
    // Tucked against the kitchen wall, near the geometric best the walls allow:
    // the cheapest few per cent of builds, so every visible star and the hidden
    // gold one.
    let mut g = game();
    g.place("switch", 416, 300).unwrap();
    for other in ["srv", "reg1", "reg2", "reg3"] {
        g.wire("m1", other).unwrap();
    }
    g.set_addr("reg1", "10.0.0.11/24").unwrap();
    g.set_addr("reg2", "10.0.0.12/24").unwrap();
    g.set_addr("reg3", "10.0.0.13/24").unwrap();
    for dst in ["srv", "reg1", "reg2", "reg3"] {
        g.set_port("m1", dst, dst).unwrap();
    }
    let out = g.run().unwrap();
    assert!(has(&out, "\"passed\":true"), "the tight build delivers: {out}");
    assert!(
        has(&out, "\"stars\":[true,true,true]"),
        "a tight build earns every visible star: {out}"
    );
    assert!(
        has(&out, "\"gold\":true"),
        "a tight build earns the gold star: {out}"
    );
}

#[test]
fn a_cable_that_overshoots_the_reel_is_refused() {
    let mut g = game();
    g.place("switch", 500, 420).unwrap();
    // reg1 is far; a switch dropped in the far corner cannot reach it within 560
    g.unplace("m1").unwrap();
    g.place("switch", 860, 110).unwrap(); // in the back-room corner
    let err = g.wire("m2", "reg2").unwrap_err();
    assert!(!err.is_empty(), "an over-long cable is refused");
}

#[test]
fn a_wrong_table_row_drops_the_frame() {
    let mut g = game();
    g.place("switch", 500, 420).unwrap();
    for other in ["srv", "reg1", "reg2", "reg3"] {
        g.wire("m1", other).unwrap();
    }
    g.set_addr("reg1", "10.0.0.11/24").unwrap();
    g.set_addr("reg2", "10.0.0.12/24").unwrap();
    g.set_addr("reg3", "10.0.0.13/24").unwrap();
    // point "to reach the server" at the wrong cable (toward reg1)
    g.set_port("m1", "srv", "reg1").unwrap();
    g.set_port("m1", "reg1", "reg1").unwrap();
    g.set_port("m1", "reg2", "reg2").unwrap();
    g.set_port("m1", "reg3", "reg3").unwrap();
    let out = g.run().unwrap();
    assert!(
        has(&out, "\"passed\":false"),
        "a wrong table row misdelivers: {out}"
    );
}

#[test]
fn an_off_block_address_cannot_reach_the_server() {
    let mut g = game();
    g.place("switch", 500, 420).unwrap();
    for other in ["srv", "reg1", "reg2", "reg3"] {
        g.wire("m1", other).unwrap();
    }
    // reg1 is put on the wrong block
    g.set_addr("reg1", "10.0.1.50/24").unwrap();
    g.set_addr("reg2", "10.0.0.12/24").unwrap();
    g.set_addr("reg3", "10.0.0.13/24").unwrap();
    for dst in ["srv", "reg1", "reg2", "reg3"] {
        g.set_port("m1", dst, dst).unwrap();
    }
    let out = g.run().unwrap();
    assert!(
        has(&out, "\"passed\":false"),
        "a register off the block cannot reach the server: {out}"
    );
}
// temp debug appended
#[test]
fn a_missing_table_row_drops_the_frame() {
    // A build switch forwards only by its table and floods nothing, so a row
    // the student never filled means that machine is unreachable.
    let mut g = game();
    g.place("switch", 500, 420).unwrap();
    for other in ["srv", "reg1", "reg2", "reg3"] {
        g.wire("m1", other).unwrap();
    }
    g.set_addr("reg1", "10.0.0.11/24").unwrap();
    g.set_addr("reg2", "10.0.0.12/24").unwrap();
    g.set_addr("reg3", "10.0.0.13/24").unwrap();
    // fill every row but the server's: the requests cannot be forwarded to it
    g.set_port("m1", "reg1", "reg1").unwrap();
    g.set_port("m1", "reg2", "reg2").unwrap();
    g.set_port("m1", "reg3", "reg3").unwrap();
    let out = g.run().unwrap();
    assert!(
        out.contains("\"passed\":false"),
        "an unfilled row leaves the server unreachable: {out}"
    );
}
