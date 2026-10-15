"""Packet Alley's own interactions for tests/browser/check.py.

A new player's first minutes: the menu; the first chapter's demonstration
scene (watch-first — inspect, predict, run, continue); then the first level on
the map, inspecting a device, committing a prediction, pointing at the printer,
running, passing; and a reload that keeps the star. Selectors and waits only —
Firefox's CSP blocks Playwright's string-form wait_for_function.

Expected values are worked out from the briefs by hand (a new player sees four
chapters, the first level unlocked and the rest locked), not read from the
game's own output.
"""
import json
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[2]
COPY = json.loads((ROOT / "data" / "copy.json").read_text())


def images_loaded(page, scope):
    for _ in range(50):
        done = page.evaluate(
            "(sel) => [...document.querySelectorAll(sel + ' img, ' + sel + ' image')]"
            ".every(i => { const u = i.getAttribute('href') || i.getAttribute('src');"
            " return !u || (i.complete !== false && (i.naturalWidth === undefined || i.naturalWidth > 0)); })",
            scope,
        )
        if done:
            return True
        page.wait_for_timeout(200)
    return False


def watch_and_run(page):
    """Inspect a glowing device, commit a prediction, run, skip to the result."""
    page.locator("#map .node-hit.evidence").first.click()
    page.wait_for_selector("#overlay:not([hidden])", timeout=5000)
    page.click("#overlay-close")
    page.wait_for_selector("#overlay[hidden]", state="attached", timeout=5000)
    page.locator("#predict-opts .opt").first.click()
    page.click("#run-btn")
    # best-effort skip; a short run can open the result overlay first
    try:
        page.click("#skip-btn", timeout=1500)
    except Exception:
        pass
    page.wait_for_selector("#overlay:not([hidden])", timeout=20000)


def run(page, check, engine):
    page.wait_for_selector("body[data-mode='menu']", state="attached", timeout=10000)
    check(page.inner_text("#brand") == COPY["title"], "the title is drawn")
    check(page.locator("#chapters .chapter").count() == 4, "four chapters are shown")
    check(page.locator(".level-card").count() == 19, "nineteen levels are listed")
    check(page.locator(".level-card.locked").count() == 18, "all but the first level start locked")
    check(images_loaded(page, "#menu"), "the menu art loaded")

    # Opening the first level shows the chapter's demonstration scene first.
    page.locator(".level-card:not(.locked)").first.click()
    page.wait_for_selector("body[data-mode='level']", state="attached", timeout=5000)
    # the eyebrow is CSS-uppercased, so compare case-insensitively
    check(COPY["scene_tag"].lower() in page.inner_text("#l-chapter").lower(), "the chapter's demo scene appears first")
    check(images_loaded(page, "#map-wrap"), "the backdrop and device icons loaded")
    check(page.locator("#l-edits").is_hidden(), "a scene offers no edits")
    # watch the scene: inspect, predict, run, then continue to the chapter
    watch_and_run(page)
    page.locator(".sheet-actions .primary").first.click()
    page.wait_for_selector("#overlay[hidden]", state="attached", timeout=5000)

    # now the chapter's first scored level
    check(page.locator("#map image").count() == 3, "the first level's map has three devices")
    check(page.locator("#l-edits .edit").count() == 1, "it offers one change")
    check(page.locator("#legend .item").count() >= 1, "the packet legend is shown")
    check(page.locator("#l-textbook:not([hidden])").count() == 1, "the textbook excerpt is shown")
    check("CC BY-SA" in page.inner_text("#l-textbook-src"), "with its CC BY-SA attribution")

    # inspect a device — on this evidence-first level, opening a glowing device
    # also records it as evidence for the prediction.
    page.locator("#map .node-hit").first.click()
    page.wait_for_selector("#overlay:not([hidden])", timeout=5000)
    check(page.locator("#overlay-body h2").count() == 1, "clicking a device opens its details")
    page.click("#overlay-close")
    page.wait_for_selector("#overlay[hidden]", state="attached", timeout=5000)

    # the prediction step appears; commit the correct outcome (the printer)
    check(page.locator("#l-predict:not([hidden])").count() == 1, "the prediction step is shown")
    page.locator("#predict-opts .opt").filter(has_text=re.compile("printer", re.I)).first.click()
    check("chosen" in (page.locator("#predict-opts .opt").first.get_attribute("class") or ""), "the prediction is recorded")

    # set the destination on the map: open the edit badge, which lights up the
    # machines you can send to, then point at the printer.
    check(page.locator("#edit-layer .edit-badge").count() == 1, "the map shows an edit badge")
    page.locator("#edit-layer .edit-badge").first.click()
    page.wait_for_selector("#map-hint:not([hidden])", timeout=5000)
    check(page.locator("#map .node-hit.candidate").count() >= 2, "the machines you can send to light up")
    page.locator("#map .node-hit.candidate").filter(has_text=re.compile("printer", re.I)).first.click()
    check("chosen" in (page.locator("#l-edits .opt").nth(1).get_attribute("class") or ""), "pointing at the printer sets the address")

    # commit and run — the button is never greyed out (it always does something)
    check(page.locator("#run-btn[disabled]").count() == 0, "the commit button is never disabled")
    page.click("#run-btn")
    check(page.locator("#controls").is_visible(), "the playback controls appear")
    try:
        page.click("#skip-btn", timeout=1500)
    except Exception:
        pass
    page.wait_for_selector("#overlay:not([hidden])", timeout=20000)
    check(page.inner_text("#overlay-body h2") == COPY["passed"], "the right address delivers the order")
    check(COPY["predict_right"] in page.inner_text("#overlay-body"), "the prediction matched the run")
    check(page.locator(".sheet-actions .primary").filter(has_text=COPY["btn_next"]).count() == 1, "a next-level button is offered")
    check(COPY["panel_concept"] in page.inner_text("#overlay-body"), "the lesson is revealed after passing")

    # the final star is the prediction's; close the result (no quiz here)
    page.locator(".sheet-actions .ghost").last.click()
    page.wait_for_selector("#overlay[hidden]", state="attached", timeout=5000)

    # a reload keeps the stars
    page.reload()
    page.wait_for_selector("body[data-mode='menu']", state="attached", timeout=10000)
    check(page.locator(".level-card .stars .full").count() >= 2, "the earned stars survive a reload")
    check(page.locator(".level-card.locked").count() == 17, "passing the first level unlocked the second")

    build_flow(page, check)


