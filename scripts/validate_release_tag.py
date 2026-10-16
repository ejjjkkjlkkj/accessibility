#!/usr/bin/env python3
"""Validate release tags without evaluating shell syntax."""
from __future__ import annotations

import re
import sys

# SemVer 2.0.0 with a required v prefix. Build metadata is intentionally omitted.
_NUMERIC = r"(?:0|[1-9][0-9]*)"
_PRERELEASE_IDENTIFIER = r"(?:0|[1-9][0-9]*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*)"
_RELEASE_TAG = re.compile(
    rf"v{_NUMERIC}\.{_NUMERIC}\.{_NUMERIC}"
    rf"(?:-(?:{_PRERELEASE_IDENTIFIER})(?:\.{_PRERELEASE_IDENTIFIER})*)?"
)


def validate_release_tag(value: str) -> str:
    """Return the original tag if valid; otherwise raise ValueError."""
    if not isinstance(value, str) or not _RELEASE_TAG.fullmatch(value):
        raise ValueError(
            "release tag must be vMAJOR.MINOR.PATCH with an optional valid "
            "SemVer prerelease (for example v0.1.0-alpha.1)"
        )
    return value


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: validate_release_tag.py TAG", file=sys.stderr)
        return 2
    try:
        print(validate_release_tag(sys.argv[1]))
    except ValueError as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
