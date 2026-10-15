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
            (excluded/"fixture.py").write_text("PRIVATE = "+repr("-----BEGIN RSA "+"PRIVATE KEY-----")+"\\n",encoding="utf-8")
            (root/"clean.py").write_text("value = 42\\n",encoding="utf-8")
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
if __name__ == "__main__": unittest.main()
