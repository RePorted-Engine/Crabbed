#!/usr/bin/env python3
"""Fixed screenshot scenes of the game, for CI and for looking at by eye.

Starts the game offscreen through horizon-ctl (its own throwaway home folders), opens one world with a fixed
seed, and writes one PNG per scene to OUT. Every scene sets its own time, weather and camera so two runs of
the same build draw the same thing.

    python3 tools/ci/screenshots.py --game target/fast/horizon-client --ctl target/fast/horizon-ctl \
        --resources .local/horizon-client/26.3 --out screenshots
"""
import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import lab  # noqa: E402  (tools/lab.py)

SEED = 1234567
WORLD = "CiScenes"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--game", help="horizon-client to launch (not with --attach)")
    parser.add_argument("--ctl", required=True)
    parser.add_argument("--resources", help="prepared game files (not with --attach)")
    parser.add_argument("--attach", metavar="SOCKET",
                        help="drive a game already running with --control-socket SOCKET (an iOS Simulator's)")
    parser.add_argument("--cwd", help="where SOCKET is relative to, with --attach (the app's Documents)")
    parser.add_argument("--shot-cmd", help="take each picture with this command instead, {path} for the PNG "
                        "(xcrun simctl io booted screenshot {path})")
    parser.add_argument("--out", required=True)
    parser.add_argument("--only", nargs="*", help="scene names to capture (default: all)")
    args = parser.parse_args()
    args.ctl = str(Path(args.ctl).resolve())
    if not args.attach:
        if not args.game or not args.resources:
            parser.error("--game and --resources, or --attach")
        args.game = str(Path(args.game).resolve())

    out = Path(args.out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    scratch = Path(tempfile.mkdtemp(prefix="horizon-scenes-"))
    # Fresh folders for the game's home, the launcher's and XDG's, no audio: never a real player's files.
    env = lab.game_env(HORIZON_TELEMETRY_URL="off")
    socket = args.attach or str(scratch / "ctl.sock")
    workdir = args.cwd or env["HORIZON_HOME"]
    log = (out / "game.log").open("w")

    def ctl(*words: str, timeout: float = 120, check: bool = True) -> str:
        done = subprocess.run(
            [args.ctl, "--socket", str(socket), *words],
            env=env, cwd=workdir, capture_output=True, text=True, timeout=timeout,
        )
        log.write(f"$ {' '.join(words)}\n{done.stdout}{done.stderr}\n")
        log.flush()
        if check and done.returncode != 0:
            raise RuntimeError(f"horizon-ctl {' '.join(words)} failed: {done.stdout}{done.stderr}")
        return done.stdout

    def shot(name: str) -> None:
        # Two frames of settling: the screenshot is the next frame drawn.
        ctl("wait", "10", check=False)
        path = out / f"{name}.png"
        if args.shot_cmd:
            subprocess.run(args.shot_cmd.replace("{path}", str(path)), shell=True, check=True, timeout=60)
        else:
            ctl("screenshot", str(path))
        print(f"captured {name}", flush=True)

    def wanted(name: str) -> bool:
        return not args.only or name in args.only

    failures = []

    def scene(name, steps):
        if not wanted(name):
            return
        try:
            steps()
            shot(name)
        except Exception as error:  # one scene failing must not hide the others
            failures.append(f"{name}: {error}")
            print(f"FAILED {name}: {error}", flush=True)

    try:
        if args.attach:
            # The app was started by whoever attached: wait until it answers.
            deadline = time.time() + 300
            while time.time() < deadline:
                reply = ctl("state", timeout=30, check=False).strip()
                if reply and '"error"' not in reply:
                    break
                time.sleep(3)
        else:
            ctl("launch", "--offscreen", "--game", args.game, "--resources", str(Path(args.resources).resolve()),
                "--width", "1280", "--height", "720", timeout=300)
        scene("title", lambda: ctl("wait-for", "screen", "title", "--timeout", "120", check=False))

        def open_world():
            ctl("open", WORLD, str(SEED), "creative", timeout=300)
            ctl("wait-ready", "240", timeout=300)
            ctl("command", "gamerule send_command_feedback false")
            ctl("options-set", "chatVisibility", "2", check=False)
            ctl("command", "gamerule advance_time false")
            ctl("command", "gamerule advance_weather false")
            ctl("command", "gamerule spawn_mobs false")
            # No falling, no hand in the way: the landscape scenes are seen as a spectator.
            ctl("command", "gamemode spectator")

        open_world()

        def settle(x, y, z):
            # The server loads around a teleport over a few seconds; wait-ready can answer before that.
            ctl("tp", str(x), str(y), str(z))
            ctl("wait", "60")
            ctl("wait-ready", "120", timeout=150, check=False)
            ctl("wait", "60")

        def day():
            ctl("command", "time set 1000")
            ctl("command", "weather clear")
            settle(0, 110, 0)
            ctl("look", "45", "30")
        scene("spawn_day", day)

        def structure():
            # A village the seed generated, found as a player would find it (feedback on to read the answer).
            ctl("command", "gamerule send_command_feedback true")
            answer = json.loads(ctl("command", "locate structure #minecraft:village", timeout=180) or "{}")
            ctl("command", "gamerule send_command_feedback false")
            text = " ".join(answer.get("chat", []))
            found = re.search(r"\[(-?\d+), [^,]+, (-?\d+)\]", text)
            if not found:
                raise RuntimeError(f"no village found: {text}")
            x, z = int(found.group(1)), int(found.group(2))
            settle(x, 120, z - 40)
            ctl("look-at", str(x), "70", str(z))
        scene("structure", structure)

        def inventory():
            ctl("command", "gamemode creative")
            ctl("open-inventory")
        scene("inventory", inventory)
        ctl("key", "esc", "tap", check=False)

        def cave():
            ctl("command", "time set 6000")
            settle(8, -40, 8)
            ctl("command", "fill 2 -41 2 14 -34 14 air")
            ctl("command", "fill 2 -42 2 14 -42 14 stone")
            ctl("command", "setblock 8 -41 4 torch")
            ctl("command", "setblock 4 -41 12 torch")
            ctl("command", "setblock 12 -41 12 lava")
            ctl("look", "0", "20")
            ctl("wait", "40")
        scene("cave", cave)

        def night_rain():
            ctl("command", "gamemode spectator")
            ctl("command", "time set 18000")
            ctl("command", "weather rain")
            settle(0, 90, 0)
            ctl("look", "45", "10")
            ctl("wait", "100")
        scene("night_rain", night_rain)
    finally:
        try:
            ctl("quit", timeout=60, check=False)
        except Exception:
            pass
        log.close()

    (out / "scenes.json").write_text(json.dumps({"seed": SEED, "failures": failures}, indent=2))
    if failures:
        print("\n".join(failures), file=sys.stderr)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
