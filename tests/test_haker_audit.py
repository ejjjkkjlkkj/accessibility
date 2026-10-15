import json, subprocess, sys, tempfile, unittest
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
SCANNER = ROOT / "scripts" / "haker_audit.py"
class HakerAuditTests(unittest.TestCase):
    def run_scanner(self, root):
        return subprocess.run([sys.executable, str(SCANNER), "--root", str(root)], text=True, capture_output=True, check=False, timeout=10)
    def test_detects_private_key_marker_without_executing_content(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp); marker="-----BEGIN RSA "+"PRIVATE KEY-----"
            (root/"sample.py").write_text("PRIVATE = "+repr(marker)+"\n", encoding="utf-8")
            result=self.run_scanner(root)
            self.assertEqual(result.returncode,0,result.stdout+result.stderr); self.assertIn("SEC001",result.stdout)
    def test_clean_input_reports_limitations(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp); (root/"clean.py").write_text("value = 42\n",encoding="utf-8")
            result=self.run_scanner(root)
            self.assertEqual(result.returncode,0,result.stdout+result.stderr); self.assertIn("FINDINGS: 0",result.stdout); self.assertIn("not a security certification",result.stdout)
    def test_json_report_has_severity_counts(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp); (root/"clean.py").write_text("value = 42\n",encoding="utf-8"); report=root/"out"/"report.json"
            result=subprocess.run([sys.executable,str(SCANNER),"--root",str(root),"--json-out",str(report)],text=True,capture_output=True,check=False,timeout=10)
            self.assertEqual(result.returncode,0,result.stdout+result.stderr); data=json.loads(report.read_text(encoding="utf-8"))
            self.assertEqual(data["audit_type"],"OFFLINE_HEURISTIC_STATIC_REVIEW"); self.assertEqual(data["root"],"."); self.assertEqual(data["severity_counts"]["HIGH"],0)
    def test_report_is_reproducible_across_absolute_roots(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp); reports=[]
            for name in ("one","two"):
                root=base/name; root.mkdir(); (root/"clean.py").write_text("value = 42\n",encoding="utf-8")
                report=root/"report.json"
                result=subprocess.run([sys.executable,str(SCANNER),"--root",str(root),"--json-out",str(report)],text=True,capture_output=True,check=False,timeout=10)
                self.assertEqual(result.returncode,0,result.stdout+result.stderr)
                data=json.loads(report.read_text(encoding="utf-8")); data.pop("root",None); reports.append(data)
            self.assertEqual(reports[0],reports[1])
    def test_excluded_dependency_trees_are_pruned(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp); excluded=root/"node_modules"/"deep"; excluded.mkdir(parents=True)
            (excluded/"fixture.py").write_text("PRIVATE = "+repr("-----BEGIN RSA "+"PRIVATE KEY-----")+"\n",encoding="utf-8")
            (root/"clean.py").write_text("value = 42\n",encoding="utf-8")
            result=self.run_scanner(root)
            self.assertEqual(result.returncode,0,result.stdout+result.stderr)
            self.assertIn("scanned 1 text files",result.stdout)
            self.assertNotIn("SEC001",result.stdout)

    def test_symlink_is_not_scanned(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp)/"root"; outside=Path(temp)/"outside"; root.mkdir(); outside.mkdir()
            (outside/"secret.py").write_text("value = 1\n",encoding="utf-8")
            try: (root/"linked").symlink_to(outside,target_is_directory=True)
            except (OSError,NotImplementedError): self.skipTest("directory symlinks unavailable")
            result=self.run_scanner(root); self.assertEqual(result.returncode,0,result.stdout+result.stderr); self.assertIn("scanned 0 text files",result.stdout)

    def test_oversized_file_is_skipped(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "oversized.py").write_bytes(b"x" * (2 * 1024 * 1024 + 1))
            (root / "clean.py").write_text("value = 42\n", encoding="utf-8")
            result = self.run_scanner(root)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("scanned 1 text files", result.stdout)
            self.assertNotIn("oversized.py", result.stdout)

    def test_invalid_utf8_is_listed_as_unreadable(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "broken.py").write_bytes(b"\xff\xfe")
            report = root / "report.json"
            result = subprocess.run(
                [sys.executable, str(SCANNER), "--root", str(root), "--json-out", str(report)],
                text=True, capture_output=True, check=False, timeout=10,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            data = json.loads(report.read_text(encoding="utf-8"))
            self.assertEqual(data["files_scanned"], 0)
            self.assertEqual(data["unreadable_files"], ["broken.py"])

    def test_symlink_file_is_not_scanned(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "root"
            outside = Path(temp) / "outside.py"
            root.mkdir()
            marker = "-----BEGIN RSA " + "PRIVATE KEY-----"
            outside.write_text("PRIVATE = " + repr(marker) + "\n", encoding="utf-8")
            try:
                (root / "linked.py").symlink_to(outside)
            except (OSError, NotImplementedError):
                self.skipTest("file symlinks unavailable")
            result = self.run_scanner(root)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("scanned 0 text files", result.stdout)
            self.assertNotIn("SEC001", result.stdout)


    def test_existing_report_output_is_excluded_on_repeat_run(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "clean.py").write_text("value = 42\n", encoding="utf-8")
            report = root / "report.json"
            command = [sys.executable, str(SCANNER), "--root", str(root), "--json-out", str(report)]
            first = subprocess.run(command, text=True, capture_output=True, check=False, timeout=10)
            self.assertEqual(first.returncode, 0, first.stdout + first.stderr)
            first_data = json.loads(report.read_text(encoding="utf-8"))
            second = subprocess.run(command, text=True, capture_output=True, check=False, timeout=10)
            self.assertEqual(second.returncode, 0, second.stdout + second.stderr)
            second_data = json.loads(report.read_text(encoding="utf-8"))
            self.assertEqual(first_data, second_data)
            self.assertEqual(second_data["files_scanned"], 1)

    def test_refuses_to_overwrite_existing_source_file(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            source = root / "source.py"
            original = "value = 42\n"
            source.write_text(original, encoding="utf-8")
            result = subprocess.run(
                [sys.executable, str(SCANNER), "--root", str(root), "--json-out", str(source)],
                text=True, capture_output=True, check=False, timeout=10,
            )
            self.assertEqual(result.returncode, 2)
            self.assertIn("refusing to overwrite", result.stderr)
            self.assertEqual(source.read_text(encoding="utf-8"), original)

    def test_refuses_symlink_report_output(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "root"
            root.mkdir()
            target = Path(temp) / "target.json"
            target.write_text('{"keep": true}', encoding="utf-8")
            output = root / "report.json"
            try:
                output.symlink_to(target)
            except (OSError, NotImplementedError):
                self.skipTest("file symlinks unavailable")
            result = subprocess.run(
                [sys.executable, str(SCANNER), "--root", str(root), "--json-out", str(output)],
                text=True, capture_output=True, check=False, timeout=10,
            )
            self.assertEqual(result.returncode, 2)
            self.assertIn("must not be a symbolic link", result.stderr)
            self.assertEqual(target.read_text(encoding="utf-8"), '{"keep": true}')

    def test_rule_table_marker_cannot_hide_findings_in_audited_source(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            candidate = root / "sample.py"
            candidate.write_text(
                "RULES = [\n"
                'password = "supersecretpasswordvalue"\n'
                "]\n",
                encoding="utf-8",
            )
            result = self.run_scanner(root)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("SEC002", result.stdout)
            self.assertIn("sample.py:2", result.stdout)


    def test_fixture_marker_cannot_suppress_a_real_finding(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            candidate = root / "sample.py"
            candidate.write_text(
                'password = "supersecretpasswordvalue" # SECURITY_SCANNER_TEST_FIXTURE\n',
                encoding="utf-8",
            )
            result = self.run_scanner(root)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("SEC002", result.stdout)
            self.assertIn("sample.py:1", result.stdout)


    def test_refuses_to_overwrite_oversized_existing_report(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "clean.py").write_text("value = 42\\n", encoding="utf-8")
            report = root / "report.json"
            prefix = (
                b'{"schema_version":"1.0",'
                b'"audit_type":"OFFLINE_HEURISTIC_STATIC_REVIEW"}'
            )
            report.write_bytes(prefix + b" " * (8 * 1024 * 1024 + 1 - len(prefix)))
            original_size = report.stat().st_size

            result = subprocess.run(
                [sys.executable, str(SCANNER), "--root", str(root), "--json-out", str(report)],
                text=True, capture_output=True, check=False, timeout=10,
            )
            self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
            self.assertIn("refusing to overwrite", result.stderr)
            self.assertEqual(report.stat().st_size, original_size)

if __name__ == "__main__": unittest.main()
