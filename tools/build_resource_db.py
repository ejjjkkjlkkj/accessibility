#!/usr/bin/env python3
"""Build a small SQLite resource catalog and perform conservative read-only URL checks."""
from __future__ import annotations

import csv
import json
import ipaddress
import os
import sqlite3
import ssl
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import urljoin, urlparse

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "data" / "resources.csv"
OUT = ROOT / "database"
DB = OUT / "accessibility_resources.sqlite"
JSON_OUT = OUT / "resource_status.json"
CSV_OUT = OUT / "resource_status.csv"
TIMEOUT = 12
USER_AGENT = "accessibility-resource-catalog/1.0 (read-only monitoring)"
CHECKED_AT = datetime.now(timezone.utc).isoformat(timespec="seconds")


def _dns_hostname_is_valid(hostname: str) -> bool:
    """Accept public-looking ASCII DNS names only; IDNs must be punycode."""
    host = hostname.rstrip(".").lower()
    if not host or len(host) > 253:
        return False
    try:
        ipaddress.ip_address(host)
        return False
    except ValueError:
        pass
    labels = host.split(".")
    if len(labels) < 2:
        return False
    return all(
        1 <= len(label) <= 63
        and label[0].isalnum()
        and label[-1].isalnum()
        and all(ch.isascii() and (ch.isalnum() or ch == "-") for ch in label)
        for label in labels
    )


def _catalog_url_error(url: str, allowed_domain: str | None = None) -> str | None:
    """Reject malformed, local, IP-literal, credentialed, and off-catalog URLs."""
    try:
        parsed = urlparse(url)
        port = parsed.port
    except ValueError:
        return "invalid_http_url"
    hostname = (parsed.hostname or "").rstrip(".").lower()
    if (
        parsed.scheme.lower() not in ("https", "http")
        or not _dns_hostname_is_valid(hostname)
        or parsed.username is not None
        or parsed.password is not None
        or hostname == "localhost"
        or hostname.endswith((".localhost", ".local", ".internal"))
        or (port is not None and not 1 <= port <= 65535)
    ):
        return "invalid_http_url"
    if allowed_domain:
        domain = allowed_domain.rstrip(".").lower()
        if not _dns_hostname_is_valid(domain):
            return "invalid_declared_domain"
        if hostname != domain and not hostname.endswith("." + domain):
            return "redirect_outside_declared_domain"
    return None


class _SafeCatalogRedirectHandler(urllib.request.HTTPRedirectHandler):
    """Keep redirects within the declared resource domain and prevent HTTPS downgrade."""

    def __init__(self, allowed_domain: str) -> None:
        super().__init__()
        self.allowed_domain = allowed_domain

    def redirect_request(self, req, fp, code, msg, headers, newurl):  # type: ignore[no-untyped-def]
        target = urljoin(req.full_url, newurl)
        if _catalog_url_error(target, self.allowed_domain):
            return None
        source_scheme = urlparse(req.full_url).scheme.lower()
        target_scheme = urlparse(target).scheme.lower()
        if source_scheme == "https" and target_scheme != "https":
            return None
        return super().redirect_request(req, fp, code, msg, headers, target)


def _open_catalog_url(request, timeout: int, allowed_domain: str):  # type: ignore[no-untyped-def]
    """Open URLs through the redirect policy and normal system TLS verification."""
    opener = urllib.request.build_opener(
        urllib.request.HTTPSHandler(context=ssl.create_default_context()),
        _SafeCatalogRedirectHandler(allowed_domain),
    )
    return opener.open(request, timeout=timeout)


