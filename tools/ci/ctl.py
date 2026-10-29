#!/usr/bin/env python3
"""A stand-in for horizon-ctl where only the socket is needed (a Mac driving the iOS Simulator, with no game build
for the Mac itself): ctl.py --socket PATH WORDS... sends the words as one line and prints the game's JSON reply.
`wait-ready [S]` and `wait-for screen NAME [--timeout S]` wait on the client side, as horizon-ctl does."""
import json
import socket
import sys
import time


def send(path: str, line: str) -> dict:
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
        s.settimeout(120)
        s.connect(path)
        s.sendall((line + "\n").encode())
        data = b""
        while not data.endswith(b"\n"):
            chunk = s.recv(65536)
            if not chunk:
                break
            data += chunk
    return json.loads(data or b"{}")


def main() -> int:
    args = sys.argv[1:]
    if len(args) < 3 or args[0] != "--socket":
        print(__doc__, file=sys.stderr)
        return 2
    path, words = args[1], args[2:]
    try:
        if words[0] == "wait-ready":
            limit = float(words[1]) if len(words) > 1 else 120
            deadline = time.time() + limit
            while True:
                status = send(path, "status")
                if status.get("player") is not None and status.get("pending_chunks") == 0 \
                        and (status.get("server") or {}).get("loaded") is not False:
                    print(json.dumps({"ok": True}))
                    return 0
                if time.time() > deadline:
                    print(json.dumps({"error": f"not ready after {limit} seconds"}))
                    return 1
                time.sleep(0.25)
        if words[0] == "wait-for" and len(words) >= 3 and words[1] == "screen":
            limit = float(words[words.index("--timeout") + 1]) if "--timeout" in words else 30
            deadline = time.time() + limit
            while time.time() < deadline:
                if words[2] in json.dumps(send(path, "screen")):
                    print(json.dumps({"ok": True}))
                    return 0
                time.sleep(0.5)
            print(json.dumps({"error": "timed out"}))
            return 1
        reply = send(path, " ".join(words))
    except (OSError, ValueError) as error:
        print(json.dumps({"error": str(error)}))
        return 1
    print(json.dumps(reply))
    return 1 if "error" in reply else 0


if __name__ == "__main__":
    sys.exit(main())
