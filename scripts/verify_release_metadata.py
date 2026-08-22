#!/usr/bin/env python3
"""Fail closed when the macros crate's public release metadata drifts."""

from __future__ import annotations

import argparse
import json
import sys
import tomllib
from pathlib import Path
from typing import Any

EXPECTED_PACKAGE: dict[str, Any] = {
    "name": "clap-noun-verb-macros",
    "version": "26.9.1",
    "edition": "2021",
    "rust-version": "1.70",
    "license": "MIT OR Apache-2.0",
    "description": (
        "Procedural macros for clap-noun-verb - attribute macros for "
        "zero-boilerplate CLI command registration"
    ),
    "repository": "https://github.com/seanchatmangpt/clap-noun-verb",
    "documentation": "https://docs.rs/clap-noun-verb",
    "homepage": "https://github.com/seanchatmangpt/clap-noun-verb",
    "keywords": ["cli", "clap", "noun-verb", "command-line", "proc-macro"],
    "categories": ["command-line-utilities", "development-tools"],
}
FORBIDDEN_DEPENDENCIES = {"proc-macro-error"}


def verify_manifest(path: Path) -> dict[str, Any]:
    with path.open("rb") as source:
        manifest = tomllib.load(source)

    package = manifest.get("package")
    dependencies = manifest.get("dependencies")
    violations: list[dict[str, Any]] = []

    if not isinstance(package, dict):
        violations.append({"rule": "PACKAGE_TABLE_REQUIRED", "actual": package})
        package = {}
    if not isinstance(dependencies, dict):
        violations.append({"rule": "DEPENDENCIES_TABLE_REQUIRED", "actual": dependencies})
        dependencies = {}

    for key, expected in EXPECTED_PACKAGE.items():
        actual = package.get(key)
        if actual != expected:
            violations.append(
                {
                    "rule": "RELEASE_METADATA_DRIFT",
                    "field": key,
                    "expected": expected,
                    "actual": actual,
                }
            )

    forbidden_present = sorted(FORBIDDEN_DEPENDENCIES.intersection(dependencies))
    if forbidden_present:
        violations.append(
            {
                "rule": "FORBIDDEN_UNMAINTAINED_DEPENDENCY",
                "dependencies": forbidden_present,
            }
        )

    admitted = not violations
    return {
        "schema_version": "1.0.0",
        "admission": "ADMITTED" if admitted else "REFUSED",
        "standing": "PARTIAL_ALIVE" if admitted else "REFUSED:RELEASE_METADATA_CONTRACT",
        "manifest": path.as_posix(),
        "checks": {
            "required_metadata_fields": len(EXPECTED_PACKAGE),
            "forbidden_dependencies": sorted(FORBIDDEN_DEPENDENCIES),
        },
        "violations": violations,
        "actuation_performed": False,
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--manifest",
        type=Path,
        default=Path("clap-noun-verb-macros/Cargo.toml"),
    )
    parser.add_argument("--report", type=Path)
    args = parser.parse_args(argv)

    report = verify_manifest(args.manifest)
    rendered = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.report is not None:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(rendered, encoding="utf-8")
    sys.stdout.write(rendered)
    return 0 if report["admission"] == "ADMITTED" else 1


if __name__ == "__main__":
    raise SystemExit(main())
