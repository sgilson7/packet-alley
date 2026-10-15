// The page draws what the core returns and decides nothing. Every call goes
// through the shim, which returns {"ok": ...} or {"error": "..."}. Every
// number the page uses comes from the core or from data/display.json; the
// page holds no rule and no literal but 0 and 1.
import init, { Game } from "./pkg/pa_shim.js";

const SVGNS = ["http:", "", "www.w3.org", "2000", "svg"].join("/");
const SAVE_KEY = "packet-alley-save";
const el = (id) => document.getElementById(id);

let game = null;
let copy = {};
let display = { speeds: [], default_speed: 0 };
let rafId = 0;
// The open on-map chooser's pointing state: the edit it is for and, for a
// "point at a machine" edit, which option each candidate device picks. The
// node elements are kept by id so candidates can be highlighted.
let pointing = null;
// When a chapter's demonstration scene is shown before a level, the level to
// open once the scene is done.
let pendingLevel = null;
const nodeEls = {};
const cur = { view: null, result: null, playing: false, ms: 0, last: 0, speedIdx: 0, logCount: 0, ran: false };

function t(key) {
  return copy[key] || "";
}

function loadSave() {
  try {
    return localStorage.getItem(SAVE_KEY) || "";
  } catch (e) {
    return "";
  }
}

function store() {
  try {
    localStorage.setItem(SAVE_KEY, call("save"));
  } catch (e) {
    /* a browser that refuses storage just forgets between visits */
  }
}

// Calls a core method and returns its value, or null (showing the message)
// on an error.
function call(method, ...args) {
  const r = JSON.parse(game[method](...args));
  if (r.error) {
    el("message").textContent = r.error;
    return null;
  }
  el("message").textContent = "";
  return r.ok;
}

function clear(node) {
  while (node.firstChild) {
    node.removeChild(node.firstChild);
  }
}

function tag(name, cls, text) {
  const n = document.createElement(name);
  if (cls) {
    n.className = cls;
  }
  if (text !== undefined) {
    n.textContent = text;
  }
  return n;
}

function svg(name, attrs) {
  const n = document.createElementNS(SVGNS, name);
  for (const k in attrs) {
    n.setAttribute(k, attrs[k]);
  }
  return n;
}

function mpt() {
  return display.speeds[cur.speedIdx].ms_per_tick;
}

// Map zoom: the slider's value (a string) becomes the --zoom custom property
// the stylesheet scales the map by. No number is named here; the slider's
// range and step live in the HTML, and the buttons nudge it with stepUp and
// stepDown, which default to one step.
function applyZoom() {
  el("map-wrap").style.setProperty("--zoom", el("zoom").value);
}

function setMode(mode) {
  document.body.dataset.mode = mode;
  el("menu").hidden = mode !== "menu";
  el("level").hidden = mode !== "level";
  el("home-btn").hidden = mode === "menu";
}

// ---------- menu ----------

function renderMenu() {
  const home = call("home");
  if (!home) {
    return;
  }
  el("star-count").textContent = home.stars;
  el("teacher").checked = home.teacher;
  el("teacher-label").textContent = t(home.teacher ? "menu_teacher_on" : "menu_teacher_off");
  const box = el("chapters");
  clear(box);
  for (const ch of home.chapters) {
    const sec = tag("section", "chapter");
    sec.appendChild(tag("p", "layer", ch.layer));
    sec.appendChild(tag("h2", null, ch.title));
    sec.appendChild(tag("p", "blurb", ch.blurb));
    const grid = tag("div", "levels");
    for (const lv of ch.levels) {
      const card = tag("button", "level-card" + (lv.locked ? " locked" : ""));
      card.type = "button";
      card.appendChild(tag("span", "num", lv.number));
      card.appendChild(tag("span", "name", lv.title));
      const stars = tag("span", "stars");
      for (const got of lv.stars) {
        const s = tag("span", got ? "full" : "empty", got ? "★" : "☆");
        stars.appendChild(s);
      }
      card.appendChild(stars);
      if (lv.locked) {
        card.appendChild(tag("span", "lock-note", t("err_locked")));
        card.disabled = true;
      } else {
        card.addEventListener("click", () => enterLevel(lv.id, ch.id, ch.scene, home.teacher));
      }
      grid.appendChild(card);
    }
    sec.appendChild(grid);
    box.appendChild(sec);
  }
  setMode("menu");
}

// ---------- level ----------

// Entering a chapter: if its demonstration scene has not been watched (and the
// player is not in teacher mode), show the scene first, then the level.
function enterLevel(levelId, chapterId, scene, teacher) {
  if (scene && scene.id && !scene.seen && !teacher) {
    pendingLevel = levelId;
    openScene(chapterId);
  } else {
    openLevel(levelId);
  }
}

function openScene(chapterId) {
  const view = call("scene", chapterId);
  if (!view) {
    return;
  }
  cur.view = view;
  cur.ran = false;
  cur.result = null;
  cur.playing = false;
  cur.sceneChapter = chapterId;
  renderLevel();
  setMode("level");
  window.scrollTo(0, 0);
}

