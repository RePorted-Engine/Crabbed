#!/usr/bin/env python3
"""Render invisible/visible equipment controls in a fresh, isolated local world.

Needs a built game/ctl, the player's prepared resources and Pillow. No downloads,
real saves, Microsoft sessions or public servers are used.
"""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile
import time

import lab


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--game', type=Path, required=True)
    parser.add_argument('--ctl', type=Path, required=True)
    parser.add_argument('--resources', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    from PIL import Image, ImageChops
    game, ctl, resources, out = (p.resolve() for p in (args.game, args.ctl, args.resources, args.out))
    out.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='hz-armor-', dir='/tmp') as scratch:
        root = Path(scratch)
        env = lab.game_env()
        for key in ('HORIZON_HOME', 'HORIZON_LAUNCHER_HOME', 'XDG_DATA_HOME', 'XDG_CONFIG_HOME',
                    'HORIZON_LAUNCHER_TRASH', 'HORIZON_LAUNCHER_CLIPBOARD', 'HORIZON_LAUNCHER_DESKTOP'):
            folder = root / key.lower()
            folder.mkdir()
            env[key] = str(folder)
        socket = str(root / 'ctl.sock')

        def call(*words):
            result = subprocess.run([str(ctl), '--socket', socket, *words], capture_output=True,
                                    text=True, timeout=15, check=True)
            answer = json.loads(result.stdout)
            if answer.get('error') or answer.get('ok') is False:
                raise RuntimeError(answer)
            return answer

        def settle():
            # Let teleport interpolation, lighting and queued mesh uploads reach a stable frame.
            start = call('state')['ticks']
            deadline = time.monotonic() + 10
            while time.monotonic() < deadline:
                if call('state')['ticks'] >= start + 10:
                    return
                time.sleep(0.05)
            raise TimeoutError('Simulation did not advance to a stable capture')

        with (out / 'game.log').open('w') as log:
            process = subprocess.Popen([str(game), '--resources', str(resources), '--world', 'ArmorRender',
                                        '--test-terrain', '--creative', '--render-distance', '2',
                                        '--offscreen', '--no-server-thread', '--control-socket', socket],
                                       cwd=root, env=env, stdout=log, stderr=subprocess.STDOUT)
            try:
                deadline = time.monotonic() + 90
                while time.monotonic() < deadline:
                    if process.poll() is not None:
                        raise RuntimeError('Game exited before world was ready; see game.log')
                    if Path(socket).exists():
                        state = call('state')
                        if state.get('screen') == 'playing' and not state.get('busy'):
                            break
                    time.sleep(0.2)
                else:
                    raise TimeoutError('World did not become ready')
                call('command', 'time set 6000')
                call('command', 'weather clear')
                call('command', 'fill 18 100 15 23 100 23 minecraft:stone')
                call('command', 'tp @s 20.5 101 17.5 0 0')
                settle()
                equipment = ',ShowArms:1b,equipment:{mainhand:{id:"minecraft:diamond_sword",count:1},head:{id:"minecraft:stone",count:1}}'
                for name, invisible, gear in [('invisible-empty', 1, ''),
                                              ('invisible-equipped', 1, equipment),
                                              ('visible-equipped', 0, equipment)]:
                    call('command', 'kill @e[type=minecraft:armor_stand]')
                    call('command', f'summon minecraft:armor_stand 20.5 101 20.5 {{Invisible:{invisible}b{gear}}}')
                    settle()
                    call('screenshot', str(out / (name + '.png')))
                    if name == 'invisible-empty':
                        settle()
                        call('screenshot', str(out / 'invisible-empty-repeat.png'))
                # Camera/world fixed; this region excludes the HUD/chat and includes head, arms and held item.
                crop = (560, 280, 740, 510)
                images = [Image.open(out / (name + '.png')).convert('RGB').crop(crop)
                          for name in ('invisible-empty', 'invisible-equipped', 'visible-equipped')]
                repeat = Image.open(out / 'invisible-empty-repeat.png').convert('RGB').crop(crop)
                background_noise = sum(max(pixel) > 32 for pixel in ImageChops.difference(images[0], repeat).getdata())
                if background_noise >= 1000:
                    raise AssertionError(f'Camera/background was not stable: {background_noise} pixels changed')
                changed = [sum(max(pixel) > 32 for pixel in ImageChops.difference(a, b).getdata())
                           for a, b in zip(images, images[1:])]
                if min(changed) < 1000:
                    raise AssertionError(f'Missing equipment or stand body in control images: {changed}')
                (out / 'results.json').write_text(json.dumps({'passed': True,
                    'background_pixels_changed': background_noise,
                    'equipment_pixels_changed': changed[0], 'body_pixels_changed': changed[1]}, indent=2) + '\n')
                print('PASS invisible equipment retained; visible-body control differs:', changed)
            finally:
                if process.poll() is None:
                    try:
                        call('quit')
                        process.wait(timeout=10)
                    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError):
                        process.terminate()
                        try:
                            process.wait(timeout=10)
                        except subprocess.TimeoutExpired:
                            process.kill()
                            process.wait()


if __name__ == '__main__':
    main()
