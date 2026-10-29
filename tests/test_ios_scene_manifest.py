"""Guard the scene adoption required to launch SDK-27 builds on iPadOS 27."""
import plistlib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class IOSSceneManifestTests(unittest.TestCase):
    def test_game_uses_one_programmatic_window_scene(self):
        with (ROOT / "packaging/ios/GameInfo.plist").open("rb") as source:
            info = plistlib.load(source)
        manifest = info["UIApplicationSceneManifest"]
        self.assertIs(manifest["UIApplicationSupportsMultipleScenes"], False)
        configs = manifest["UISceneConfigurations"]["UIWindowSceneSessionRoleApplication"]
        self.assertEqual(len(configs), 1)
        self.assertEqual(configs[0]["UISceneClassName"], "UIWindowScene")
        self.assertEqual(configs[0]["UISceneDelegateClassName"], "WinitSceneDelegate")
        self.assertNotIn("UISceneStoryboardFile", configs[0])
        self.assertEqual(info["MinimumOSVersion"], "16.0")

    def test_plist_delegate_matches_registered_backend_class(self):
        with (ROOT / "packaging/ios/GameInfo.plist").open("rb") as source:
            config = plistlib.load(source)["UIApplicationSceneManifest"]["UISceneConfigurations"]["UIWindowSceneSessionRoleApplication"][0]
        backend = (ROOT / "third_party/winit/src/platform_impl/ios/scene.rs").read_text()
        self.assertIn(f'const NAME: &\'static str = "{config["UISceneDelegateClassName"]}";', backend)


if __name__ == "__main__":
    unittest.main()
