# Licences

Packet Alley is MIT licensed; see `LICENSE`.

## Third-party

- **Trystero** — not used in this project (there is no online mode).
- All art is original, drawn in TikZ (`tools/`) and compiled with pdflatex and
  pdftocairo. No image asset is vendored.
- No fonts, scripts, styles or services are loaded from another origin; the
  page runs entirely from its own files (checked by `tests/browser/check.py`).

## Textbook excerpts (CC BY-SA 3.0)

The device and mechanism explanations shown in the game (the hub, switch and
router inspectors) and the "From the networking textbook" passage in each
chapter's brief quote short passages **verbatim** from *Computer Networking:
Principles, Protocols and Practice*, 3rd edition, by Olivier Bonaventure and
contributors (`github.com/cnp3/ebook`, book at
`beta.computer-networking.info`). That work is licensed **Creative Commons
Attribution–ShareAlike 3.0 Unported** (`creativecommons.org/licenses/by-sa/3.0/`).

These excerpts are **not** covered by Packet Alley's MIT licence; they remain
under CC BY-SA 3.0 and are reproduced with attribution and without modification.
Each quoted string keeps an inline credit, and `data/textbook_extracts.json`
records, for every passage, its source URL, the exact extracted text, a SHA-256
of that text, and the extraction date, so any reader can check the quotation
against the original page. The extraction tool and source manifest are in
`packet_alley_curriculum_packet/`.
