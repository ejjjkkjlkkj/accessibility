import unittest

from tools import local_agent_loop


class AgentPatchValidationTests(unittest.TestCase):
    def test_auto_push_is_disabled_by_default(self):
        from unittest.mock import patch

        with patch.dict("os.environ", {}, clear=True):
            self.assertFalse(local_agent_loop.auto_push_enabled())
        self.assertFalse(local_agent_loop.auto_push_enabled("0"))
        self.assertFalse(local_agent_loop.auto_push_enabled("false"))
        self.assertFalse(local_agent_loop.auto_push_enabled("no"))

    def test_auto_push_requires_explicit_opt_in(self):
        self.assertTrue(local_agent_loop.auto_push_enabled("1"))
        self.assertTrue(local_agent_loop.auto_push_enabled("true"))
        self.assertTrue(local_agent_loop.auto_push_enabled("yes"))

    def test_accepts_small_source_and_test_patch(self):
        patch = (
            "diff --git a/src/example.py b/src/example.py\n"
            "--- a/src/example.py\n"
            "+++ b/src/example.py\n"
            "@@ -1 +1 @@\n-old\n+new\n"
        )
        self.assertEqual(local_agent_loop.validate_patch(patch), ["src/example.py"])

    def test_rejects_no_patch(self):
        with self.assertRaisesRegex(RuntimeError, "no patch"):
            local_agent_loop.validate_patch("NO_PATCH")

    def test_rejects_path_traversal(self):
        patch = (
            "diff --git a/src/example.py b/../../outside.py\n"
            "--- a/src/example.py\n"
            "+++ b/../../outside.py\n"
            "@@ -1 +1 @@\n-old\n+new\n"
        )
        with self.assertRaisesRegex(RuntimeError, "unsafe patch path"):
            local_agent_loop.validate_patch(patch)


    def test_rejects_git_pathspec_magic(self):
        patch = (
            "diff --git a/src/example.py b/:(glob)**\n"
            "--- a/src/example.py\n"
            "+++ b/:(glob)**\n"
            "@@ -1 +1 @@\n-old\n+new\n"
        )
        with self.assertRaisesRegex(RuntimeError, "unsafe patch path"):
            local_agent_loop.validate_patch(patch)


    def test_rejects_mixed_case_workflow_path(self):
        patch = (
            "diff --git a/.GITHUB/WORKFLOWS/ci.yml b/.GITHUB/WORKFLOWS/ci.yml\n"
            "--- a/.GITHUB/WORKFLOWS/ci.yml\n"
            "+++ b/.GITHUB/WORKFLOWS/ci.yml\n"
            "@@ -1 +1 @@\n-old\n+new\n"
        )
        with self.assertRaisesRegex(RuntimeError, "protected path"):
            local_agent_loop.validate_patch(patch)

    def test_rejects_workflow_changes(self):
        patch = (
            "diff --git a/.github/workflows/ci.yml b/.github/workflows/ci.yml\n"
            "--- a/.github/workflows/ci.yml\n"
            "+++ b/.github/workflows/ci.yml\n"
            "@@ -1 +1 @@\n-old\n+new\n"
        )
        with self.assertRaisesRegex(RuntimeError, "protected path"):
            local_agent_loop.validate_patch(patch)


    def test_rejects_team_bootstrap_changes(self):
        patch = (
            "diff --git a/tools/install_both_agents.ps1 b/tools/install_both_agents.ps1\n"
            "--- a/tools/install_both_agents.ps1\n"
            "+++ b/tools/install_both_agents.ps1\n"
            "@@ -1 +1 @@\n-old\n+new\n"
        )
        with self.assertRaisesRegex(RuntimeError, "protected path"):
            local_agent_loop.validate_patch(patch)

    def test_rejects_runner_changes(self):
        patch = (
            "diff --git a/tools/local_agent_loop.py b/tools/local_agent_loop.py\n"
            "--- a/tools/local_agent_loop.py\n"
            "+++ b/tools/local_agent_loop.py\n"
            "@@ -1 +1 @@\n-old\n+new\n"
        )
        with self.assertRaisesRegex(RuntimeError, "protected path"):
            local_agent_loop.validate_patch(patch)


if __name__ == "__main__":
    unittest.main()
