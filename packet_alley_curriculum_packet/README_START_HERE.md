# Packet Alley: curriculum and narrative rewrite packet

**Status:** researched and authored handoff, **not** an edited or deployed game. Source review: public `sgilson7/packet-alley` `main` as viewed on 15 October 2026. This environment could read the public JSON, README and Builder spec, but could not clone or run the repository locally. No full Rust/test suite run is claimed.

**First read:** `01_CURRENT_GAME_AUDIT.md`, `02_TEXTBOOK_GAP_ANALYSIS.md`, `03_TEXTBOOK_EXERCISE_MAP.md`, `04_STORY_AND_PLOT_REWRITE.md`, `05_ZONE_LESSONS.md`, `06_LEVEL_REDESIGN.md`, `07_BUILDER_CONSTRAINTS.md`, `08_CLAUDE_CODE_EXECUTION.md`, `09_COPYRIGHT_PROVENANCE.md`, `10_TESTS_AND_ACCEPTANCE.md`.

**Machine-readable contracts:** `data/verbatim_story_copy.json` contains **the exact new human-readable title, story, mission, outcome and instruction copy** for all 19 existing IDs plus four chapter intros. `data/level_blueprints.json` describes interaction/learning checks for the same 19 IDs. `data/verbatim_interface_and_technical_routes.json` maps old UI and mechanical copy to exact overrides or sourced excerpt IDs. `data/textbook_source_manifest.json` identifies fifteen short instructional excerpts to extract *verbatim* from the openly licensed book. `scripts/extract_textbook.py` performs deterministic text extraction to a local JSON file: the coding agent must not replace an unfound excerpt with invented explanation.

### What is intentionally not delivered as a ready-to-apply patch

The repo was not cloned here. The exact UI's import contracts, Rust validation APIs and schema are to be verified on the local checkout. The replacement files are a **canonical verbatim content source plus an executable extraction tool**, not an asserted drop-in schema match. Agent must adapt **only the plumbing** to use these strings verbatim and report any source drift in `SECOND-ORDER.md`. Source text extracted from a CC-licensed textbook must retain its attribution and applicable share-alike license; it should not be republished as MIT-licensed original game prose.

### Sources

- Game: https://github.com/sgilson7/packet-alley, `BRIEF.md`, `BUILDER-SPEC.md`, `README.md`, `data/levels.json`, `data/copy.json`.
- Builder: https://github.com/sgilson7/builder-setting
- Book and license: https://github.com/cnp3/ebook ; https://beta.computer-networking.info/syllabus/default/
- Exercises: https://beta.computer-networking.info/syllabus/default/exercises/lan.html ; https://beta.computer-networking.info/syllabus/default/exercises/tcp.html ; https://beta.computer-networking.info/syllabus/default/exercises/tcp-2.html

**Non-negotiable:** no answer-bearing hints in student prompts; no hidden reasoning in HTML or JavaScript; no prompt that says precisely which option to click; no free grade-awarding menu guessing; no unsafe active attack construction. Preserve the 2D map, direct object manipulation, packet animation, accessible labels, keyboard paths, speed controls and inspectors.


## Running this packet

```bash
python3 scripts/validate_packet.py
python3 scripts/extract_textbook.py  # on a networked machine; does not invent missing prose
```

The source extractor must be run in the cloned game developer environment and its results checked against source and attribution records before technical explanation strings are wired into the browser. The pre-run validation here checks **packet completeness**, not game integration.
