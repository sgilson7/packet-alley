//! Phase 2, the evidence-first model. A small custom level with a `predict`
//! block, evidence and one fixed practice variant exercises the new core:
//! the run will not start until the student has inspected evidence and
//! committed a prediction; the prediction is graded against the real run, not
//! a stored option index; the final star needs both a pass and a correct
//! prediction; and a practice variant changes the numbers but awards nothing.
//!
//! Every assertion reads the JSON the page would receive, so these are the
//! same guarantees a browser sees.

use std::fs;
use std::path::PathBuf;

use pa_core::game::Game;

fn copy() -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/copy.json");
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("read copy.json: {e}"))
}

/// A one-level file: a computer, a switch and a printer on one block. The order
/// goes to the computer itself (does not reach the printer) until the student
/// points it at the printer. The prediction asks who receives the order.
fn levels() -> &'static str {
    r#"{
      "chapters":[{"id":"block","title":"B","layer":"L","blurb":"b","backdrop":"block"}],
      "levels":[{
        "id":"demo","chapter":"block","title":"Demo","story":"s","concept":["c"],"task":"t","ticks":20,
        "nodes":[
          {"id":"pc","kind":"host","name":"PC","x":180,"y":300,"ip":"10.0.0.10/24"},
          {"id":"sw","kind":"switch","name":"SW","x":500,"y":300},
          {"id":"prn","kind":"printer","name":"PRN","x":820,"y":300,"ip":"10.0.0.20/24"}],
        "links":[{"id":"a","a":"pc","b":"sw"},{"id":"b","a":"sw","b":"prn"}],
        "apps":[{"id":"order","kind":"udp","from":"pc","to":"10.0.0.10","at":1,"count":1,"every":3,"label":"order"}],
        "edits":[{"id":"dest","prompt":"Where?","target":{"app":"order","field":"to"},
                  "options":[{"label":"itself","value":"10.0.0.10"},{"label":"printer","value":"10.0.0.20"}],"start":0}],
        "goal":[{"kind":"delivered","app":"order","n":1,"text":"delivered"}],
        "bonus":{"kind":"no_drops","text":"no loss"},
        "quiz":{"question":"q","options":["a","b"],"answer":0,"why":"w"},
        "debrief":"d",
        "evidence":["pc","prn","sw"],
        "predict":{"prompt":"Who receives the order?","options":[
          {"label":"the printer","claim":{"kind":"delivered_to","node":"prn"}},
          {"label":"the computer","claim":{"kind":"delivered_to","node":"pc"}},
          {"label":"nobody","claim":{"kind":"not_delivered","app":"order"}}]},
        "variants":[{"seed":2,"overrides":[{"target":{"node":"pc","field":"ip"},"value":"10.0.0.11/24"}]}]
      }]}"#
}

fn game() -> Game {
    let mut g = Game::new(levels(), &copy(), "").expect("the demo level loads and validates");
    g.set_teacher(true);
    g.open("demo").expect("demo opens");
    g
}

fn has(json: &str, needle: &str) -> bool {
    json.contains(needle)
}

#[test]
fn a_run_waits_for_evidence_then_a_prediction() {
    let mut g = game();
    // Nothing inspected, nothing predicted.
    assert!(g.run().is_err(), "a run must wait for the evidence");
    g.inspect("pc").unwrap();
    // Evidence seen, but still no prediction.
    assert!(g.run().is_err(), "a run must wait for a prediction");
    g.predict(0).unwrap();
    assert!(g.run().is_ok(), "with evidence and a prediction the run proceeds");
}

#[test]
fn the_final_star_needs_a_pass_and_a_right_prediction() {
    // Point the order at the printer, predict the printer, and it is mastered.
    let mut g = game();
    g.choose(0, 1).unwrap();
    g.inspect("prn").unwrap();
    g.predict(0).unwrap();
    let out = g.run().unwrap();
    assert!(has(&out, "\"passed\":true"), "the order reaches the printer");
    assert!(has(&out, "\"right\":true"), "the prediction matches the run");
    assert!(
        has(&out, "\"stars\":[true,true,true]"),
        "pass, no-loss and prediction"
    );
}

