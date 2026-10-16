import unittest

from scripts.validate_release_tag import validate_release_tag


class ReleaseTagValidationTests(unittest.TestCase):
    def test_accepts_stable_and_prerelease_tags(self):
        for tag in ("v0.1.0", "v1.2.3", "v0.1.0-alpha.1", "v2.0.0-rc.2"):
            with self.subTest(tag=tag):
                self.assertEqual(validate_release_tag(tag), tag)

    def test_rejects_invalid_versions(self):
        for tag in (
            "", "v1.2", "1.2.3", "v01.2.3", "v1.02.3", "v1.2.03",
            "v1.2.3-", "v1.2.3-alpha.", "v1.2.3-..", "v1.2.3-01",
            "v1.2.3+build.1", "v1.2.3/../../main",
        ):
            with self.subTest(tag=tag):
                with self.assertRaises(ValueError):
                    validate_release_tag(tag)

    def test_rejects_shell_metacharacters_without_evaluating_them(self):
        for tag in (
            "v1.2.3$(touch /tmp/should-not-exist)",
            "v1.2.3; echo injected",
            "v1.2.3\n touch /tmp/should-not-exist",
            chr(96) + "id" + chr(96),
        ):
            with self.subTest(tag=tag):
                with self.assertRaises(ValueError):
                    validate_release_tag(tag)


if __name__ == "__main__":
    unittest.main()
