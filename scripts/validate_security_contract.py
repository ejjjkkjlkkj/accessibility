#!/usr/bin/env python3
"""Validate the repository's security-agent contract without network access."""
from __future__ import annotations

import argparse
import json
import os
import stat
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "agents" / "manifest.json"
MAX_MANIFEST_BYTES = 1024 * 1024
REQUIRED_TEAMS = {"DARK", "WHITE", "GREY", "QA", "INDEPENDENT_AUDIT", "CONTROL"}
REQUIRED_AGENT_IDS = {
    "DARK-LEAD", "DARK-UEFI", "DARK-BINARY", "DARK-FUZZ", "DARK-MEMORY",
    "DARK-SUPPLY", "WHITE-SEC", "WHITE-ACCESS", "GREY-RESEARCH", "QA-VERIFY",
    "AUDIT-INDEPENDENT", "ORCHESTRATOR",
}
REQUIRED_PASS_EVIDENCE = {
    "reproducible_command", "recorded_exit_code", "test_output",
    "scope_and_environment", "independent_review_for_critical_findings",
}
REQUIRED_BLOCKERS = {
    "critical_or_high_unresolved_finding", "missing_required_evidence",
    "failed_ci", "scope_violation",
}
REQUIRED_STATUSES = {
    "PASS", "FAIL", "PARTIAL", "BLOCKED", "NOT_VALIDATED", "UNPROVEN"
}


def fail(message: str) -> None:
    print(f"FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def unique_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
    """Reject duplicate JSON keys instead of silently accepting the last value."""
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON object key: {key}")
        result[key] = value
    return result


def reject_json_constant(value: str) -> None:
    """Reject NaN and Infinity, which are not valid JSON constants."""
    raise ValueError(f"non-standard JSON constant: {value}")


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
        if args.manifest.is_symlink():
            fail("manifest must not be a symbolic link")
        with args.manifest.open("rb") as stream:
            metadata = os.fstat(stream.fileno())
            if not stat.S_ISREG(metadata.st_mode):
                fail("manifest must be a regular file")
            if metadata.st_size > MAX_MANIFEST_BYTES:
                fail(f"manifest exceeds {MAX_MANIFEST_BYTES} bytes")
            raw = stream.read(MAX_MANIFEST_BYTES + 1)
        if len(raw) > MAX_MANIFEST_BYTES:
            fail(f"manifest exceeds {MAX_MANIFEST_BYTES} bytes")
        data = json.loads(
            raw.decode("utf-8"),
            object_pairs_hook=unique_object,
            parse_constant=reject_json_constant,
        )
    except OSError as exc:
        fail(f"cannot read manifest: {exc}")
    except UnicodeDecodeError as exc:
        fail(f"manifest is not valid UTF-8: {exc}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON at line {exc.lineno}, column {exc.colno}: {exc.msg}")
    except (ValueError, RecursionError) as exc:
        fail(f"invalid JSON: {exc}")

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
        if agent["id"] == "DARK-FUZZ":
            if not isinstance(limits, dict):
                fail("DARK-FUZZ must define bounded local limits")
            if limits.get("network") is not False:
                fail("DARK-FUZZ limits.network must be false")
            if limits.get("filesystem_scope") != "temporary test workspace":
                fail("DARK-FUZZ filesystem scope must be the temporary test workspace")
            duration = limits.get("max_runtime_seconds")
            if not isinstance(duration, int) or isinstance(duration, bool) or not 1 <= duration <= 120:
                fail("DARK-FUZZ max_runtime_seconds must be between 1 and 120")
        ids.append(agent["id"])
        teams.add(agent["team"])

    if len(ids) != len(set(ids)):
        fail("agent IDs must be unique")
    missing_agents = REQUIRED_AGENT_IDS - set(ids)
    if missing_agents:
        fail("missing required agent roles: " + ", ".join(sorted(missing_agents)))
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
    gate_lists = {}
    for field in ("pass_requires", "block_on"):
        values = gate.get(field)
        if not isinstance(values, list) or not values or not all(
            non_empty_string(value) for value in values
        ):
            fail(f"release_gate {field} must be a non-empty array of strings")
        gate_lists[field] = set(values)
    missing_evidence = REQUIRED_PASS_EVIDENCE - gate_lists["pass_requires"]
    if missing_evidence:
        fail("release_gate pass_requires is missing mandatory evidence: " + ", ".join(sorted(missing_evidence)))
    missing_blockers = REQUIRED_BLOCKERS - gate_lists["block_on"]
    if missing_blockers:
        fail("release_gate block_on is missing mandatory blockers: " + ", ".join(sorted(missing_blockers)))
    if not non_empty_string(gate.get("physical_hardware_proof")):
        fail("physical hardware evidence boundary must be explicit")

    print(f"PASS: valid contract; {len(agents)} agent roles; {len(teams)} teams")
    print("LIMIT: validates policy metadata only; does not run agents or prove security.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
