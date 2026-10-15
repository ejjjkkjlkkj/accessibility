#!/usr/bin/env python3
"""Validate the repository's security-agent contract without network access."""
from __future__ import annotations

import argparse
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
    print(f"FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def non_empty_string(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip())


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--manifest",
        type=Path,
        default=MANIFEST,
        help="manifest JSON to validate (defaults to agents/manifest.json)",
    )
    args = parser.parse_args()

    try:
        data = json.loads(args.manifest.read_text(encoding="utf-8"))
    except OSError as exc:
        fail(f"cannot read manifest: {exc}")
    except UnicodeDecodeError as exc:
        fail(f"manifest is not valid UTF-8: {exc}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON at line {exc.lineno}, column {exc.colno}: {exc.msg}")

    if not isinstance(data, dict):
        fail("manifest root must be a JSON object")
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
    for field in ("authorization", "secrets", "evidence", "no_false_claims"):
        if not non_empty_string(rules.get(field)):
            fail(f"operating_rules.{field} must be a non-empty string")
    if not non_empty_string(rules.get("firmware_nvram_bios_reboot")):
        fail("firmware/NVRAM/BIOS/reboot policy must be explicit")

    agents = data.get("agents")
    if not isinstance(agents, list) or not agents:
        fail("agents must be a non-empty array")

    ids: list[str] = []
    teams: set[str] = set()
    for index, agent in enumerate(agents):
        if not isinstance(agent, dict):
            fail(f"agents[{index}] must be an object")
        for field in ("id", "team", "specialty", "capabilities"):
            if field not in agent:
                fail(f"agents[{index}] missing {field}")
        for field in ("id", "team", "specialty"):
            if not non_empty_string(agent[field]):
                fail(f"agents[{index}].{field} must be a non-empty string")
        if agent["team"] not in REQUIRED_TEAMS:
            fail(f"agents[{index}].team is not an approved team")
        capabilities = agent["capabilities"]
        if not isinstance(capabilities, list) or not capabilities:
            fail(f"agents[{index}].capabilities must be a non-empty array")
        if not all(non_empty_string(capability) for capability in capabilities):
            fail(f"agents[{index}].capabilities entries must be non-empty strings")
        for optional_list in ("forbidden", "requirements"):
            if optional_list in agent:
                values = agent[optional_list]
                if not isinstance(values, list) or not all(non_empty_string(value) for value in values):
                    fail(f"agents[{index}].{optional_list} must contain only non-empty strings")
        if "limits" in agent and not isinstance(agent["limits"], dict):
            fail(f"agents[{index}].limits must be an object")
        limits = agent.get("limits")
        if isinstance(limits, dict):
            if "network" in limits and not isinstance(limits["network"], bool):
                fail(f"agents[{index}].limits.network must be a boolean")
            if "filesystem_scope" in limits and not non_empty_string(limits["filesystem_scope"]):
                fail(f"agents[{index}].limits.filesystem_scope must be a non-empty string")
            if "max_runtime_seconds" in limits:
                duration = limits["max_runtime_seconds"]
                if not isinstance(duration, int) or isinstance(duration, bool) or duration < 1:
                    fail(f"agents[{index}].limits.max_runtime_seconds must be a positive integer")
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
    statuses_value = gate.get("required_statuses")
    if not isinstance(statuses_value, list) or not all(
        isinstance(status, str) for status in statuses_value
    ):
        fail("release_gate required_statuses must be an array of strings")
    if set(statuses_value) != REQUIRED_STATUSES:
        fail("release_gate required_statuses do not match the approved vocabulary")
    for field in ("pass_requires", "block_on"):
        values = gate.get(field)
        if not isinstance(values, list) or not values or not all(
            non_empty_string(value) for value in values
        ):
            fail(f"release_gate {field} must be a non-empty array of strings")
    if not non_empty_string(gate.get("physical_hardware_proof")):
        fail("physical hardware evidence boundary must be explicit")

    print(f"PASS: valid contract; {len(agents)} agent roles; {len(teams)} teams")
    print("LIMIT: validates policy metadata only; does not run agents or prove security.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