def check_url(url: str, expected_domain: str | None = None) -> dict:
    try:
        parsed = urlparse(url)
    except ValueError:
        return {
            "http_status": None,
            "final_url": None,
            "reachable": False,
            "tls": False,
            "error": "invalid_http_url",
        }
    result = {
        "http_status": None,
        "final_url": None,
        "reachable": False,
        "tls": parsed.scheme.lower() == "https",
        "error": None,
    }
    allowed_domain = (expected_domain or parsed.hostname or "").rstrip(".").lower()
    validation_error = _catalog_url_error(url, allowed_domain)
    if validation_error:
        result["error"] = validation_error
        return result
    request = urllib.request.Request(
        url, headers={"User-Agent": USER_AGENT}, method="HEAD"
    )
    try:
        with _open_catalog_url(request, TIMEOUT, allowed_domain) as response:
            result["http_status"] = response.status
            result["final_url"] = response.geturl()
            result["tls"] = urlparse(result["final_url"]).scheme.lower() == "https"
            result["reachable"] = 200 <= response.status < 400
    except urllib.error.HTTPError as exc:
        # Retry only errors commonly caused by HEAD restrictions, not arbitrary failures.
        result["http_status"] = exc.code
        if exc.code not in (403, 405, 501):
            result["error"] = f"http_{exc.code}"
            return result
        try:
            request = urllib.request.Request(
                url, headers={"User-Agent": USER_AGENT, "Range": "bytes=0-0"}
            )
            with _open_catalog_url(request, TIMEOUT, allowed_domain) as response:
                result["http_status"] = response.status
                result["final_url"] = response.geturl()
                result["tls"] = urlparse(result["final_url"]).scheme.lower() == "https"
                result["reachable"] = 200 <= response.status < 400
        except Exception as retry_exc:
            result["error"] = f"http_{exc.code}; retry_{type(retry_exc).__name__}"
    except Exception as exc:
        result["error"] = type(exc).__name__
    return result


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    with SOURCE.open("r", encoding="utf-8-sig", newline="") as handle:
        resources = list(csv.DictReader(handle))
    if not resources:
        raise SystemExit("FAIL: source catalog is empty")
    ids = [row["id"] for row in resources]
    if len(ids) != len(set(ids)):
        raise SystemExit("FAIL: duplicate resource id")

    records = []
    for row in resources:
        status = check_url(row["url"], row["domain"])
        records.append({**row, **status, "checked_at": CHECKED_AT})
        print(
            f"{'PASS' if status['reachable'] else 'PARTIAL'} "
            f"{row['id']}: HTTP={status['http_status']} "
            f"reachable={status['reachable']} error={status['error']}"
        )
        time.sleep(0.15)

    with sqlite3.connect(DB) as conn:
        conn.execute("PRAGMA journal_mode=DELETE")
        conn.execute("DROP TABLE IF EXISTS resources")
        conn.execute("""
            CREATE TABLE resources (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                domain TEXT NOT NULL,
                url TEXT NOT NULL,
                category TEXT NOT NULL,
                layer TEXT NOT NULL,
                official_status TEXT NOT NULL,
                notes TEXT NOT NULL,
                http_status INTEGER,
                final_url TEXT,
                reachable INTEGER NOT NULL,
                tls INTEGER NOT NULL,
                error TEXT,
                checked_at TEXT NOT NULL
            )
        """)
        conn.execute("CREATE INDEX idx_resources_layer ON resources(layer)")
        conn.execute("CREATE INDEX idx_resources_domain ON resources(domain)")
        conn.executemany("""
            INSERT INTO resources VALUES (
                :id, :name, :domain, :url, :category, :layer, :official_status,
                :notes, :http_status, :final_url, :reachable, :tls, :error, :checked_at
            )
        """, [
            {**r, "reachable": int(r["reachable"]), "tls": int(r["tls"])}
            for r in records
        ])
        conn.execute("CREATE TABLE metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
        conn.executemany("INSERT INTO metadata VALUES (?, ?)", [
            ("checked_at", CHECKED_AT),
            ("source", "data/resources.csv"),
            ("meaning", "HTTP reachability is not proof of domain ownership or official status"),
        ])
        conn.commit()

    JSON_OUT.write_text(
        json.dumps({
            "checked_at": CHECKED_AT,
            "resource_count": len(records),
            "reachable_count": sum(r["reachable"] for r in records),
            "unreachable_or_unverified_count": sum(not r["reachable"] for r in records),
            "warning": "Reachability, TLS and redirects do not establish domain ownership or official status.",
            "resources": records,
        }, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    fieldnames = list(records[0].keys())
    with CSV_OUT.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(records)

    print(f"PASS: catalog resources={len(records)}")
    print(f"DB: {DB.relative_to(ROOT)}")
    print(f"JSON: {JSON_OUT.relative_to(ROOT)}")
    print(f"CSV: {CSV_OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