#[test]
fn a_wrong_prediction_withholds_only_the_final_star() {
    // Delivered to the printer (passes, no loss), but predicted the computer.
    let mut g = game();
    g.choose(0, 1).unwrap();
    g.inspect("prn").unwrap();
    g.predict(1).unwrap();
    let out = g.run().unwrap();
    assert!(has(&out, "\"passed\":true"));
    assert!(
        has(&out, "\"right\":false"),
        "the printer received it, not the computer"
    );
    assert!(
        has(&out, "\"stars\":[true,true,false]"),
        "the final star is withheld"
    );
}

#[test]
fn the_same_option_is_right_or_wrong_by_the_run_not_its_index() {
    // Option 0 ("the printer") was correct above with the order pointed at the
    // printer. With the order left pointing at itself, the very same option is
    // graded wrong — so correctness comes from the run, not the option number
    // (Phase 2, PA8: no hardcoded right-answer index).
    let mut g = game();
    // leave the destination on its starting (itself) option
    g.inspect("prn").unwrap();
    g.predict(0).unwrap();
    let out = g.run().unwrap();
    assert!(has(&out, "\"passed\":false"), "it never reached the printer");
    assert!(
        has(&out, "\"right\":false"),
        "so 'the printer' is now the wrong prediction"
    );
    assert!(has(&out, "\"stars\":[false,false,false]"));
}

#[test]
fn predicting_a_failure_you_do_not_fix_earns_no_star() {
    // Correctly predict that nobody gets the order, but never fix it: the
    // prediction is right, yet the level is not passed, so no mastery star.
    let mut g = game();
    g.inspect("pc").unwrap();
    g.predict(2).unwrap(); // "nobody"
    let out = g.run().unwrap();
    assert!(has(&out, "\"passed\":false"));
    assert!(has(&out, "\"right\":true"), "nobody did receive it");
    assert!(
        has(&out, "\"stars\":[false,false,false]"),
        "a right call on an unfixed net earns nothing"
    );
}

#[test]
fn a_practice_variant_changes_the_numbers_and_awards_nothing() {
    let mut g = game();
    // The variant re-addresses the computer; its numbers appear in the view.
    let view = g.practice().unwrap();
    assert!(has(&view, "10.0.0.11"), "the practice case uses fresh addresses");
    // Solve the variant perfectly...
    g.choose(0, 1).unwrap();
    g.inspect("prn").unwrap();
    g.predict(0).unwrap();
    let out = g.run().unwrap();
    assert!(has(&out, "\"on_variant\":true"));
    assert!(has(&out, "\"passed\":true"));
    assert!(has(&out, "\"right\":true"));
    // ...and still earn no stars, because practice is for learning.
    assert!(
        has(&out, "\"stars\":[false,false,false]"),
        "a practice run never awards a star"
    );
}

#[test]
fn a_level_without_a_prediction_keeps_its_quiz_and_bonus_layout() {
    // The evidence-first flow is opt-in per level: a level with no `predict`
    // block plays exactly as before — the quiz is offered on a pass, the bonus
    // takes the third star, and the run reports no prediction. (All shipped
    // levels are migrated now, so this guards the fallback with a custom one.)
    let plain = r#"{
      "chapters":[{"id":"block","title":"B","layer":"L","blurb":"b","backdrop":"block"}],
      "levels":[{
        "id":"plain","chapter":"block","title":"Plain","story":"s","concept":["c"],"task":"t","ticks":20,
        "nodes":[
          {"id":"pc","kind":"host","name":"PC","x":180,"y":300,"ip":"10.0.0.10/24"},
          {"id":"sw","kind":"switch","name":"SW","x":500,"y":300},
          {"id":"prn","kind":"printer","name":"PRN","x":820,"y":300,"ip":"10.0.0.20/24"}],
        "links":[{"id":"a","a":"pc","b":"sw"},{"id":"b","a":"sw","b":"prn"}],
        "apps":[{"id":"order","kind":"udp","from":"pc","to":"10.0.0.10","at":1,"count":1,"every":3,"label":"order"}],
        "edits":[{"id":"dest","prompt":"Where?","target":{"app":"order","field":"to"},
                  "options":[{"label":"itself","value":"10.0.0.10"},{"label":"printer","value":"10.0.0.20"}],"start":0}],
        "goal":[{"kind":"delivered","app":"order","n":1,"text":"delivered"}],
        "bonus":{"kind":"no_drops","text":"no loss"},
        "quiz":{"question":"q","options":["a","b"],"answer":0,"why":"w"},
        "debrief":"d"
      }]}"#;
    let mut g = Game::new(plain, &copy(), "").expect("plain level loads");
    g.set_teacher(true);
    g.open("plain").unwrap();
    g.choose(0, 1).unwrap();
    let out = g.run().unwrap();
    assert!(has(&out, "\"passed\":true"));
    assert!(
        has(&out, "\"quiz\":{"),
        "a level with no prediction still shows its quiz"
    );
    assert!(has(&out, "\"predict\":null"), "and reports no prediction");
}

