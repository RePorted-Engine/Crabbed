#!/usr/bin/env python3
"""Compare fresh scene captures with the baseline: screenshot_diff.py BASELINE NEW [--tolerance PERCENT].

Prints a Markdown table (for the job summary) of each scene's mean pixel difference, writes NAME.diff.png beside
the new capture for every scene over the tolerance, and exits 1 when any is. Scenes without a baseline are listed,
not failed: a new scene gets its baseline by copying the capture into BASELINE. Needs Pillow.
"""
import argparse
import sys
from pathlib import Path

from PIL import Image, ImageChops


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("baseline")
    parser.add_argument("new")
    parser.add_argument("--tolerance", type=float, default=4.0,
                        help="mean difference per channel, in percent of 255, a scene may have (default 4)")
    args = parser.parse_args()
    baseline, new = Path(args.baseline), Path(args.new)
    print("## Screenshots against the baseline\n")
    if not baseline.is_dir():
        print(f"No baseline in `{baseline}` yet.")
        return 0
    print("| Scene | Difference | |\n|---|---|---|")
    over = False
    for shot in sorted(new.glob("*.png")):
        if shot.name == "sheet.png" or shot.name.endswith(".diff.png"):
            continue
        reference = baseline / shot.name
        if not reference.exists():
            print(f"| {shot.stem} | no baseline | |")
            continue
        a = Image.open(reference).convert("RGB")
        b = Image.open(shot).convert("RGB").resize(a.size)
        diff = ImageChops.difference(a, b)
        pixels = a.size[0] * a.size[1] * 3
        mean = sum(i * n for i, n in enumerate(diff.histogram())) / pixels / 255 * 100
        bad = mean > args.tolerance
        over |= bad
        if bad:
            diff.point(lambda v: min(255, v * 4)).save(shot.with_name(f"{shot.stem}.diff.png"))
        print(f"| {shot.stem} | {mean:.2f}% | {'over' if bad else 'ok'} |")
    return 1 if over else 0


if __name__ == "__main__":
    sys.exit(main())
