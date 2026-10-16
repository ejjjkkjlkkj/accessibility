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

    def test_rejects_local_ip_and_localhost_without_network(self):
        for url in (
            "http://127.0.0.1/",
            "https://[::1]/",
            "http://localhost/",
            "https://service.internal/",
            "https://printer.local/",
        ):
            with self.subTest(url=url):
                result = builder.check_url(url)
                self.assertFalse(result["reachable"])
                self.assertEqual(result["error"], "invalid_http_url")

    def test_rejects_url_userinfo_and_invalid_ports_without_network(self):
        for url in (
            "https://user:pass@example.org/",
            "https://example.org:0/",
            "https://example.org:65536/",
            "https://example.org:notaport/",
        ):
            with self.subTest(url=url):
                result = builder.check_url(url)
                self.assertFalse(result["reachable"])
                self.assertEqual(result["error"], "invalid_http_url")

    @patch.object(builder, "_open_catalog_url")
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

    @patch.object(builder, "_open_catalog_url")
    def test_plain_http_response_reports_tls_false(self, urlopen):
        response = MagicMock()
        response.status = 200
        response.geturl.return_value = "http://example.org/"
        response.__enter__.return_value = response
        urlopen.return_value = response

        result = builder.check_url("http://example.org/")

        self.assertTrue(result["reachable"])
        self.assertFalse(result["tls"])

    def test_initial_url_must_match_declared_catalog_domain(self):
        result = builder.check_url("https://evil.test/", "example.org")
        self.assertFalse(result["reachable"])
        self.assertEqual(result["error"], "redirect_outside_declared_domain")

    @patch.object(builder, "_open_catalog_url")
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

    @patch.object(builder, "_open_catalog_url")
    def test_network_failure_is_partial_not_exception(self, urlopen):
        urlopen.side_effect = TimeoutError()
        result = builder.check_url("https://example.org/")
        self.assertFalse(result["reachable"])
        self.assertEqual(result["error"], "TimeoutError")


    def test_rejects_redirect_outside_declared_domain(self):
        handler = builder._SafeCatalogRedirectHandler("example.org")
        request = builder.urllib.request.Request("https://example.org/start", method="HEAD")
        self.assertIsNone(
            handler.redirect_request(request, None, 302, "Found", {}, "https://evil.test/")
        )

    def test_rejects_redirect_to_localhost_or_ip_literal(self):
        handler = builder._SafeCatalogRedirectHandler("example.org")
        request = builder.urllib.request.Request("https://example.org/start", method="HEAD")
        for target in ("https://127.0.0.1/", "https://localhost/", "https://printer.local/"):
            with self.subTest(target=target):
                self.assertIsNone(
                    handler.redirect_request(request, None, 302, "Found", {}, target)
                )

    def test_rejects_https_to_http_downgrade(self):
        handler = builder._SafeCatalogRedirectHandler("example.org")
        request = builder.urllib.request.Request("https://example.org/start", method="HEAD")
        self.assertIsNone(
            handler.redirect_request(request, None, 302, "Found", {}, "http://example.org/insecure")
        )

    def test_allows_same_domain_https_redirect(self):
        handler = builder._SafeCatalogRedirectHandler("example.org")
        request = builder.urllib.request.Request("https://example.org/start", method="HEAD")
        redirected = handler.redirect_request(
            request, None, 301, "Moved Permanently", {}, "https://www.example.org/final"
        )
        self.assertIsNotNone(redirected)
        self.assertEqual(redirected.full_url, "https://www.example.org/final")

    def test_rejects_hostname_that_resolves_as_ip_literal_syntax(self):
        for url in ("https://127.0.0.1/", "https://[::1]/", "https://example..org/"):
            with self.subTest(url=url):
                self.assertFalse(builder.check_url(url)["reachable"])


if __name__ == "__main__":
    unittest.main()
