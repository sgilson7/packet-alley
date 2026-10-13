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
        ("gateway", vec![2]),
        ("route", vec![2]),
        ("ttl", vec![1]),
        ("arp", vec![1]),
        ("handshake", vec![2, 2]),
        ("transfer", vec![1, 1]),
    ]
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
        let out = g.run().unwrap_or_else(|e| panic!("run {id}: {e}"));
        println!("{id}: {}", if passed(&out) { "PASS" } else { "FAIL" });
        assert!(
            passed(&out),
            "level {id} did not pass with its intended choices: {out}"
        );
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
    // The quiz answer is option 0.
    assert!(g.answer(1).unwrap().contains("\"correct\":false"));
    assert!(g.answer(0).unwrap().contains("\"correct\":true"));
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
fn passing_the_quiz_right_adds_the_second_star() {
    let mut g = game();
    g.set_teacher(true);
    g.open("address").unwrap();
    g.choose(0, 1).unwrap();
    g.run().unwrap();
    // The quiz answer for "address" is option 0.
    let wrong = g.answer(1).unwrap();
    assert!(wrong.contains("\"correct\":false"));
    let right = g.answer(0).unwrap();
    assert!(right.contains("\"correct\":true"));
    // The star shows in the saved progress.
    assert!(g.save().contains("address"));
}
