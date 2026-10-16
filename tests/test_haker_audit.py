import json, os, subprocess, sys, tempfile, unittest
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


    def test_bounded_reader_rejects_fifo_without_blocking(self):
        if not hasattr(os, "mkfifo"):
            self.skipTest("named pipes unavailable on this platform")
        with tempfile.TemporaryDirectory() as temp:
            fifo = Path(temp) / "scanner-input.fifo"
            os.mkfifo(fifo)
            probe = (
                "import sys; from pathlib import Path; "
                f"sys.path.insert(0, {str(SCANNER.parent)!r}); "
                "import haker_audit; "
                "print(repr(haker_audit.read_bounded_text(Path(sys.argv[1]))))"
            )
            result = subprocess.run(
                [sys.executable, "-c", probe, str(fifo)],
                text=True, capture_output=True, check=False, timeout=3,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual(result.stdout.strip(), "None")
            self.assertNotIn("Traceback", result.stderr)


    def test_refuses_to_overwrite_report_with_duplicate_json_keys(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "clean.py").write_text("value = 42\n", encoding="utf-8")
            report = root / "report.json"
            original = (
                '{"schema_version":"0.9","schema_version":"1.0",'
                '"audit_type":"OFFLINE_HEURISTIC_STATIC_REVIEW"}'
            )
            report.write_text(original, encoding="utf-8")
            result = subprocess.run(
                [sys.executable, str(SCANNER), "--root", str(root), "--json-out", str(report)],
                text=True, capture_output=True, check=False, timeout=10,
            )
            self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
            self.assertIn("refusing to overwrite", result.stderr)
            self.assertEqual(report.read_text(encoding="utf-8"), original)
            self.assertNotIn("Traceback", result.stderr)

    def test_refuses_to_overwrite_report_with_inconsistent_severity_counts(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "clean.py").write_text("value = 42\n", encoding="utf-8")
            report = root / "report.json"
            data = {
                "schema_version": "1.0",
                "audit_type": "OFFLINE_HEURISTIC_STATIC_REVIEW",
                "root": ".",
                "files_scanned": 0,
                "unreadable_files": [],
                "finding_count": 0,
                "severity_counts": {
                    "CRITICAL": 1, "HIGH": 0, "MEDIUM": 0, "LOW": 0, "INFO": 0,
                },
                "findings": [],
                "limitations": ["This is a test fixture, not a security certification."],
            }
            original = json.dumps(data, sort_keys=True)
            report.write_text(original, encoding="utf-8")
            result = subprocess.run(
                [sys.executable, str(SCANNER), "--root", str(root), "--json-out", str(report)],
                text=True, capture_output=True, check=False, timeout=10,
            )
            self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
            self.assertIn("refusing to overwrite", result.stderr)
            self.assertEqual(report.read_text(encoding="utf-8"), original)
            self.assertNotIn("Traceback", result.stderr)

    def test_refuses_to_overwrite_report_with_nonstandard_json_constant(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "clean.py").write_text("value = 42\n", encoding="utf-8")
            report = root / "report.json"
            original = (
                '{"schema_version":"1.0",'
                '"audit_type":"OFFLINE_HEURISTIC_STATIC_REVIEW","extra":NaN}'
            )
            report.write_text(original, encoding="utf-8")
            result = subprocess.run(
                [sys.executable, str(SCANNER), "--root", str(root), "--json-out", str(report)],
                text=True, capture_output=True, check=False, timeout=10,
            )
            self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
            self.assertIn("refusing to overwrite", result.stderr)
            self.assertEqual(report.read_text(encoding="utf-8"), original)
            self.assertNotIn("Traceback", result.stderr)

    def test_refuses_to_overwrite_report_with_unknown_deeply_nested_field(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "clean.py").write_text("value = 42\n", encoding="utf-8")
            report = root / "report.json"
            original = (
                b'{"schema_version":"1.0",'
                b'"audit_type":"OFFLINE_HEURISTIC_STATIC_REVIEW","nested":'
                + b"[" * 1500 + b"0" + b"]" * 1500 + b"}"
            )
            report.write_bytes(original)
            result = subprocess.run(
                [sys.executable, str(SCANNER), "--root", str(root), "--json-out", str(report)],
                text=True, capture_output=True, check=False, timeout=10,
            )
            self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
            self.assertIn("refusing to overwrite", result.stderr)
            self.assertEqual(report.read_bytes(), original)
            self.assertNotIn("Traceback", result.stderr)

    def test_bounded_reader_refuses_symlink_directly(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            target = base / "target.py"
            target.write_text("value = 42\n", encoding="utf-8")
            link = base / "linked.py"
            try:
                link.symlink_to(target)
            except (OSError, NotImplementedError):
                self.skipTest("file symlinks unavailable")
            probe = (
                "import sys\n"
                "from pathlib import Path\n"
                f"sys.path.insert(0, {str(SCANNER.parent)!r})\n"
                "import haker_audit\n"
                "try:\n"
                "    value = haker_audit.read_bounded_text(Path(sys.argv[1]))\n"
                "except OSError:\n"
                "    print('REFUSED')\n"
                "else:\n"
                "    print('READ' if value is not None else 'REFUSED')\n"
            )
            result = subprocess.run(
                [sys.executable, "-c", probe, str(link)],
                text=True, capture_output=True, check=False, timeout=5,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual(result.stdout.strip(), "REFUSED")
            self.assertNotIn("Traceback", result.stderr)

    def test_secret_evidence_is_redacted_from_json_report(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            secret = "never-print-this-secret-value-42"
            (root / "sample.py").write_text(
                'password = "' + secret + '"\\n', encoding="utf-8"
            )
            report = root / "report.json"
            result = subprocess.run(
                [sys.executable, str(SCANNER), "--root", str(root), "--json-out", str(report)],
                text=True, capture_output=True, check=False, timeout=10,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            rendered = report.read_text(encoding="utf-8")
            self.assertIn("[REDACTED:", rendered)
            self.assertNotIn(secret, rendered)

    def test_private_key_marker_is_redacted_from_json_report(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            marker = "-----BEGIN RSA " + "PRIVATE KEY-----"
            (root / "sample.py").write_text("key = " + repr(marker) + "\\n", encoding="utf-8")
            report = root / "report.json"
            result = subprocess.run(
                [sys.executable, str(SCANNER), "--root", str(root), "--json-out", str(report)],
                text=True, capture_output=True, check=False, timeout=10,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            rendered = report.read_text(encoding="utf-8")
            self.assertIn("[REDACTED: private-key marker]", rendered)
            self.assertNotIn(marker, rendered)

    def test_flags_unsafe_deserialization_and_legacy_hashes(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "sample.py").write_text(
                "item = pickle.loads(blob)\\n"
                "digest = hashlib.md5(payload)\\n"
                "path = tempfile.mktemp()\\n",
                encoding="utf-8",
            )
            result = self.run_scanner(root)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("SEC007", result.stdout)
            self.assertIn("SEC008", result.stdout)
            self.assertIn("SEC009", result.stdout)


    def test_secret_is_redacted_for_every_finding_on_same_line(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            secret = "do-not-leak-this-value-987654"
            (root / "sample.py").write_text(
                'password = "' + secret + '"; eval(user_input)\\n',
                encoding="utf-8",
            )
            report = root / "report.json"
            result = subprocess.run(
                [sys.executable, str(SCANNER), "--root", str(root), "--json-out", str(report)],
                text=True, capture_output=True, check=False, timeout=10,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            data = json.loads(report.read_text(encoding="utf-8"))
            self.assertIn("SEC002", [item["id"] for item in data["findings"]])
            self.assertIn("SEC004", [item["id"] for item in data["findings"]])
            self.assertNotIn(secret, json.dumps(data))
            self.assertTrue(all("[REDACTED" in item["evidence"] for item in data["findings"]))


    def test_flags_fstring_sql_query_assembly(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "sample.py").write_text(
                'cursor.execute(f"SELECT * FROM users WHERE name = {name}")\\n',
                encoding="utf-8",
            )
            result = self.run_scanner(root)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("SEC010", result.stdout)


if __name__ == "__main__": unittest.main()
