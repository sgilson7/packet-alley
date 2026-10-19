# Triage

Stage 7. Concrete changes from using the tool. The peer and Sam play it and
add rows here; each becomes a change to `data/` or a rule. Rows are numbered
T1, T2 and so on, and each commit that answers one names it ("(T1)").

| # | Request, in the person's words | Expected | What happened | Severity | Cost | Disposition |
| --- | --- | --- | --- | --- | --- | --- |
| T1 | Sam, 2026-10-19, after a K-12 classroom feasibility analysis of his Builder tools: "ok execute all of the changes you recommend to increase feasabilitiy, i approve make deploy in all builder made repos, and make deploy in the website, keep track of all changes made in the design log here" | The analysis's recommendations for this game: (a) the Builder privacy kit (`kits/privacy`): the stricter CSP and no-referrer, a boundary test for them, and a browser check that fails on any request after ready and on stored data, with the leak proof; (b) a plain message, not a blank page, when a school's device policy blocks WebAssembly (Chrome `DefaultJavaScriptJitSetting=block`), saying what is blocked and that a school's IT can allow it for sgilson7.github.io | Before: CSP without font/frame/worker/manifest `'none'`; no check on same-origin requests after ready or on storage; art fetched after ready; with WebAssembly blocked the page showed "The game data could not be loaded." | wanted | half a day | done: (a) with the storage rule tightened to the declared key only, since the brief keeps progress between visits (SECOND-ORDER row 40); (b) `blocked_wasm` in `data/copy.json`, shown by `blocked()` in `web/app.js`. Proven by `tests/breaks.json` (7 of 7 caught) and `scripts/prove-privacy.sh` (P1–P5 caught) |
