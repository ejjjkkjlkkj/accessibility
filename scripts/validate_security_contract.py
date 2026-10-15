#!/usr/bin/env python3
"""Validate the repository's security-agent contract without network access."""
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "agents" / "manifest.json"
REQUIRED_TEAMS = {"DARK", "WHITE", "GREY", "QA", "INDEPENDENT_AUDIT", "CONTROL"}
REQUIRED_STATUSES = {
    "PASS", "FAIL", "PARTIAL", "BLOCKED", "NOT_VALIDATED", "UNPROVEN"
}


def fail(message: str) -> None:
    print(f"FAIL: {message}")
    raise SystemExit(1)


def main() -> int:
    try:
        data = json.loads(MANIFEST.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing manifest: {MANIFEST.relative_to(ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON at line {exc.lineno}, column {exc.colno}: {exc.msg}")

    if data.get("schema_version") != "1.0.0":
        fail("schema_version must be 1.0.0")
    if data.get("status") != "CONTRACT_ONLY_NOT_AUTONOMOUS":
        fail("manifest must not claim unimplemented agent execution")
    rules = data.get("operating_rules")
    if not isinstance(rules, dict):
        fail("operating_rules must be an object")
    if rules.get("external_targets") != "PROHIBITED":
        fail("external target testing must be prohibited")
    if rules.get("destructive_actions") != "PROHIBITED":
        fail("destructive actions must be prohibited")
    if rules.get("secrets") is None or rules.get("evidence") is None:
        fail("secret-handling and evidence policies are required")

    agents = data.get("agents")
    if not isinstance(agents, list) or not agents:
        fail("agents must be a non-empty array")
    ids = []
    teams = set()
    for index, agent in enumerate(agents):
        if not isinstance(agent, dict):
            fail(f"agents[{index}] must be an object")
        for field in ("id", "team", "specialty", "capabilities"):
            if field not in agent:
                fail(f"agents[{index}] missing {field}")
        if not isinstance(agent["capabilities"], list):
            fail(f"agents[{index}].capabilities must be an array")
        ids.append(agent["id"])
        teams.add(agent["team"])
    if len(ids) != len(set(ids)):
        fail("agent IDs must be unique")
    missing_teams = REQUIRED_TEAMS - teams
    if missing_teams:
        fail("missing teams: " + ", ".join(sorted(missing_teams)))

    gate = data.get("release_gate")
    if not isinstance(gate, dict):
        fail("release_gate must be an object")
    statuses = set(gate.get("required_statuses", []))
    if statuses != REQUIRED_STATUSES:
        fail("release_gate required_statuses do not match the approved vocabulary")
    if not gate.get("pass_requires") or not gate.get("block_on"):
        fail("release gate must define evidence requirements and blocking conditions")
    if not gate.get("physical_hardware_proof"):
        fail("physical hardware evidence boundary must be explicit")

    print(f"PASS: valid contract; {len(agents)} agent roles; {len(teams)} teams")
    print("LIMIT: validates policy metadata only; does not run agents or prove security.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
