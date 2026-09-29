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

# A marker word appearing inside a Rust string or raw-string literal -- e.g. a
# code generator's template text, or an example error message quoting "TODO" as
# sample placeholder text -- describes data, not an admission of unfinished work
# in *this* repository's own source. Real markers in real code or comments must
# still be refused, so blanking is done with a small hand-rolled scanner rather
# than a single regex: Rust raw strings (`r#"..."#` with an arbitrary, matching
# hash count) and escapes inside normal strings (`\"`, `\\`) are not reliably
# expressible as a single non-backtracking regex without risking exactly the
# false "raw string start" match a stray `r"` substring inside an unrelated
# normal string (e.g. the word "later" immediately before a closing quote) would
# trigger.
def blank_string_literals(text: str) -> str:
    """Replace the contents of Rust string/raw-string literals with spaces,
    preserving line breaks and overall length so line numbers and snippet
    extraction against the original text stay accurate. Comments and real code
    are left untouched."""

    out = list(text)
    i = 0
    n = len(text)

    def blank_range(start: int, end: int) -> None:
        for idx in range(start, end):
            if out[idx] != "\n":
                out[idx] = " "

    while i < n:
        ch = text[i]

        # Line comment: skip to end of line untouched.
        if ch == "/" and i + 1 < n and text[i + 1] == "/":
            i = text.find("\n", i)
            i = n if i < 0 else i
            continue

        # Block comment: skip to closing */ untouched (non-nesting).
        if ch == "/" and i + 1 < n and text[i + 1] == "*":
            end = text.find("*/", i + 2)
            i = n if end < 0 else end + 2
            continue

        # Raw string: r, br, or Rb followed by zero-or-more # then a quote.
        if ch in "rb":
            j = i
            if text[j] == "b" and j + 1 < n and text[j + 1] == "r":
                j += 1
            if j < n and text[j] == "r":
                k = j + 1
                hashes = 0
                while k < n and text[k] == "#":
                    hashes += 1
                    k += 1
                if k < n and text[k] == '"':
                    content_start = k + 1
                    terminator = '"' + ("#" * hashes)
                    end = text.find(terminator, content_start)
                    if end < 0:
                        end = n
                        blank_range(content_start, end)
                        i = end
                    else:
                        blank_range(content_start, end)
                        i = end + len(terminator)
                    continue

        # Normal (possibly byte) string: "..." with \x escapes.
        if ch == '"' or (ch == "b" and i + 1 < n and text[i + 1] == '"'):
            start = i + (1 if ch == "b" else 0)
            content_start = start + 1
            j = content_start
            while j < n:
                if text[j] == "\\" and j + 1 < n:
                    j += 2
                    continue
                if text[j] == '"':
                    break
                j += 1
            blank_range(content_start, j)
            i = j + 1 if j < n else n
            continue

        i += 1

    return "".join(out)


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


def blank_span(chars: list[str], start: int, end: int) -> None:
    """Blank a lexical span without moving offsets or line numbers."""
    for index in range(start, end):
        if chars[index] != "\n":
            chars[index] = " "


def rust_code_view(text: str) -> str:
    """Hide Rust string payloads while preserving source coordinates.

    The verifier admits executable macros and comments, not parser fixtures or
    validator examples embedded inside Rust strings. Raw and ordinary strings
    are removed conservatively while comments remain visible.
    """
    chars = list(text)
    length = len(text)
    index = 0

    while index < length:
        raw = re.match(r'(?:br|r)(?P<hashes>#{0,255})"', text[index:])
        if raw is not None:
            hashes = raw.group("hashes")
            start = index
            payload = index + raw.end()
            terminator = '"' + hashes
            close = text.find(terminator, payload)
            end = length if close < 0 else close + len(terminator)
            blank_span(chars, start, end)
            index = end
            continue

        prefix = 1 if text[index : index + 2] == 'b"' else 0
        if text[index + prefix : index + prefix + 1] == '"':
            start = index
            cursor = index + prefix + 1
            escaped = False
            while cursor < length:
                character = text[cursor]
                if escaped:
                    escaped = False
                elif character == "\\":
                    escaped = True
                elif character == '"':
                    cursor += 1
                    break
                cursor += 1
            blank_span(chars, start, cursor)
            index = cursor
            continue

        index += 1

    return "".join(chars)


def marker_view(path: Path, text: str) -> str:
    return rust_code_view(text) if path.suffix == ".rs" else text


def scan_file(root: Path, path: Path) -> list[Violation]:
    text = path.read_text(encoding="utf-8", errors="replace")
    scan_text = marker_view(path, text)
    relative = path.relative_to(root).as_posix()
    violations: list[Violation] = []

    marker_text = blank_string_literals(text) if path.suffix == ".rs" else text
    for rule, pattern in MARKER_RULES:
        for match in pattern.finditer(marker_text):
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
        for match in EMPTY_MAIN.finditer(scan_text):
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
        "schema_version": "1.1.0",
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
