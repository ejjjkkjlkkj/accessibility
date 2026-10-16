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
from urllib.parse import urlparse

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "data" / "resources.csv"
OUT = ROOT / "database"
OUT.mkdir(parents=True, exist_ok=True)
DB = OUT / "accessibility_resources.sqlite"
JSON_OUT = OUT / "resource_status.json"
CSV_OUT = OUT / "resource_status.csv"
TIMEOUT = 12
USER_AGENT = "accessibility-resource-catalog/1.0 (read-only monitoring)"
CHECKED_AT = datetime.now(timezone.utc).isoformat(timespec="seconds")


def check_url(url: str) -> dict:
    parsed = urlparse(url)
    result = {
        "http_status": None,
        "final_url": None,
        "reachable": False,
        "tls": parsed.scheme.lower() == "https",
        "error": None,
    }
    try:
        port = parsed.port
    except ValueError:
        result["error"] = "invalid_http_url"
        return result
    hostname = (parsed.hostname or "").rstrip(".").lower()
    # The catalog contract requires DNS hostnames, not IP literals or local
    # names. Reject userinfo to avoid confusing the visible authority with
    # the actual destination.
    try:
        ipaddress.ip_address(hostname)
        is_ip_literal = bool(hostname)
    except ValueError:
        is_ip_literal = False
    if (
        parsed.scheme.lower() not in ("https", "http")
        or not hostname
        or parsed.username is not None
        or parsed.password is not None
        or is_ip_literal
        or hostname == "localhost"
        or hostname.endswith((".localhost", ".local", ".internal"))
        or (port is not None and not 1 <= port <= 65535)
    ):
        result["error"] = "invalid_http_url"
        return result
    request = urllib.request.Request(
        url, headers={"User-Agent": USER_AGENT}, method="HEAD"
    )
    try:
        with urllib.request.urlopen(
            request, timeout=TIMEOUT, context=ssl.create_default_context()
        ) as response:
            result["http_status"] = response.status
            result["final_url"] = response.geturl()
            result["tls"] = urlparse(result["final_url"]).scheme.lower() == "https"
            result["reachable"] = 200 <= response.status < 400
    except urllib.error.HTTPError as exc:
        # Some sites reject HEAD; retry a bounded GET without downloading the body.
        result["http_status"] = exc.code
        try:
            request = urllib.request.Request(
                url, headers={"User-Agent": USER_AGENT, "Range": "bytes=0-0"}
            )
            with urllib.request.urlopen(
                request, timeout=TIMEOUT, context=ssl.create_default_context()
            ) as response:
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
    with SOURCE.open("r", encoding="utf-8-sig", newline="") as handle:
        resources = list(csv.DictReader(handle))
    if not resources:
        raise SystemExit("FAIL: source catalog is empty")
    ids = [row["id"] for row in resources]
    if len(ids) != len(set(ids)):
        raise SystemExit("FAIL: duplicate resource id")

    records = []
    for row in resources:
        status = check_url(row["url"])
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
