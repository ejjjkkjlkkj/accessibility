import importlib.util
import unittest
from pathlib import Path
from unittest.mock import MagicMock, patch
import urllib.error

MODULE_PATH = Path(__file__).resolve().parents[1] / "tools" / "build_resource_db.py"
SPEC = importlib.util.spec_from_file_location("build_resource_db", MODULE_PATH)
builder = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(builder)


class CheckUrlTests(unittest.TestCase):
    def test_rejects_non_http_url_without_network(self):
        result = builder.check_url("file:///etc/passwd")
        self.assertFalse(result["reachable"])
        self.assertEqual(result["error"], "invalid_http_url")

    @patch.object(builder.urllib.request, "urlopen")
    def test_successful_head_is_reachable(self, urlopen):
        response = MagicMock()
        response.status = 200
        response.geturl.return_value = "https://example.org/"
        response.__enter__.return_value = response
        urlopen.return_value = response

        result = builder.check_url("https://example.org/")

        self.assertTrue(result["reachable"])
        self.assertEqual(result["http_status"], 200)
        self.assertEqual(result["final_url"], "https://example.org/")
        self.assertTrue(result["tls"])

    @patch.object(builder.urllib.request, "urlopen")
    def test_head_405_retries_with_bounded_range_get(self, urlopen):
        response = MagicMock()
        response.status = 206
        response.geturl.return_value = "https://example.org/"
        response.__enter__.return_value = response
        urlopen.side_effect = [
            urllib.error.HTTPError("https://example.org/", 405, "Method Not Allowed", {}, None),
            response,
        ]

        result = builder.check_url("https://example.org/")

        self.assertTrue(result["reachable"])
        self.assertEqual(result["http_status"], 206)
        self.assertEqual(urlopen.call_count, 2)
        retry_request = urlopen.call_args_list[1].args[0]
        self.assertEqual(retry_request.get_header("Range"), "bytes=0-0")

    @patch.object(builder.urllib.request, "urlopen")
    def test_network_failure_is_partial_not_exception(self, urlopen):
        urlopen.side_effect = TimeoutError()
        result = builder.check_url("https://example.org/")
        self.assertFalse(result["reachable"])
        self.assertEqual(result["error"], "TimeoutError")


if __name__ == "__main__":
    unittest.main()
