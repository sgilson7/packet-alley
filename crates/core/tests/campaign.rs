//! A probe over the shipped levels: every level builds, and the intended
//! "correct" set of choices passes it, while the starting (wrong) choices do
//! not. The correct choices are written here by hand from the briefs, not
//! read from the game, so the test would catch a level that quietly became
//! unwinnable or one that passes with the wrong answer.
//!
//!     cargo test -p pa_core --test campaign -- --nocapture
//!
//! prints how each level scored.

use std::fs;
use std::path::PathBuf;

use pa_core::game::Game;

fn data(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data")
        .join(name);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

fn game() -> Game {
    Game::new(&data("levels.json"), &data("copy.json"), "").expect("the shipped data loads")
}

fn passed(json: &str) -> bool {
    json.contains("\"passed\":true")
}

/// The intended correct choices for every level, by id: for each edit, the
/// option index that should be picked. Teacher mode is on so every level opens.
fn correct() -> Vec<(&'static str, Vec<usize>)> {
    vec![
        ("address", vec![1]),
        ("hub", vec![1]),
        ("loop", vec![1]),
        ("crowd", vec![1, 1]),
        ("backbone", vec![1, 1]),
        ("floor", vec![1, 1, 1]),
        ("gateway", vec![2]),
        ("route", vec![2]),
        ("ttl", vec![1]),
        ("point", vec![1, 1]),
        ("roads", vec![1, 1]),
        ("across", vec![1, 1, 1]),
        ("arp", vec![1]),
        ("handshake", vec![2, 2]),
        ("transfer", vec![1, 1]),
        ("window", vec![1, 1]),
        ("connect", vec![1, 2, 2]),
        ("reliable", vec![1, 1, 1]),
    ]
}

/// Levels migrated to the evidence-first flow (Phase 2): the id, an evidence
/// id to inspect, and the prediction option that is correct once the intended
/// choices are made. A level not listed here plays the old way.
fn predict_of(id: &str) -> Option<(&'static str, usize)> {
    match id {
        "address" => Some(("prn", 0)),
        "hub" => Some(("dev", 0)),
        "loop" => Some(("prn", 0)),
        "crowd" => Some(("box1", 0)),
        "backbone" => Some(("prn", 0)),
        "floor" => Some(("srv", 0)),
        "gateway" => Some(("r", 0)),
        "route" => Some(("r1", 0)),
        "ttl" => Some(("r1", 0)),
        "point" => Some(("pc", 0)),
        "roads" => Some(("r1", 0)),
        "across" => Some(("r1", 0)),
        "arp" => Some(("prn", 0)),
        "handshake" => Some(("pc", 0)),
        "transfer" => Some(("srv", 0)),
        "window" => Some(("srv", 0)),
        "connect" => Some(("pc", 0)),
        "reliable" => Some(("r1", 0)),
        _ => None,
    }
}

#[test]
fn every_level_passes_with_its_intended_choices() {
    let mut g = game();
    g.set_teacher(true);
    for (id, choices) in correct() {
        g.open(id).unwrap_or_else(|e| panic!("open {id}: {e}"));
        for (edit, &opt) in choices.iter().enumerate() {
            g.choose(edit, opt)
                .unwrap_or_else(|e| panic!("choose {id} {edit}={opt}: {e}"));
        }
        if let Some((ev, opt)) = predict_of(id) {
            g.inspect(ev).unwrap_or_else(|e| panic!("inspect {id}: {e}"));
            g.predict(opt).unwrap_or_else(|e| panic!("predict {id}: {e}"));
        }
        let out = g.run().unwrap_or_else(|e| panic!("run {id}: {e}"));
        println!("{id}: {}", if passed(&out) { "PASS" } else { "FAIL" });
        assert!(
            passed(&out),
            "level {id} did not pass with its intended choices: {out}"
        );
        if predict_of(id).is_some() {
            assert!(
                out.contains("\"right\":true"),
                "level {id} prediction should be graded right: {out}"
            );
        }
    }
}

#[test]
fn the_starting_choices_do_not_already_pass() {
    // Every level must require at least one change: opened at its start
    // options it should not already be solved, or the edit teaches nothing.
    let mut g = game();
    g.set_teacher(true);
    for (id, _) in correct() {
        g.open(id).unwrap();
        // A migrated level still needs its evidence and a prediction before it
        // will run; give the correct prediction so only the start config (not a
        // missing prediction) is what keeps it from passing.
        if let Some((ev, opt)) = predict_of(id) {
            g.inspect(ev).unwrap();
            g.predict(opt).unwrap();
        }
        let out = g.run().unwrap();
        assert!(!passed(&out), "level {id} passes before any change is made");
    }
}

#[test]
fn the_explainer_passes_by_being_watched_and_its_quiz_scores() {
    // The "flood" level has no edits: it is watched, not played, so it passes
    // the moment it is run, and its quiz still awards the knowledge star.
    let mut g = game();
    g.set_teacher(true);
    g.open("flood").expect("open flood");
    // flood is migrated to the evidence-first flow: inspect, then predict.
    g.inspect("sw").expect("inspect flood switch");
    g.predict(0).expect("predict flood");
    let out = g.run().expect("run flood");
    println!("flood: {}", if passed(&out) { "PASS" } else { "FAIL" });
    assert!(passed(&out), "the explainer should pass just by running: {out}");
    // It is meant to show a flood: the run must actually drop frames, or the
    // lesson has no pile-up to point at. The summary carries the exact line
    // "{n} frames were lost."; the quoted "0 ..." matches only a run with none.
    assert!(
        !out.contains("\"0 frames were lost.\""),
        "the flood should drop frames, but none were lost: {out}"
    );
    // The prediction (the link fills and sheds requests) is graded right.
    assert!(
        out.contains("\"right\":true"),
        "the flood prediction should be right: {out}"
    );
}

#[test]
fn a_fresh_player_sees_only_the_first_level_unlocked() {
    let g = game();
    let home = g.home();
    assert!(
        home.contains("\"locked\":true"),
        "later levels should start locked"
    );
    assert!(
        home.contains("\"id\":\"address\""),
        "the first level should be listed"
    );
}

#[test]
fn predicting_right_adds_the_final_star_for_address() {
    // "address" is migrated to the evidence-first flow: its final star comes
    // from a correct prediction graded against the run, not a quiz.
    let mut g = game();
    g.set_teacher(true);
    g.open("address").unwrap();
    g.choose(0, 1).unwrap();
    g.inspect("prn").unwrap();
    // Predict the computer: passes, but the prediction is wrong — no final star.
    g.predict(1).unwrap();
    let wrong = g.run().unwrap();
    assert!(wrong.contains("\"passed\":true"));
    assert!(wrong.contains("\"right\":false"));
    // Predict the printer: the run delivers there, so the final star is earned.
    g.predict(0).unwrap();
    let right = g.run().unwrap();
    assert!(right.contains("\"right\":true"));
    assert!(right.contains("\"stars\":[true,true,true]"));
    assert!(g.save().contains("address"));
}
