#!/usr/bin/env python3
"""Manufacture a deterministic diagnostic manifest for release-quality gate outputs."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any

REQUIRED_GATES = ("format", "clippy", "docs-benchmark")
SHA_RE = re.compile(r"^[0-9a-f]{40}$")


def digest(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def build_report(subject: str, outcomes_path: Path, diagnostics_dir: Path) -> dict[str, Any]:
    violations: list[dict[str, Any]] = []
    if not SHA_RE.fullmatch(subject):
        violations.append({"rule": "EXACT_SUBJECT_REQUIRED", "actual": subject})

    try:
        outcomes = json.loads(outcomes_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        outcomes = {}
        violations.append({"rule": "OUTCOMES_REQUIRED", "error": type(exc).__name__})

    gates: dict[str, Any] = {}
    for gate in REQUIRED_GATES:
        entry = outcomes.get(gate) if isinstance(outcomes, dict) else None
        if not isinstance(entry, dict):
            violations.append({"rule": "GATE_OUTCOME_REQUIRED", "gate": gate})
            continue
        exit_code = entry.get("exit_code")
        command = entry.get("command")
        if not isinstance(exit_code, int) or exit_code < 0:
            violations.append({"rule": "VALID_EXIT_CODE_REQUIRED", "gate": gate, "actual": exit_code})
        if not isinstance(command, str) or not command.strip():
            violations.append({"rule": "COMMAND_IDENTITY_REQUIRED", "gate": gate})

        log_path = diagnostics_dir / f"{gate}.log"
        if not log_path.is_file():
            violations.append({"rule": "DIAGNOSTIC_LOG_REQUIRED", "gate": gate})
            continue
        size = log_path.stat().st_size
        if exit_code != 0 and size == 0:
            violations.append({"rule": "FAILED_GATE_REQUIRES_DIAGNOSTIC_PAYLOAD", "gate": gate})
        gates[gate] = {
            "command": command,
            "exit_code": exit_code,
            "log_bytes": size,
            "log_sha256": digest(log_path),
        }

    admitted = not violations and set(gates) == set(REQUIRED_GATES)
    failing = sorted(g for g, v in gates.items() if v.get("exit_code") != 0)
    standing = "ALIVE" if admitted and not failing else "BUILD_BROKEN" if admitted else "REFUSED:GATE_DIAGNOSTICS_CONTRACT"
    return {
        "schema_version": "1.0.0",
        "subject_sha": subject,
        "admission": "ADMITTED" if admitted else "REFUSED",
        "standing": standing,
        "diagnostics_complete": admitted,
        "failing_gates": failing,
        "gates": gates,
        "violations": violations,
        "execution_observed": admitted,
        "actuation_performed": False,
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--subject", required=True)
    parser.add_argument("--outcomes", type=Path, required=True)
    parser.add_argument("--diagnostics-dir", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args(argv)
    report = build_report(args.subject, args.outcomes, args.diagnostics_dir)
    rendered = json.dumps(report, indent=2, sort_keys=True) + "\n"
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(rendered, encoding="utf-8")
    sys.stdout.write(rendered)
    return 0 if report["admission"] == "ADMITTED" else 2


if __name__ == "__main__":
    raise SystemExit(main())