def mapclick(page, mx, my):
    """Clicks the screen point for a map coordinate, via the SVG's own matrix
    (the viewBox is letterboxed, so a linear guess would miss)."""
    q = page.evaluate(
        "([mx, my]) => { const s = document.getElementById('map');"
        " const p = s.createSVGPoint(); p.x = mx; p.y = my;"
        " const r = p.matrixTransform(s.getScreenCTM()); return [r.x, r.y]; }",
        [mx, my],
    )
    page.mouse.click(q[0], q[1])


def machine_img(page, i):
    return page.locator(".build-machine").nth(i).locator("image")


def build_flow(page, check):
    """The end-of-zone build level: place a switch, wire the four machines,
    address the registers, fill the switch table, and open for business. The
    expected outcome (a passing, cheapest-few build earns the gold star) is
    worked out from the level by hand, not read from the game."""
    # Teacher mode unlocks the super challenge without clearing the whole zone.
    page.check("#teacher")
    page.wait_for_selector(".super-card", timeout=5000)
    check(page.locator(".super-card").count() == 1, "the zone offers one super challenge")
    page.locator(".super-card").first.click()
    page.wait_for_selector("body[data-mode='level']", state="attached", timeout=5000)
    check(page.locator("#l-build:not([hidden])").count() == 1, "the build panel is shown")
    check(images_loaded(page, "#map-wrap"), "the restaurant floor loaded")

    # A cable cannot cross the kitchen, so one switch can't reach everyone — the
    # student places two, one by each pair of counters, and joins them. (index 4
    # is the first placed switch, "Switch 1"; index 5 is "Switch 2".)
    page.locator(".palette-item").first.click()
    mapclick(page, 260, 150)
    page.locator(".palette-item").first.click()
    mapclick(page, 720, 150)
    page.wait_for_selector("#map .build-machine", timeout=5000)
    check(page.locator(".build-machine").count() == 6, "two switches join the four fixed machines")

    # draw the cables: m1 to the left registers and to m2; m2 to the server and bar
    page.locator(".build-tool").filter(has_text=re.compile("cable", re.I)).first.click()
    for a, b in [(4, 0), (4, 1), (4, 5), (5, 3), (5, 2)]:
        machine_img(page, a).click()
        machine_img(page, b).click()
    check(page.locator(".build-cable").count() == 5, "five cables were run around the kitchen")

    # leave drawing mode; give each register an address
    page.locator(".build-tool").filter(has_text=re.compile("Drawing", re.I)).first.click()
    for i, idx in [(0, 1), (1, 2), (2, 3)]:
        machine_img(page, i).click()
        page.wait_for_selector("#chooser:not([hidden])", timeout=5000)
        page.locator("#chooser .build-select").first.select_option(index=idx)
        page.keyboard.press("Escape")
        page.wait_for_selector("#chooser[hidden]", state="attached", timeout=5000)

    # fill each switch's table: a machine on this switch's own cables is reached
    # directly; anything on the far side goes across the link to the other switch.
    local = {4: ["Front register", "Patio register"], 5: ["Card-auth server", "Bar register"]}
    other = {4: "Switch 2", 5: "Switch 1"}
    for sw in (4, 5):
        machine_img(page, sw).click()
        page.wait_for_selector("#chooser .switch-table", timeout=5000)
        rows = page.locator("#chooser .switch-table .build-select")
        for i in range(rows.count()):
            name = page.locator("#chooser .switch-table td.st-dst").nth(i).inner_text()
            via = name if name in local[sw] else other[sw]
            rows.nth(i).select_option(label=via)
        page.keyboard.press("Escape")
        page.wait_for_selector("#chooser[hidden]", state="attached", timeout=5000)

    # open for business
    page.click("#run-btn")
    try:
        page.click("#skip-btn", timeout=4000)
    except Exception:
        pass
    page.wait_for_selector("#overlay:not([hidden])", timeout=20000)
    check(page.inner_text("#overlay-body h2") == COPY["passed"], "every register reaches the server")
    check(page.locator("#overlay-body .stars-big .gold").count() == 1, "a cheapest-few build earns the hidden gold star")