function openLevel(id) {
  const view = call("open", id);
  if (!view) {
    return;
  }
  cur.view = view;
  cur.ran = false;
  cur.result = null;
  cur.playing = false;
  renderLevel();
  setMode("level");
  window.scrollTo(0, 0);
}

function renderLevel() {
  const v = cur.view;
  const sc = !!v.scene;
  el("l-chapter").textContent = (sc ? t("scene_tag") + " · " : "") + v.chapter + " · " + v.layer;
  el("l-title").textContent = sc ? v.title : v.number + ": " + v.title;
  el("l-story").textContent = v.story;
  el("task-h").textContent = t("panel_task");
  el("l-task").textContent = v.task;
  // A simplified-model caveat (e.g. Go-Back-N, not full TCP).
  const note = el("l-note");
  note.hidden = !v.note;
  if (v.note) {
    note.textContent = v.note;
  }
  // "What this level teaches" gives the answer away, so for a scored level it
  // is held back until the level is passed (shown in the result). A scene is a
  // demonstration, so it keeps its teaching points up front.
  el("concept-h").hidden = !sc;
  el("l-concept").hidden = !sc;
  if (sc) {
    el("concept-h").textContent = t("panel_concept");
    const cl = el("l-concept");
    clear(cl);
    for (const c of v.concept) {
      cl.appendChild(tag("li", null, c));
    }
  }
  // A verbatim passage from the open networking textbook, with attribution.
  const tb = el("l-textbook");
  tb.hidden = !v.textbook;
  if (v.textbook) {
    el("textbook-h").textContent = t("panel_textbook");
    el("l-textbook-text").textContent = v.textbook;
    el("l-textbook-src").textContent = v.textbook_src;
  }
  // A demonstration scene has no goals, bonus or edits — just inspect, predict
  // and watch. A scored level shows them as before.
  el("goals-h").hidden = sc;
  el("l-goals").hidden = sc;
  el("l-bonus").hidden = sc;
  const editsHead = el("edits-h").parentElement;
  editsHead.hidden = sc;
  el("edits-hint").hidden = sc;
  el("l-edits").hidden = sc;
  if (!sc) {
    el("goals-h").textContent = t("panel_goals");
    const gl = el("l-goals");
    clear(gl);
    for (const g of v.goals) {
      gl.appendChild(tag("li", null, g));
    }
    el("l-bonus").textContent = "★ " + v.bonus;
    el("edits-h").textContent = t("panel_edits");
    el("edits-hint").textContent = t("panel_edits_hint");
    renderEdits(v);
    el("panel-toggle").textContent = t("panel_hide");
    el("panel-toggle").setAttribute("aria-expanded", "true");
  }
  renderPredict(v);
  updateCommit(v);

  el("backdrop").src = "art/backdrops/" + v.backdrop + ".png";
  renderMap(v);
  renderLegend(v);
  markEvidence(v);

  el("legend-h").textContent = t("panel_legend");
  el("log-h").textContent = t("panel_log");
  el("hud").hidden = true;
  el("controls").hidden = true;
  resetLog();
}

function clearLogRows() {
  logRows.length = 0;
  renderLogChips();
  renderLogRows();
}

// One option button, shown the same way in the side panel and in the on-map
// chooser: a device palette shows the device's icon, a "point at a machine"
// pick shows the target machine's icon and name, everything else shows words.
function optionButton(e, ei, o, oi) {
  const b = tag("button", "opt" + (o.chosen ? " chosen" : ""));
  b.type = "button";
  b.setAttribute("aria-pressed", o.chosen ? "true" : "false");
  if (e.ui === "palette" && o.nkind) {
    b.classList.add("opt-dev");
    const img = document.createElement("img");
    img.className = "opt-icon";
    img.src = "art/icons/" + o.nkind + ".png";
    img.alt = "";
    b.appendChild(img);
    b.appendChild(tag("span", null, o.label));
  } else if (e.ui === "device" && o.node) {
    b.classList.add("opt-dev");
    const img = document.createElement("img");
    img.className = "opt-icon";
    img.src = "art/icons/" + o.nkind + ".png";
    img.alt = "";
    b.appendChild(img);
    const text = tag("span", "opt-dev-text");
    text.appendChild(tag("span", "opt-dev-name", o.nname));
    text.appendChild(tag("span", "opt-dev-sub", o.label));
    b.appendChild(text);
  } else {
    b.textContent = o.label;
  }
  b.addEventListener("click", () => choose(ei, oi));
  return b;
}

function editSection(e, ei) {
  const wrap = tag("div", "edit");
  wrap.appendChild(tag("p", "prompt", e.prompt));
  const opts = tag("div", "opts" + (e.ui === "palette" || e.ui === "device" ? " opts-dev" : ""));
  e.options.forEach((o, oi) => opts.appendChild(optionButton(e, ei, o, oi)));
  wrap.appendChild(opts);
  return wrap;
}

function renderEdits(v) {
  const box = el("l-edits");
  clear(box);
  v.edits.forEach((e, ei) => box.appendChild(editSection(e, ei)));
}

// ---------- evidence-first prediction (Phase 2) ----------

