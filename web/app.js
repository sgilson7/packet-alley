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
        card.addEventListener("click", () => openLevel(lv.id));
      }
      grid.appendChild(card);
    }
    sec.appendChild(grid);
    box.appendChild(sec);
  }
  setMode("menu");
}

// ---------- level ----------

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
  el("l-chapter").textContent = v.chapter + " · " + v.layer;
  el("l-title").textContent = v.number + ": " + v.title;
  el("l-story").textContent = v.story;
  el("task-h").textContent = t("panel_task");
  el("l-task").textContent = v.task;
  el("concept-h").textContent = t("panel_concept");
  const cl = el("l-concept");
  clear(cl);
  for (const c of v.concept) {
    cl.appendChild(tag("li", null, c));
  }
  el("goals-h").textContent = t("panel_goals");
  const gl = el("l-goals");
  clear(gl);
  for (const g of v.goals) {
    gl.appendChild(tag("li", null, g));
  }
  el("l-bonus").textContent = "★ " + v.bonus;
  renderEdits(v);
  el("run-btn").textContent = t(cur.ran ? "btn_rerun" : "btn_run");

  el("backdrop").src = "art/backdrops/" + v.backdrop + ".png";
  renderMap(v);
  renderLegend(v);

  el("legend-h").textContent = t("panel_legend");
  el("log-h").textContent = t("panel_log");
  el("hud").hidden = true;
  el("controls").hidden = true;
  clear(el("log"));
}

function renderEdits(v) {
  const box = el("l-edits");
  clear(box);
  v.edits.forEach((e, ei) => {
    const wrap = tag("div", "edit");
    wrap.appendChild(tag("p", "prompt", e.prompt));
    const opts = tag("div", "opts");
    e.options.forEach((o, oi) => {
      const b = tag("button", "opt" + (o.chosen ? " chosen" : ""), o.label);
      b.type = "button";
      b.setAttribute("aria-pressed", o.chosen ? "true" : "false");
      b.addEventListener("click", () => choose(ei, oi));
      opts.appendChild(b);
    });
    wrap.appendChild(opts);
    box.appendChild(wrap);
  });
}

function choose(edit, option) {
  const view = call("choose", edit, option);
  if (!view) {
    return;
  }
  cur.view = view;
  cur.ran = false;
  cur.result = null;
  renderLevel();
}

function renderMap(v) {
  const map = el("map");
  map.setAttribute("viewBox", v.viewbox);
  clear(map);
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
    cables.appendChild(svg("line", { class: cls, x1: k.x1, y1: k.y1, x2: k.x2, y2: k.y2 }));
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
    g.addEventListener("click", () => inspectNode(n.i));
    nodes.appendChild(g);
  }
  map.appendChild(nodes);

  map.appendChild(svg("g", { id: "packets" }));
  map.appendChild(svg("g", { id: "effects" }));
}

function renderLegend(v) {
  const box = el("legend");
  clear(box);
  for (const p of v.legend) {
    const item = tag("span", "item");
    const chip = tag("span", "chip " + p.cls, p.letter);
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
  clear(el("log"));
  el("hud").hidden = false;
  el("controls").hidden = false;
  el("run-btn").textContent = t("btn_rerun");
  setPlay(true);
}

function setPlay(on) {
  cur.playing = on;
  el("play-btn").textContent = t(on ? "btn_pause" : "btn_play");
  if (on) {
    cur.last = performance.now();
    requestAnimationFrame(loop);
  }
}

function loop(now) {
  if (!cur.playing) {
    return;
  }
  const dt = now - cur.last;
  cur.last = now;
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
    showResults();
  } else if (cur.playing) {
    requestAnimationFrame(loop);
  }
}

function renderPackets(f) {
  const pk = el("packets");
  clear(pk);
  for (const p of f.packets) {
    const g = svg("g", { class: "pkt " + p.cls, transform: "translate(" + p.x + "," + p.y + ")" });
    g.appendChild(svg("circle", { cx: 0, cy: 0 }));
    const txt = svg("text", { x: 0, y: 0 });
    txt.textContent = p.letter;
    g.appendChild(txt);
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

function appendLog(f) {
  if (f.log_count === cur.logCount) {
    return;
  }
  const log = el("log");
  for (const line of f.log) {
    const li = tag("li", line.fx);
    li.appendChild(tag("span", "sym", line.symbol));
    li.appendChild(document.createTextNode(line.t + "  " + line.text));
    log.appendChild(li);
  }
  cur.logCount = f.log_count;
  log.scrollTop = log.scrollHeight;
}

function skip() {
  cur.ms = (cur.result.ticks + 1) * mpt();
  cur.playing = false;
  drawFrame();
}

function replay() {
  cur.ms = 0;
  cur.logCount = 0;
  clear(el("log"));
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

function showResults() {
  store();
  const r = cur.result;
  const body = el("overlay-body");
  clear(body);
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

  if (r.passed && r.debrief) {
    body.appendChild(tag("p", "debrief", r.debrief));
  }

  const actions = tag("div", "sheet-actions");
  if (r.passed && r.quiz) {
    const qb = tag("button", "primary", t("quiz_q"));
    qb.type = "button";
    qb.addEventListener("click", () => showQuiz(r.quiz));
    actions.appendChild(qb);
  }
  if (r.passed && r.next) {
    const nb = tag("button", "ghost", t("btn_next"));
    nb.type = "button";
    nb.addEventListener("click", () => {
      closeOverlay();
      openLevel(r.next);
    });
    actions.appendChild(nb);
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
  const chip = tag("span", "chip " + p.cls, p.letter);
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
  const ul = tag("ul");
  for (const line of n.lines) {
    ul.appendChild(tag("li", null, line));
  }
  body.appendChild(ul);
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
  el("play-btn").addEventListener("click", () => setPlay(!cur.playing));
  el("skip-btn").addEventListener("click", skip);
  el("replay-btn").addEventListener("click", replay);
  el("speed-btn").addEventListener("click", cycleSpeed);
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
