#!/usr/bin/env python3
"""Refuse unfinished implementation debt on admitted repository surfaces."""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable

SCAN_ROOTS = (
    "src",
    "clap-noun-verb-macros/src",
    "utils/src",
    "examples",
    "benches",
    "tests",
    "scripts",
    ".github/workflows",
    "packs",
)

TEXT_SUFFIXES = {
    ".rs",
    ".py",
    ".sh",
    ".toml",
    ".yml",
    ".yaml",
    ".ts",
    ".tsx",
    ".js",
    ".mjs",
    ".cjs",
    ".rq",
    ".ttl",
    ".tera",
}

SELF = Path("scripts/verify_no_wip.py")

MARKER_RULES = (
    ("TODO_MARKER", re.compile(r"\bTO" + r"DO\b")),
    ("FIXME_MARKER", re.compile(r"\bFIX" + r"ME\b")),
    ("WIP_MARKER", re.compile(r"\bW" + r"IP\b")),
    ("TODO_MACRO", re.compile(r"\bto" + r"do!\s*\(")),
    ("UNIMPLEMENTED_MACRO", re.compile(r"\bun" + r"implemented!\s*\(")),
    (
        "DISABLED_WORKFLOW",
        re.compile(
            r"^\s*if:\s*(?:\$\{\{\s*)?false(?:\s*\}\})?\s*$",
            re.MULTILINE,
        ),
    ),
)

EMPTY_MAIN = re.compile(r"fn\s+main\s*\(\s*\)\s*\{\s*\}", re.DOTALL)


@dataclass(frozen=True)
class Violation:
    rule: str
    path: str
    line: int
    snippet: str


def iter_admitted_files(root: Path) -> Iterable[Path]:
    for relative in SCAN_ROOTS:
        surface = root / relative
        if not surface.exists():
            continue
        for path in surface.rglob("*"):
            if not path.is_file() or path.suffix not in TEXT_SUFFIXES:
                continue
            rel = path.relative_to(root)
            if rel == SELF or any(
                part in {"target", "node_modules", "vendor", "vendors"} for part in rel.parts
            ):
                continue
            yield path


def line_number(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def scan_file(root: Path, path: Path) -> list[Violation]:
    text = path.read_text(encoding="utf-8", errors="replace")
    relative = path.relative_to(root).as_posix()
    violations: list[Violation] = []

    for rule, pattern in MARKER_RULES:
        for match in pattern.finditer(text):
            start = text.rfind("\n", 0, match.start()) + 1
            end = text.find("\n", match.end())
            if end < 0:
                end = len(text)
            violations.append(
                Violation(
                    rule,
                    relative,
                    line_number(text, match.start()),
                    text[start:end].strip()[:240],
                )
            )

    if relative.startswith("examples/"):
        for match in EMPTY_MAIN.finditer(text):
            violations.append(
                Violation(
                    "EMPTY_EXAMPLE_MAIN",
                    relative,
                    line_number(text, match.start()),
                    "fn main() {}",
                )
            )

    return violations


def verify(root: Path) -> dict[str, object]:
    files = sorted(iter_admitted_files(root))
    violations = [violation for path in files for violation in scan_file(root, path)]
    return {
        "schema_version": "1.0.0",
        "admission": "ADMITTED" if not violations else "REFUSED",
        "standing": "PARTIAL_ALIVE" if not violations else "BUILD_BROKEN",
        "files_scanned": len(files),
        "violation_count": len(violations),
        "violations": [asdict(item) for item in violations],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()

    report = verify(args.root.resolve())
    rendered = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(rendered, encoding="utf-8")
    sys.stdout.write(rendered)
    return 0 if report["admission"] == "ADMITTED" else 1


if __name__ == "__main__":
    raise SystemExit(main())
