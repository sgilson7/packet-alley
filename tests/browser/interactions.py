"""Packet Alley's own interactions for tests/browser/check.py.

A new player's first minutes: the menu, the first level open on the map,
picking the right address, running it, skipping to the result, passing, and
answering the quiz; then a reload that keeps the star. Selectors and waits
only — Firefox's CSP blocks Playwright's string-form wait_for_function.

Expected values are worked out from the briefs by hand (a new player sees
four chapters, the first level unlocked and the rest locked, the first
level has a switch and a printer), not read from the game's own output.
"""
import json
import pathlib

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


def run(page, check, engine):
    page.wait_for_selector("body[data-mode='menu']", state="attached", timeout=10000)
    check(page.inner_text("#brand") == COPY["title"], "the title is drawn")
    check(page.locator("#chapters .chapter").count() == 4, "four chapters are shown")
    check(page.locator(".level-card").count() == 19, "nineteen levels are listed")
    check(page.locator(".level-card.locked").count() == 18, "all but the first level start locked")
    check(images_loaded(page, "#menu"), "the menu art loaded")

    # open the first level
    page.locator(".level-card:not(.locked)").first.click()
    page.wait_for_selector("body[data-mode='level']", state="attached", timeout=5000)
    check(page.locator("#map image").count() == 3, "the first level's map has three devices")
    check(page.locator("#l-edits .edit").count() == 1, "it offers one change")
    check(page.locator("#legend .item").count() >= 1, "the packet legend is shown")
    check(images_loaded(page, "#map-wrap"), "the backdrop and device icons loaded")

    # inspect a device
    page.locator("#map .node-hit").first.click()
    page.wait_for_selector("#overlay:not([hidden])", timeout=5000)
    check(page.locator("#overlay-body h2").count() == 1, "clicking a device opens its details")
    page.click("#overlay-close")
    page.wait_for_selector("#overlay[hidden]", state="attached", timeout=5000)

    # pick the printer's address (the second option) and run
    page.locator("#l-edits .opt").nth(1).click()
    check("chosen" in (page.locator("#l-edits .opt").nth(1).get_attribute("class") or ""), "the choice is marked")
    page.click("#run-btn")
    check(page.locator("#controls").is_visible(), "the playback controls appear")
    # The run plays itself; skip to the end if the controls are still up, then
    # wait for the result. (Clicking skip is racy — a short run can finish and
    # open the result overlay first — so the skip is best-effort.)
    try:
        page.click("#skip-btn", timeout=1500)
    except Exception:
        pass
    page.wait_for_selector("#overlay:not([hidden])", timeout=20000)
    check(page.inner_text("#overlay-body h2") == COPY["passed"], "the right address delivers the order")

    # answer the quiz (option 0 is correct for this level)
    page.locator(".sheet-actions .primary").first.click()
    page.wait_for_selector(".quiz-opt", timeout=5000)
    page.locator(".quiz-opt").first.click()
    check(page.locator(".quiz-opt.right").count() == 1, "the right quiz answer is accepted")
    page.locator(".sheet-actions .primary").first.click()
    page.wait_for_selector("#overlay[hidden]", state="attached", timeout=5000)

    # a reload keeps the stars
    page.reload()
    page.wait_for_selector("body[data-mode='menu']", state="attached", timeout=10000)
    check(page.locator(".level-card .stars .full").count() >= 2, "the earned stars survive a reload")
    check(page.locator(".level-card.locked").count() == 17, "passing the first level unlocked the second")
