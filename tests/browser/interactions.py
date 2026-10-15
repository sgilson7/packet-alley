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

    # commit and run — the button was disabled until evidence and a prediction
    check(page.locator("#run-btn:not([disabled])").count() == 1, "commit is enabled once evidence and a prediction are in")
    page.click("#run-btn")
    check(page.locator("#controls").is_visible(), "the playback controls appear")
    try:
        page.click("#skip-btn", timeout=1500)
    except Exception:
        pass
    page.wait_for_selector("#overlay:not([hidden])", timeout=20000)
    check(page.inner_text("#overlay-body h2") == COPY["passed"], "the right address delivers the order")
    check(COPY["predict_right"] in page.inner_text("#overlay-body"), "the prediction matched the run")

    # the final star is the prediction's; close the result (no quiz here)
    page.locator(".sheet-actions .ghost").last.click()
    page.wait_for_selector("#overlay[hidden]", state="attached", timeout=5000)

    # a reload keeps the stars
    page.reload()
    page.wait_for_selector("body[data-mode='menu']", state="attached", timeout=10000)
    check(page.locator(".level-card .stars .full").count() >= 2, "the earned stars survive a reload")
    check(page.locator(".level-card.locked").count() == 17, "passing the first level unlocked the second")
