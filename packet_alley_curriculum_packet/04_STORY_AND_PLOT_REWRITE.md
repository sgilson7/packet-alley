# Exact story and human-readable rewrite

All student-visible narrative **must** be taken from `data/verbatim_story_copy.json`. Do not have Claude improvise character voices, exposition, hints, debriefs, or filler sentences. Case labels are exact strings; any grammatical adjustment needs an explicit logged approved change.

## Throughline

The city of Packet Alley is preparing for its night market. The noodle shop is the local anchor. First the shop's own devices must talk, then it has to communicate across neighboring blocks, then people need reliable connections and complete documents. The final scene is a legitimate crowd of simultaneous users testing the limits of a small server. Players are the network technician throughout; the narrative uses real places and real work orders. No villain, forced crisis, or arbitrary plot twist is required.

Each dispatch is concise: a tangible problem, a reason the player should care, then a clear task. No paragraph explains a protocol; **technical concepts are supplied only by sourced textbook excerpts and interactive simulation state.** Keep the student responsible for choosing the repair.

## Four chapter openings and endings

| Chapter | Opening motivation | End point |
|---|---|---|
| Frames on the Block | The shop and upstairs desks need a dependable local network. | Upstairs opens with service working. |
| Routes Across Town | Services on other blocks need to exchange messages. | The city's longer route reaches the archive. |
| Handshakes and Delivery | People need their requests, pages and files to arrive intact. | The market receives the last report. |
| When Too Much Arrives | Legitimate traffic increases when the market opens. | Students explain the observed queue and loss. |

## Hand-off use

- Lookup story content by unchanged `level.id`, not sequence number.
- Inject `chapters[chapter].opening` once before the first level in a chapter, alongside its interactive demonstration. The read-and-do lesson is mandatory but can be replayed.
- `levels[id].story` appears in the mission brief; `task` appears with the interactive map. `debrief` appears **after** a committed attempt is evaluated and never before.
- `levels[id].predict_prompt` and `inspect_prompt` are scene prompts; do not replace with multiple choice with highlighted correct options.
- Preserve existing instrument/tooltips mechanically except where they are inaccurate; for any technical tooltip substitute a registered **verbatim excerpt** or map-measured state string, not self-authored technical prose.
- Preserve packet letters, device labels, zoom, map focus, and color-blind symbols.

The exact 19 sets of dispatch, task and debrief text are in the JSON. They are deliberately short and use ordinary language: no inflated dialogue and no sentence that tells the player which repair button to press.