// The prediction step, shown only for a migrated level (one that carries a
// `predict` block). The student commits an outcome before the run; the core
// grades it against the run, so no option is marked correct here.
function renderPredict(v) {
  const box = el("l-predict");
  if (!v.predict) {
    box.hidden = true;
    return;
  }
  box.hidden = false;
  el("predict-h").textContent = t("panel_predict");
  el("predict-hint").textContent = t("predict_hint");
  el("predict-prompt").textContent = v.predict.prompt;
  const opts = el("predict-opts");
  clear(opts);
  v.predict.options.forEach((o, i) => {
    const picked = v.predict.chosen === i;
    const b = tag("button", "opt" + (picked ? " chosen" : ""));
    b.type = "button";
    b.setAttribute("aria-pressed", picked ? "true" : "false");
    b.textContent = o.label;
    b.addEventListener("click", () => predict(i));
    opts.appendChild(b);
  });
}

function predict(i) {
  const view = call("predict", i);
  if (!view) {
    return;
  }
  cur.view = view;
  renderPredict(cur.view);
  updateCommit(cur.view);
}

// The run button is never disabled — a greyed-out button that silently blocks
// is confusing. It always does something: it runs, or (on a level that still
// wants a prediction) a hint says so and clicking explains.
function updateCommit(v) {
  const label = v.predict ? "btn_commit" : cur.ran ? "btn_rerun" : "btn_run";
  el("run-btn").textContent = t(label);
  const needPredict = !!v.predict && (v.predict.chosen === null || v.predict.chosen === undefined);
  el("commit-hint").hidden = !needPredict;
  if (needPredict) {
    el("commit-hint").textContent = t("commit_hint");
  }
}

// Glow the devices still to be inspected for evidence; a device already looked
// at loses the glow.
function markEvidence(v) {
  for (const id in nodeEls) {
    nodeEls[id].classList.remove("evidence");
  }
  if (!v.predict) {
    return;
  }
  for (const id of v.evidence) {
    if (!v.evidence_seen.includes(id) && nodeEls[id]) {
      nodeEls[id].classList.add("evidence");
    }
  }
}

// Records that a device was inspected, when the open level asks for evidence.
function recordInspect(id) {
  const v = cur.view;
  if (!v.predict || !v.evidence.includes(id) || v.evidence_seen.includes(id)) {
    return;
  }
  const view = call("inspect", id);
  if (!view) {
    return;
  }
  cur.view = view;
  renderPredict(cur.view);
  markEvidence(cur.view);
  updateCommit(cur.view);
}

// The on-map chooser for one badge's group of edits. For a "point at a
// machine" edit it also lights up the candidate machines so the student can
// pick one straight off the map.
function openChooser(group) {
  const v = cur.view;
  closeChooser();
  // On an advanced level a device may be both evidence and the spot an edit
  // badge sits on; opening that badge also counts as inspecting the device, so
  // the evidence can always be gathered.
  group.forEach((ei) => {
    const e = v.edits[ei];
    const n = v.nodes.find((n) => n.cx === e.ax && n.cy === e.ay);
    if (n) {
      recordInspect(n.id);
    }
  });
  // A "point at a machine" edit is answered on the map, not in a popover that
  // would cover the machines; everything else gets the popover.
  const body = el("chooser-body");
  const other = [];
  group.forEach((ei) => {
    const e = v.edits[ei];
    if (!pointing && e.ui === "device") {
      const byNode = {};
      e.options.forEach((o, oi) => {
        if (o.node) {
          byNode[o.node] = oi;
        }
      });
      pointing = { ei: ei, byNode: byNode, prompt: e.prompt };
    } else {
      other.push(ei);
    }
  });
  if (pointing) {
    for (const id in pointing.byNode) {
      if (nodeEls[id]) {
        nodeEls[id].classList.add("candidate");
      }
    }
    el("map-hint").textContent = pointing.prompt + " " + t("point_hint");
    el("map-hint").hidden = false;
  }
  if (other.length) {
    other.forEach((ei) => body.appendChild(editSection(v.edits[ei], ei)));
    const ch = el("chooser");
    ch.hidden = false;
    positionChooser(v.edits[other[0]]);
  }
}

function positionChooser(e) {
  const svgEl = el("map");
  const pt = svgEl.createSVGPoint();
  pt.x = e.ax;
  pt.y = e.ay;
  const sp = pt.matrixTransform(svgEl.getScreenCTM());
  const ch = el("chooser");
  const base = (ch.offsetParent || ch.parentElement).getBoundingClientRect();
  ch.style.setProperty("left", sp.x - base.left + "px");
  ch.style.setProperty("top", sp.y - base.top + "px");
}

function clearPointing() {
  for (const id in nodeEls) {
    nodeEls[id].classList.remove("candidate");
  }
}

function closeChooser() {
  el("chooser").hidden = true;
  el("chooser-body").replaceChildren();
  el("map-hint").hidden = true;
  clearPointing();
  pointing = null;
}

function onNodeClick(i, id) {
  if (pointing && pointing.byNode[id] !== undefined) {
    choose(pointing.ei, pointing.byNode[id]);
    return;
  }
  recordInspect(id);
  inspectNode(i);
}

