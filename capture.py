"""Renode capture sequence for Othello (not yet run).

Run from xous-dev-toolkit:
    python3 scripts/renode_capture.py --app othello --script ../precursor-othello/capture.py

Contract: run(ctl, outdir) -> number of failed steps. The app is already launched.
"""
import os


def run(ctl, outdir) -> int:
    failures = 0

    def shot(name):
        nonlocal failures
        if not ctl.try_screenshot(os.path.join(outdir, name)):
            failures += 1

    shot("01_main.png")
    ctl.inject_line("")            # Enter: New game
    ctl.sleep(1.0)
    shot("02_levels.png")
    ctl.inject_key("5")            # two players
    ctl.sleep(1.0)
    shot("03_start.png")
    ctl.inject_line("")            # Enter: place at the cursor (D4, occupied: "Not a legal move")
    ctl.sleep(1.0)
    shot("04_illegal.png")
    ctl.timed_key("Up", after=1.0)  # D3 is a legal opening move
    ctl.inject_line("")
    ctl.sleep(1.0)
    shot("05_first_move.png")
    ctl.timed_key("F1", after=1.0)  # game menu
    shot("06_game_menu.png")
    ctl.timed_key("F4", after=1.0)  # back to the board
    ctl.timed_key("F4", after=1.0)  # leave game (saved) to the main menu
    shot("07_main_with_resume.png")
    ctl.timed_key("F4", after=1.0)  # leave the app
    return failures
