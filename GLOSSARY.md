# Glossary

What each word in this tool's strings means. Started in October 2026 with the words of the blocked-device message (SECOND-ORDER row 40); the game's older words are not yet entered. Write each entry as a `### Term` heading, one or two plain sentences, then an optional `Not:` line of words that must not be used for it. `crates/core/tests/copy.rs` checks the strings whose key starts with `blocked_` against the `Not:` words.

### JavaScript JIT

The browser setting, as Chrome names it, whose policy a school sets to block fast-compiled code. Blocking it can also block WebAssembly.

Not: Java

### WebAssembly

The compiled form of the game's rules, which the browser runs. Without it the game cannot start.

Not: plugin, extension, download, app