// The edit layer is an SVG group, where the `hidden` IDL property does not
// reflect to the attribute, so toggle the attribute the [hidden] rule matches.
function badges(on) {
  const g = el("edit-layer");
  if (!g) {
    return;
  }
  if (on) {
    g.removeAttribute("hidden");
  } else {
    g.setAttribute("hidden", "hidden");
  }
}

function togglePanel() {
  const box = el("l-edits");
  box.hidden = !box.hidden;
  const shown = !box.hidden;
  el("panel-toggle").setAttribute("aria-expanded", shown ? "true" : "false");
  el("panel-toggle").textContent = shown ? t("panel_hide") : t("panel_show");
}

function choose(edit, option) {
  const view = call("choose", edit, option);
  if (!view) {
    return;
  }
  closeChooser();
  cur.view = view;
  cur.ran = false;
  cur.result = null;
  renderLevel();
}

function renderMap(v) {
  const map = el("map");
  map.setAttribute("viewBox", v.viewbox);
  clear(map);
  for (const id in nodeEls) {
    delete nodeEls[id];
  }
  // Cables are drawn as a quadratic curve through the control point the core
  // returns. A lone cable's control point is its midpoint, so the curve is a
  // straight line; backup cables that share two ends bow to opposite sides.
  const cables = svg("g", {});
  for (const k of v.links) {
    let cls = "cable";
    if (k.down) {
      cls += " down";
    }
    if (k.blocked) {
      cls += " blocked";
    }
    if (k.lossy) {
      cls += " lossy";
    }
    const d = "M " + k.x1 + " " + k.y1 + " Q " + k.cpx + " " + k.cpy + " " + k.x2 + " " + k.y2;
    cables.appendChild(svg("path", { class: cls, d: d }));
  }
  map.appendChild(cables);

  const nodes = svg("g", {});
  for (const n of v.nodes) {
    const g = svg("g", { class: "node-hit" });
    const img = svg("image", { x: n.x, y: n.y, width: n.size, height: n.size, "aria-label": n.kind_name });
    img.setAttribute("href", "art/icons/" + n.kind + ".png");
    g.appendChild(img);
    g.appendChild(svg("text", { class: "node-label", x: n.cx, y: n.ly }));
    g.lastChild.textContent = n.name;
    if (n.addr) {
      g.appendChild(svg("text", { class: "node-addr", x: n.cx, y: n.ly2 }));
      g.lastChild.textContent = n.addr;
    }
    g.addEventListener("click", () => onNodeClick(n.i, n.id));
    nodeEls[n.id] = g;
    nodes.appendChild(g);
  }
  map.appendChild(nodes);

  map.appendChild(svg("g", { id: "packets" }));
  map.appendChild(svg("g", { id: "effects" }));

  // One edit badge per spot on the map. Edits that share a spot (the two sides
  // of a handshake, a transfer's window and timeout) group under one badge.
  const editLayer = svg("g", { id: "edit-layer" });
  const seen = {};
  v.edits.forEach((e, ei) => {
    const key = e.ax + "," + e.ay;
    if (seen[key]) {
      seen[key].push(ei);
      return;
    }
    const group = [ei];
    seen[key] = group;
    const g = svg("g", { class: "edit-badge" });
    g.setAttribute("tabindex", "0");
    g.setAttribute("role", "button");
    g.setAttribute("aria-label", t("edit_here"));
    g.appendChild(svg("use", { href: "#edit-mark", x: e.ax, y: e.ay }));
    g.addEventListener("click", () => openChooser(group));
    g.addEventListener("keydown", (ev) => {
      if (ev.key === "Enter" || ev.key === " ") {
        ev.preventDefault();
        openChooser(group);
      }
    });
    editLayer.appendChild(g);
  });
  map.appendChild(editLayer);
}

function renderLegend(v) {
  const box = el("legend");
  clear(box);
  for (const p of v.legend) {
    const item = tag("span", "item");
    const chip = svg("svg", { class: "chip-svg", role: "img", "aria-label": p.name });
    chip.appendChild(svg("use", { href: "#pkt-sym-" + p.cls }));
    item.appendChild(chip);
    item.appendChild(tag("span", null, p.name));
    box.appendChild(item);
  }
}

// ---------- run and playback ----------

function runLevel() {
  const result = call("run");
  if (!result) {
    return;
  }
  cur.result = result;
  cur.ran = true;
  cur.ms = 0;
  cur.logCount = 0;
  cur.speedIdx = display.default_speed;
  clearLogRows();
  el("hud").hidden = false;
  el("controls").hidden = false;
  el("run-btn").textContent = t(cur.view.predict ? "btn_commit" : "btn_rerun");
  setPlay(true);
}

// Only ever one animation loop is in flight: a stored handle lets a new run,
// a replay or a play/pause cancel the previous one instead of stacking loops.
function stopLoop() {
  if (rafId) {
    cancelAnimationFrame(rafId);
    rafId = 0;
  }
}

function setPlay(on) {
  cur.playing = on;
  el("play-btn").textContent = t(on ? "btn_pause" : "btn_play");
  stopLoop();
  badges(!on);
  if (on) {
    closeChooser();
    cur.last = performance.now();
    rafId = requestAnimationFrame(loop);
  }
}

