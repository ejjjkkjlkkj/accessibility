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

    def test_rejects_protected_git_header_even_when_body_uses_safe_paths(self):
        patch = (
            "diff --git a/.github/workflows/ci.yml b/.github/workflows/ci.yml\n"
            "--- a/src/example.py\n"
            "+++ b/src/example.py\n"
            "@@ -1 +1 @@\n-old\n+new\n"
        )
        with self.assertRaisesRegex(RuntimeError, "protected path"):
            local_agent_loop.validate_patch(patch)

    def test_rejects_github_agent_task_changes(self):
        patch = (
            "diff --git a/.github/agent-task.md b/.github/agent-task.md\n"
            "--- a/.github/agent-task.md\n"
            "+++ b/.github/agent-task.md\n"
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




class OllamaEndpointValidationTests(unittest.TestCase):
    def test_accepts_ipv4_loopback_and_adds_default_port(self):
        self.assertEqual(
            local_agent_loop.validate_ollama_url("http://127.0.0.1/"),
            "http://127.0.0.1:11434",
        )

    def test_accepts_ipv6_loopback(self):
        self.assertEqual(
            local_agent_loop.validate_ollama_url("http://[::1]:11434/"),
            "http://[::1]:11434",
        )

    def test_rejects_remote_or_non_literal_hosts(self):
        for value in (
            "https://api.example.test",
            "http://192.168.1.10:11434",
            "http://localhost:11434",
            "http://127.0.0.1@evil.example:11434",
        ):
            with self.subTest(value=value):
                with self.assertRaises(ValueError):
                    local_agent_loop.validate_ollama_url(value)

    def test_rejects_credentials_paths_queries_and_fragments(self):
        for value in (
            "http://user:pass@127.0.0.1:11434",
            "http://127.0.0.1:11434/api",
            "http://127.0.0.1:11434?forward=1",
            "http://127.0.0.1:11434#fragment",
        ):
            with self.subTest(value=value):
                with self.assertRaises(ValueError):
                    local_agent_loop.validate_ollama_url(value)

    def test_rejects_invalid_ports(self):
        for value in ("http://127.0.0.1:0", "http://127.0.0.1:65536", "http://127.0.0.1:not-a-port"):
            with self.subTest(value=value):
                with self.assertRaises(ValueError):
                    local_agent_loop.validate_ollama_url(value)


class CargoValidationCommandTests(unittest.TestCase):
    def test_uefi_manifest_uses_target_build_not_host_tests(self):
        manifest = local_agent_loop.ROOT / "src" / "rust-uefi" / "Cargo.toml"
        command = local_agent_loop.cargo_command_for_manifest(manifest)
        self.assertEqual(command[1], "build")
        self.assertIn("--target", command)
        self.assertIn("x86_64-unknown-uefi", command)
        self.assertNotEqual(command[1], "test")

    def test_regular_rust_manifest_uses_tests(self):
        manifest = local_agent_loop.ROOT / "crates" / "accessibility-core" / "Cargo.toml"
        command = local_agent_loop.cargo_command_for_manifest(manifest)
        self.assertEqual(command[1], "test")


class AgentLoopSchedulingTests(unittest.TestCase):
    def test_final_iteration_does_not_sleep(self):
        from unittest.mock import patch

        with (
            patch.object(local_agent_loop, "MAX_RUNS", 1),
            patch.object(local_agent_loop, "once") as once,
            patch.object(local_agent_loop.time, "sleep") as sleep,
        ):
            local_agent_loop.main()
        once.assert_called_once()
        sleep.assert_not_called()


if __name__ == "__main__":
    unittest.main()
