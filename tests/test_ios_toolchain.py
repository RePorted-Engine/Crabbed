"""Check the actual CI preflight rejects SDKs that cannot compile Liquid Glass."""
from pathlib import Path
import os
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class IOSToolchainTests(unittest.TestCase):
    def check_toolchain(self, version, sdk):
        with tempfile.TemporaryDirectory(prefix="horizon-xcode-preflight-") as temp:
            directory = Path(temp)
            # Only external tool output is substituted; run the production preflight.
            for tool, output in (("xcodebuild", f"Xcode {version}\\nBuild version fixture"),
                                 ("xcrun", sdk)):
                path = directory / tool
                path.write_text(f"#!/bin/sh\nprintf '%b\\n' '{output}'\n")
                path.chmod(0o755)
            env = {**os.environ, "PATH": str(directory) + os.pathsep + os.environ["PATH"]}
            return subprocess.run(["bash", str(ROOT / "tools/ci/ios-toolchain.sh")],
                                  env=env, capture_output=True, text=True, timeout=10)

    def test_old_xcode_fails_before_the_build(self):
        result = self.check_toolchain("16.4", "18.5")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("requires Xcode 26", result.stderr)

    def test_sdk_override_cannot_silently_select_old_headers(self):
        result = self.check_toolchain("26.6", "18.5")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("SDK must be 26", result.stderr)

    def test_current_and_newer_toolchains_pass(self):
        for version in ("26.6", "27.0"):
            with self.subTest(version=version):
                result = self.check_toolchain(version, version)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