function loop(now) {
  rafId = 0;
  if (!cur.playing) {
    return;
  }
  let dt = now - cur.last;
  cur.last = now;
  // Keep a frame's advance within sane bounds. A frame's timestamp can precede
  // the performance.now() captured when playback started (a real timing quirk,
  // more likely on a re-run), which would make dt negative; left alone, the
  // negative time became a huge unsigned value in the core and the run jumped
  // straight to its end. And a backgrounded tab, a lost focus or a hitch can
  // hand back one enormous gap. So never go backwards, and never cross more
  // than one tick in a frame; the cap comes from the core, so no number lives
  // here. Playback then resumes from where it paused instead of skipping.
  if (dt < 0) {
    dt = 0;
  }
  if (dt > mpt()) {
    dt = mpt();
  }
  cur.ms += dt;
  drawFrame();
}

function drawFrame() {
  const f = call("frame", cur.ms, mpt(), cur.logCount);
  if (!f) {
    return;
  }
  renderPackets(f);
  appendLog(f);
  el("hud").textContent = f.hud;
  if (f.done) {
    cur.playing = false;
    el("play-btn").textContent = t("btn_play");
    badges(true);
    showResults();
  } else if (cur.playing) {
    rafId = requestAnimationFrame(loop);
  }
}

function renderPackets(f) {
  const pk = el("packets");
  clear(pk);
  for (const p of f.packets) {
    const g = svg("g", { class: "pkt " + p.cls, transform: "translate(" + p.x + "," + p.y + ")" });
    g.appendChild(svg("use", { href: "#pkt-" + p.cls }));
    g.addEventListener("click", () => inspectPacket(p.id));
    pk.appendChild(g);
  }
  const fx = el("effects");
  clear(fx);
  for (const e of f.effects) {
    const g = svg("g", { class: "fx " + e.fx, transform: "translate(" + e.x + "," + e.y + ")" });
    const txt = svg("text", { x: 0, y: 0 });
    txt.textContent = e.symbol;
    g.appendChild(txt);
    fx.appendChild(g);
  }
}

// The log keeps every line of the run so it can be searched and filtered
// without replaying. `off*` hold the categories and machines the student has
// switched off (empty means everything shows).
const logRows = [];
const logView = { search: "", offKinds: new Set(), offMachines: new Set() };

function resetLog() {
  logRows.length = 0;
  logView.search = "";
  logView.offKinds.clear();
  logView.offMachines.clear();
  el("log-search").value = "";
  el("log-panel").classList.remove("full", "collapsed");
  el("log-full-btn").textContent = t("log_full");
  el("log-toggle-btn").textContent = t("log_hide");
  el("log-toggle-btn").setAttribute("aria-expanded", "true");
  renderLogChips();
  renderLogRows();
}

function appendLog(f) {
  if (f.log_count === cur.logCount) {
    return;
  }
  for (const line of f.log) {
    logRows.push(line);
  }
  cur.logCount = f.log_count;
  renderLogChips();
  renderLogRows();
  const body = el("log-body");
  body.scrollTop = body.scrollHeight;
}

// A row shown or hidden by the current search and filter chips.
function logShown(line) {
  if (logView.offKinds.has(line.kind) || logView.offMachines.has(line.node)) {
    return false;
  }
  const q = logView.search.trim().toLowerCase();
  if (!q) {
    return true;
  }
  return (line.t + " " + line.node + " " + line.text).toLowerCase().includes(q);
}

function distinct(key) {
  const seen = [];
  for (const line of logRows) {
    if (!seen.includes(line[key])) {
      seen.push(line[key]);
    }
  }
  return seen;
}

// One filter chip: toggles a value into/out of an `off` set. A chip is pressed
// (on) unless its value sits in the off set.
function filterChip(label, sym, off, value, symClass) {
  const on = !off.has(value);
  const b = tag("button", "log-chip");
  b.type = "button";
  b.setAttribute("aria-pressed", on ? "true" : "false");
  if (sym) {
    b.appendChild(tag("span", "chip-sym" + (symClass ? " " + symClass : ""), sym));
  }
  b.appendChild(tag("span", null, label));
  b.addEventListener("click", () => {
    if (off.has(value)) {
      off.delete(value);
    } else {
      off.add(value);
    }
    renderLogChips();
    renderLogRows();
  });
  return b;
}

function renderLogChips() {
  const kinds = distinct("kind");
  const machines = distinct("node");
  const kbox = el("log-kinds");
  clear(kbox);
  for (const k of kinds) {
    kbox.appendChild(filterChip(t("logkind_" + k), t("logsym_" + k), logView.offKinds, k, k));
  }
  const mbox = el("log-machines");
  clear(mbox);
  for (const m of machines) {
    mbox.appendChild(filterChip(m, "", logView.offMachines, m));
  }
}

