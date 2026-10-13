# Measurements

Stage 4. The starting state, before the tool's own code was written.

- Base: the Builder template (`builder-setting/template`), copied in and its
  crates renamed `pa_core` / `pa_shim`.
- Template check at copy-in: `cargo test` green (the template's boundary tests),
  `scripts/build.sh` produced a counter page, browser check green in three
  engines.
- Tooling present: rustc 1.99, wasm-bindgen 0.2.127, wasm-tools, pdflatex,
  pdftocairo, Playwright 1.63.0 with Chromium, Firefox and WebKit.

What the finished MVP adds over the template is recorded in `HANDOFF.md`.