#[test]
fn a_shipped_practice_variant_is_winnable_with_the_same_logic() {
    // The practice variants on the simple-edit levels only shift the addresses,
    // so the same choice and the same prediction still solve them — and still
    // award nothing, because practice is for learning.
    let shipped = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/levels.json");
    let levels = fs::read_to_string(&shipped).unwrap();
    let mut g = Game::new(&levels, &copy(), "").expect("shipped data loads");
    g.set_teacher(true);
    g.open("hub").unwrap();
    let view = g.practice().unwrap();
    assert!(has(&view, "10.0.5."), "the practice case uses a shifted subnet");
    // swap the hub for a switch (option 1), inspect, predict no collision
    g.choose(0, 1).unwrap();
    g.inspect("dev").unwrap();
    g.predict(0).unwrap();
    let out = g.run().unwrap();
    assert!(has(&out, "\"on_variant\":true"));
    assert!(
        has(&out, "\"passed\":true"),
        "the variant solves the same way: {out}"
    );
    assert!(
        has(&out, "\"right\":true"),
        "and the prediction still grades: {out}"
    );
    assert!(
        has(&out, "\"stars\":[false,false,false]"),
        "practice awards nothing: {out}"
    );
}

#[test]
fn each_chapter_scene_watches_and_its_prediction_grades() {
    // Every chapter's demonstration scene: it passes by being watched (observe
    // goal), its intended prediction (option 0) is graded right against the
    // run, and it awards no stars.
    let shipped = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/levels.json");
    let levels = fs::read_to_string(&shipped).unwrap();
    let mut g = Game::new(&levels, &copy(), "").expect("shipped data loads");
    for (chapter, ev) in [("block", "sw"), ("town", "r"), ("wires", "cl"), ("storm", "sw")] {
        g.scene(chapter)
            .unwrap_or_else(|e| panic!("scene {chapter}: {e}"));
        g.inspect(ev)
            .unwrap_or_else(|e| panic!("inspect scene {chapter}: {e}"));
        g.predict(0)
            .unwrap_or_else(|e| panic!("predict scene {chapter}: {e}"));
        let out = g.run().unwrap_or_else(|e| panic!("run scene {chapter}: {e}"));
        assert!(
            has(&out, "\"passed\":true"),
            "scene {chapter} passes by watching: {out}"
        );
        assert!(
            has(&out, "\"right\":true"),
            "scene {chapter} option 0 is the right prediction: {out}"
        );
        assert!(
            has(&out, "\"on_variant\":false"),
            "a scene is not a variant: {out}"
        );
        assert!(
            has(&out, "\"stars\":[false,false,false]"),
            "a scene awards no stars: {out}"
        );
    }
    // Scenes are not level cards, not in the progression, and not in the star
    // total: the home still shows nineteen levels across fifty-seven stars.
    let home = g.home();
    assert!(
        home.contains("\"57 of 57 stars\"") || home.contains(" of 57 "),
        "scenes stay out of the star total: {home}"
    );
}