function renderLogRows() {
  const log = el("log");
  clear(log);
  let shown = 0;
  for (const line of logRows) {
    if (!logShown(line)) {
      continue;
    }
    shown += 1;
    const li = tag("li");
    li.appendChild(tag("span", "l-t", line.t));
    const tagEl = tag("span", "l-tag " + line.kind);
    tagEl.appendChild(tag("span", "chip-sym", t("logsym_" + line.kind)));
    tagEl.appendChild(tag("span", null, t("logkind_" + line.kind)));
    li.appendChild(tagEl);
    li.appendChild(tag("span", "l-machine", line.node));
    li.appendChild(tag("span", "l-text", line.text));
    log.appendChild(li);
  }
  const note = el("log-note");
  if (!logRows.length) {
    note.hidden = false;
    note.textContent = t("log_empty");
  } else if (!shown) {
    note.hidden = false;
    note.textContent = t("log_none_match");
  } else {
    note.hidden = true;
  }
}

function toggleLogFull() {
  const full = el("log-panel").classList.toggle("full");
  if (full) {
    el("log-panel").classList.remove("collapsed");
    el("log-toggle-btn").textContent = t("log_hide");
    el("log-toggle-btn").setAttribute("aria-expanded", "true");
  }
  el("log-full-btn").textContent = t(full ? "log_exit_full" : "log_full");
}

function toggleLogCollapse() {
  const hidden = el("log-panel").classList.toggle("collapsed");
  el("log-toggle-btn").textContent = t(hidden ? "log_show" : "log_hide");
  el("log-toggle-btn").setAttribute("aria-expanded", hidden ? "false" : "true");
}

function skip() {
  stopLoop();
  cur.ms = (cur.result.ticks + 1) * mpt();
  cur.playing = false;
  badges(true);
  drawFrame();
}

function replay() {
  cur.ms = 0;
  cur.logCount = 0;
  clearLogRows();
  clear(el("packets"));
  setPlay(true);
}

function cycleSpeed() {
  const old = mpt();
  let i = cur.speedIdx + 1;
  if (i > display.speeds.length - 1) {
    i = 0;
  }
  cur.speedIdx = i;
  cur.ms = (cur.ms * mpt()) / old;
  el("speed-btn").textContent = display.speeds[i].label;
}

// ---------- overlays ----------

function openOverlay() {
  el("overlay").hidden = false;
}

function closeOverlay() {
  el("overlay").hidden = true;
}

function starsRow(stars) {
  const row = tag("div", "stars-big");
  for (const got of stars) {
    row.appendChild(tag("span", got ? "full" : "empty", got ? "★" : "☆"));
  }
  return row;
}

// A demonstration scene's result: no stars, the prediction outcome and a
// Continue button that marks the scene watched and goes on to the level.
function showSceneResult(r, body) {
  body.appendChild(tag("h2", null, cur.view.title));
  if (r.predict) {
    body.appendChild(
      tag("p", r.predict.right ? "met" : "unmet", t(r.predict.right ? "predict_right" : "predict_wrong")),
    );
  }
  if (r.debrief) {
    body.appendChild(tag("p", "debrief", r.debrief));
  }
  const actions = tag("div", "sheet-actions");
  const cont = tag("button", "primary", t("btn_continue"));
  cont.type = "button";
  cont.addEventListener("click", () => {
    call("scene_done", cur.sceneChapter);
    store();
    closeOverlay();
    if (pendingLevel) {
      const lv = pendingLevel;
      pendingLevel = null;
      openLevel(lv);
    } else {
      renderMenu();
    }
  });
  actions.appendChild(cont);
  const again = tag("button", "ghost", t("btn_restart"));
  again.type = "button";
  again.addEventListener("click", closeOverlay);
  actions.appendChild(again);
  body.appendChild(actions);
  openOverlay();
}

function showResults() {
  store();
  const r = cur.result;
  const body = el("overlay-body");
  clear(body);
  if (cur.view.scene) {
    showSceneResult(r, body);
    return;
  }
  body.appendChild(tag("h2", null, r.headline));
  body.appendChild(starsRow(r.stars));
  body.appendChild(tag("p", null, r.stars_text));

  const goals = tag("ul", "result-goals");
  for (const g of r.goals) {
    goals.appendChild(tag("li", g.met ? "met" : "unmet", (g.met ? "✓ " : "✕ ") + g.text));
  }
  goals.appendChild(tag("li", r.bonus.met ? "met" : "unmet", (r.bonus.met ? "★ " : "☆ ") + r.bonus.text));
  body.appendChild(goals);

  const sum = tag("ul", "summary");
  for (const s of r.summary) {
    sum.appendChild(tag("li", null, s));
  }
  body.appendChild(sum);

  if (!r.passed) {
    body.appendChild(tag("h3", null, t("diagnosis_title")));
    const d = tag("div", "diagnosis");
    for (const line of r.diagnosis) {
      d.appendChild(tag("p", null, line));
    }
    body.appendChild(d);
  }

  // Evidence-first result: whether the committed prediction matched the run.
  if (r.predict) {
    body.appendChild(
      tag("p", r.predict.right ? "met" : "unmet", t(r.predict.right ? "predict_right" : "predict_wrong")),
    );
  }

  if (r.passed && r.debrief) {
    body.appendChild(tag("p", "debrief", r.debrief));
  }

  // "What this level teaches" is revealed here, once the level is passed.
  if (r.passed && r.concept && r.concept.length) {
    body.appendChild(tag("h3", null, t("panel_concept")));
    const cl = tag("ul", "result-concept");
    for (const c of r.concept) {
      cl.appendChild(tag("li", null, c));
    }
    body.appendChild(cl);
  }

  const actions = tag("div", "sheet-actions");
  if (r.passed && r.next) {
    const nb = tag("button", "primary", t("btn_next"));
    nb.type = "button";
    nb.addEventListener("click", () => {
      closeOverlay();
      openLevel(r.next);
    });
    actions.appendChild(nb);
  }
  if (r.can_practice) {
    const pb = tag("button", "ghost", t("btn_practice"));
    pb.type = "button";
    pb.addEventListener("click", () => {
      const view = call("practice");
      if (!view) {
        return;
      }
      closeOverlay();
      cur.view = view;
      cur.ran = false;
      cur.result = null;
      renderLevel();
    });
    actions.appendChild(pb);
  }
  if (r.passed && r.quiz) {
    const qb = tag("button", "ghost", t("quiz_q"));
    qb.type = "button";
    qb.addEventListener("click", () => showQuiz(r.quiz));
    actions.appendChild(qb);
  }
  const retry = tag("button", "ghost", t("btn_retry"));
  retry.type = "button";
  retry.addEventListener("click", closeOverlay);
  actions.appendChild(retry);
  body.appendChild(actions);
  openOverlay();
}

function showQuiz(quiz) {
  const body = el("overlay-body");
  clear(body);
  body.appendChild(tag("h2", null, quiz.question));
  const opts = tag("div", "quiz-opts");
  quiz.options.forEach((label, i) => {
    const b = tag("button", "quiz-opt", label);
    b.type = "button";
    b.addEventListener("click", () => answerQuiz(i, opts));
    opts.appendChild(b);
  });
  body.appendChild(opts);
  openOverlay();
}

function answerQuiz(option, opts) {
  const res = call("answer", option);
  if (!res) {
    return;
  }
  store();
  const buttons = opts.querySelectorAll(".quiz-opt");
  buttons[option].className = "quiz-opt " + (res.correct ? "right" : "wrong");
  for (const b of buttons) {
    b.disabled = true;
  }
  const body = el("overlay-body");
  body.appendChild(tag("p", null, res.headline));
  if (res.why) {
    body.appendChild(tag("p", "debrief", res.why));
  }
  body.appendChild(starsRow(res.stars));
  const actions = tag("div", "sheet-actions");
  const done = tag("button", "primary", t("btn_close"));
  done.type = "button";
  done.addEventListener("click", () => {
    closeOverlay();
    cur.view = call("open", cur.view.id) || cur.view;
    renderLevel();
  });
  actions.appendChild(done);
  body.appendChild(actions);
}

function inspectPacket(id) {
  const p = call("packet", id);
  if (!p) {
    return;
  }
  const body = el("overlay-body");
  clear(body);
  const head = tag("h2", null, "");
  const chip = svg("svg", { class: "chip-svg", role: "img", "aria-label": p.cls });
  chip.appendChild(svg("use", { href: "#pkt-sym-" + p.cls }));
  head.appendChild(chip);
  head.appendChild(document.createTextNode(" " + t("panel_legend")));
  body.appendChild(head);
  for (const layer of p.layers) {
    const wrap = tag("div", "layer");
    wrap.appendChild(tag("div", "lname", layer.name));
    const table = tag("table");
    for (const row of layer.rows) {
      const tr = tag("tr");
      tr.appendChild(tag("td", "k", row[0]));
      tr.appendChild(tag("td", "v", row[1]));
      table.appendChild(tr);
    }
    wrap.appendChild(table);
    body.appendChild(wrap);
  }
  openOverlay();
}

function inspectNode(i) {
  const n = call("node", i, cur.ms, mpt());
  if (!n) {
    return;
  }
  const body = el("overlay-body");
  clear(body);
  body.appendChild(tag("h2", null, n.name));
  body.appendChild(tag("p", "eyebrow", n.kind_name));
  body.appendChild(tag("p", null, n.blurb));
  body.appendChild(tag("h3", null, t("panel_node")));
  // Facts (IP, MAC, gateway…) as a key/value grid.
  if (n.facts.length) {
    const table = tag("table", "node-facts");
    for (const row of n.facts) {
      const tr = tag("tr");
      tr.appendChild(tag("td", "k", row[0]));
      tr.appendChild(tag("td", "v", row[1]));
      table.appendChild(tr);
    }
    body.appendChild(table);
  }
  // Titled tables (a switch's MAC table, a router's routes) as styled grids.
  for (const tbl of n.tables) {
    const wrap = tag("div", "node-table");
    wrap.appendChild(tag("div", "nt-title", tbl.title));
    if (tbl.rows.length) {
      const table = tag("table");
      const thead = tag("thead");
      const htr = tag("tr");
      for (const col of tbl.cols) {
        htr.appendChild(tag("th", null, col));
      }
      thead.appendChild(htr);
      table.appendChild(thead);
      const tbody = tag("tbody");
      for (const row of tbl.rows) {
        const tr = tag("tr");
        for (const cell of row) {
          tr.appendChild(tag("td", null, cell));
        }
        tbody.appendChild(tr);
      }
      table.appendChild(tbody);
      wrap.appendChild(table);
    } else if (tbl.empty) {
      wrap.appendChild(tag("p", "nt-empty", tbl.empty));
    }
    body.appendChild(wrap);
  }
  openOverlay();
}

function showHow() {
  const body = el("overlay-body");
  clear(body);
  body.appendChild(tag("h2", null, t("how_title")));
  for (const k of ["how_1", "how_2", "how_3", "how_4", "how_5"]) {
    body.appendChild(tag("p", null, t(k)));
  }
  openOverlay();
}

// ---------- wiring ----------

function wire() {
  el("home-btn").addEventListener("click", renderMenu);
  el("how-btn").addEventListener("click", showHow);
  el("overlay-close").addEventListener("click", closeOverlay);
  el("overlay").addEventListener("click", (e) => {
    if (e.target === el("overlay")) {
      closeOverlay();
    }
  });
  el("run-btn").addEventListener("click", runLevel);
  el("panel-toggle").addEventListener("click", togglePanel);
  el("chooser-close").addEventListener("click", closeChooser);
  document.addEventListener("keydown", (ev) => {
    if (ev.key === "Escape") {
      closeChooser();
    }
  });
  document.addEventListener("click", (ev) => {
    if (el("chooser").hidden && !pointing) {
      return;
    }
    if (ev.target.closest("#chooser") || ev.target.closest(".edit-badge") || ev.target.closest(".node-hit")) {
      return;
    }
    closeChooser();
  });
  el("play-btn").addEventListener("click", () => setPlay(!cur.playing));
  el("skip-btn").addEventListener("click", skip);
  el("replay-btn").addEventListener("click", replay);
  el("speed-btn").addEventListener("click", cycleSpeed);
  el("log-search").addEventListener("input", (e) => {
    logView.search = e.target.value;
    renderLogRows();
  });
  el("log-full-btn").addEventListener("click", toggleLogFull);
  el("log-toggle-btn").addEventListener("click", toggleLogCollapse);
  el("zoom").addEventListener("input", applyZoom);
  el("zoom-in").addEventListener("click", () => {
    el("zoom").stepUp();
    applyZoom();
  });
  el("zoom-out").addEventListener("click", () => {
    el("zoom").stepDown();
    applyZoom();
  });
  el("zoom-reset").addEventListener("click", () => {
    el("zoom").value = el("zoom").defaultValue;
    applyZoom();
  });
  el("teacher").addEventListener("change", (e) => {
    call("set_teacher", e.target.checked);
    store();
    renderMenu();
  });
  el("reset-btn").addEventListener("click", () => {
    if (window.confirm(t("menu_reset_confirm"))) {
      call("reset");
      store();
      renderMenu();
    }
  });
}

function labels() {
  el("brand").textContent = t("title");
  el("how-btn").textContent = t("menu_how");
  el("home-btn").textContent = "← " + t("menu_play");
  el("menu-intro").textContent = t("tagline");
  el("reset-btn").textContent = t("menu_reset");
  el("play-btn").textContent = t("btn_play");
  el("skip-btn").textContent = t("btn_skip");
  el("replay-btn").textContent = t("btn_restart");
  el("speed-btn").textContent = display.speeds[display.default_speed].label;
  el("log-search").setAttribute("placeholder", t("log_search_ph"));
  el("log-search").setAttribute("aria-label", t("log_search"));
  el("log-by-action").textContent = t("log_by_action");
  el("log-by-machine").textContent = t("log_by_machine");
  el("log-full-btn").textContent = t("log_full");
  el("log-toggle-btn").textContent = t("log_hide");
  el("zoom-h").textContent = t("zoom_h");
  el("zoom-reset").textContent = t("zoom_reset");
  el("zoom-in").setAttribute("aria-label", t("zoom_in"));
  el("zoom-out").setAttribute("aria-label", t("zoom_out"));
  el("zoom").setAttribute("aria-label", t("zoom_label"));
}

function preloadArt() {
  const kinds = ["host", "printer", "server", "hub", "switch", "router"];
  const backs = ["block", "town", "wires"];
  for (const k of kinds) {
    const img = new Image();
    img.src = "art/icons/" + k + ".png";
  }
  for (const b of backs) {
    const img = new Image();
    img.src = "art/backdrops/" + b + ".png";
  }
}

async function start() {
  try {
    await init();
    const [levels, copyText, displayText] = await Promise.all([
      fetch("data/levels.json").then((r) => r.text()),
      fetch("data/copy.json").then((r) => r.text()),
      fetch("data/display.json").then((r) => r.text()),
    ]);
    copy = JSON.parse(copyText);
    display = JSON.parse(displayText);
    cur.speedIdx = display.default_speed;
    game = new Game(levels, copyText, loadSave());
    labels();
    wire();
    renderMenu();
    preloadArt();
  } catch (e) {
    el("message").textContent = (copy && copy.err_load) || "The game could not start.";
    document.body.dataset.ready = "true";
    return;
  }
  document.body.dataset.ready = "true";
}

start();
