#!/usr/bin/env python3
"""Fail-closed static verifier for the clap-noun-verb ggen pack contract.

This verifier does not issue an execution receipt. It validates the authored
contract before ggen crosses the filesystem write boundary. ggen remains the
only producer and verifier of `.ggen-v2` receipts.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
from dataclasses import dataclass, asdict
from pathlib import Path, PurePosixPath
from typing import Any

PINNED_GGEN_SHA = "68952593c40214ac1a681073d65f3902a9cdfce4"
CANONICAL_PROJECT = "clap-noun-verb"
CANONICAL_VOCABULARY = "http://clap-noun-verb.io/ontology#"
ALLOWED_OUTPUT_PREFIXES = ("src/verbs/",)
REQUIRED_QUERY_COLUMNS = {
    "noun_name",
    "verb_name",
    "verb_about",
    "return_type",
    "handler_name",
    "args",
}


class ContractError(RuntimeError):
    """One typed contract refusal."""

    def __init__(self, code: str, message: str) -> None:
        super().__init__(f"{code}: {message}")
        self.code = code
        self.message = message


@dataclass(frozen=True)
class VerificationReport:
    state: str
    project: str
    version: str
    generation_rules: int
    validation_gates: int
    pinned_ggen_sha: str
    checked_surfaces: tuple[str, ...]


def load_toml(path: Path) -> dict[str, Any]:
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise ContractError("GGEN_CONTRACT_MISSING", str(path)) from exc
    except tomllib.TOMLDecodeError as exc:
        raise ContractError("GGEN_CONTRACT_TOML_INVALID", f"{path}: {exc}") from exc


def require_file(root: Path, relative: str) -> Path:
    path = root / relative
    if not path.is_file():
        raise ContractError("GGEN_CONTRACT_INPUT_MISSING", relative)
    return path


def bounded_output(raw: str) -> None:
    path = PurePosixPath(raw)
    if path.is_absolute() or ".." in path.parts:
        raise ContractError("GGEN_OUTPUT_ESCAPE_REFUSED", raw)
    normalized = path.as_posix()
    if not any(normalized.startswith(prefix) for prefix in ALLOWED_OUTPUT_PREFIXES):
        raise ContractError("GGEN_OUTPUT_AUTHORITY_REFUSED", normalized)


def select_columns(query: str) -> set[str]:
    match = re.search(r"\bSELECT\b(?P<body>.*?)\bWHERE\b", query, flags=re.I | re.S)
    if match is None:
        raise ContractError("GGEN_QUERY_SELECT_REQUIRED", "verb-signatures.rq")
    body = re.sub(r"(?m)^\s*#.*$", "", match.group("body"))
    return set(re.findall(r"\?([A-Za-z_][A-Za-z0-9_]*)", body))


def ask_body(text: str) -> str:
    match = re.search(r"\bASK\s*\{(?P<body>.*)\}\s*$", text, flags=re.I | re.S)
    if match is None:
        raise ContractError("GGEN_COLLISION_ASK_REQUIRED", "field collision gate")
    body = re.sub(r"(?m)^\s*#.*$", "", match.group("body"))
    replacements = {
        "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>": "rdf:type",
        f"<{CANONICAL_VOCABULARY}Verb>": "cnv:Verb",
        f"<{CANONICAL_VOCABULARY}hasArguments>": "cnv:hasArguments",
        f"<{CANONICAL_VOCABULARY}hasArgumentName>": "cnv:hasArgumentName",
        f"<{CANONICAL_VOCABULARY}fieldName>": "cnv:fieldName",
    }
    for source, target in replacements.items():
        body = body.replace(source, target)
    body = re.sub(r"(?<![A-Za-z0-9_])a\s+cnv:Verb", "rdf:type cnv:Verb", body)
    return re.sub(r"\s+", " ", body).strip()


def verify(root: Path) -> VerificationReport:
    root = root.resolve()
    manifest = load_toml(root / "ggen.toml")
    cargo = load_toml(root / "Cargo.toml")
    package = load_toml(root / "package.toml")
    example = load_toml(root / "examples/greet-demo/ggen.toml")

    project = manifest.get("project", {})
    cargo_package = cargo.get("package", {})
    if project.get("name") != CANONICAL_PROJECT:
        raise ContractError("GGEN_PROJECT_IDENTITY_REFUSED", str(project.get("name")))
    if cargo_package.get("name") != CANONICAL_PROJECT:
        raise ContractError("GGEN_CARGO_IDENTITY_REFUSED", str(cargo_package.get("name")))
    if project.get("version") != cargo_package.get("version"):
        raise ContractError(
            "GGEN_VERSION_DRIFT_REFUSED",
            f"ggen={project.get('version')} cargo={cargo_package.get('version')}",
        )

    ontology = manifest.get("ontology", {})
    require_file(root, str(ontology.get("source", "")))
    prefixes = ontology.get("prefixes", {})
    if prefixes.get("cnv") != CANONICAL_VOCABULARY:
        raise ContractError("GGEN_VOCABULARY_DRIFT_REFUSED", str(prefixes.get("cnv")))

    pack = package.get("pack", {})
    if pack.get("name") != CANONICAL_PROJECT:
        raise ContractError("GGEN_PACK_IDENTITY_REFUSED", str(pack.get("name")))
    if set(pack) != {"name", "outputs"}:
        raise ContractError(
            "GGEN_TRANSPORT_AUTHORITY_LEAK_REFUSED",
            "package.toml may declare only pack name and outputs",
        )
    for surface in package.get("pack", {}).get("outputs", {}).values():
        path = root / str(surface)
        if not path.is_dir():
            raise ContractError("GGEN_PACK_OUTPUT_MISSING", str(surface))

    generation = manifest.get("generation", {})
    rules = generation.get("rules", [])
    if not rules:
        raise ContractError("GGEN_GENERATION_RULE_REQUIRED", "no generation rules")
    names: set[str] = set()
    for rule in rules:
        name = str(rule.get("name", ""))
        if not name or name in names:
            raise ContractError("GGEN_GENERATION_RULE_ID_REFUSED", name)
        names.add(name)
        query = str(rule.get("query", {}).get("file", ""))
        template = str(rule.get("template", {}).get("file", ""))
        require_file(root, query)
        require_file(root, template)
        bounded_output(str(rule.get("output_file", "")))
        if rule.get("mode") != "Overwrite":
            raise ContractError("GGEN_WRITE_MODE_REFUSED", f"{name}: {rule.get('mode')}")
        if rule.get("skip_empty") is not True:
            raise ContractError("GGEN_EMPTY_PROJECTION_REFUSED", name)

    query_text = require_file(root, "queries/verb-signatures.rq").read_text(encoding="utf-8")
    missing_columns = REQUIRED_QUERY_COLUMNS - select_columns(query_text)
    if missing_columns:
        raise ContractError("GGEN_QUERY_TEMPLATE_CLOSURE_REFUSED", ",".join(sorted(missing_columns)))

    template_text = require_file(root, "templates/verb.rs.tera").read_text(encoding="utf-8")
    if "rendered from O* by ggen" not in template_text:
        raise ContractError("GGEN_GENERATED_BANNER_REQUIRED", "templates/verb.rs.tera")
    for column in REQUIRED_QUERY_COLUMNS:
        if column not in template_text:
            raise ContractError("GGEN_QUERY_TEMPLATE_CLOSURE_REFUSED", column)

    canonical_gate_path = require_file(root, "gates/fieldname-collision.rq")
    canonical_gate = canonical_gate_path.read_text(encoding="utf-8")
    if not re.search(r"(?m)^# MESSAGE: \S", canonical_gate):
        raise ContractError("GGEN_GATE_MESSAGE_REQUIRED", "gates/fieldname-collision.rq")
    canonical_body = ask_body(canonical_gate)
    if "FILTER NOT EXISTS" in canonical_body.upper():
        raise ContractError("GGEN_GATE_POLARITY_REFUSED", "canonical collision ASK is inverted")

    root_validation = manifest.get("validation", {})
    if root_validation.get("rules"):
        raise ContractError("GGEN_INLINE_GATE_AUTHORITY_REFUSED", "root manifest")
    root_gates = root_validation.get("gates", [])
    if root_gates != ["gates/fieldname-collision.rq"]:
        raise ContractError("GGEN_COLLISION_GATE_CARDINALITY_REFUSED", str(root_gates))

    example_rules = example.get("generation", {}).get("rules", [])
    if len(example_rules) != len(rules):
        raise ContractError("GGEN_EXAMPLE_RULE_DRIFT_REFUSED", str(len(example_rules)))
    for rule in example_rules:
        if rule.get("skip_empty") is not True:
            raise ContractError("GGEN_EMPTY_PROJECTION_REFUSED", f"example:{rule.get('name')}")
    example_validation = example.get("validation", {})
    if example_validation.get("rules"):
        raise ContractError("GGEN_INLINE_GATE_AUTHORITY_REFUSED", "greet-demo")
    example_gates = example_validation.get("gates", [])
    if example_gates != ["../../gates/fieldname-collision.rq"]:
        raise ContractError("GGEN_COLLISION_GATE_CARDINALITY_REFUSED", str(example_gates))
    example_gate_path = (root / "examples/greet-demo" / example_gates[0]).resolve()
    if example_gate_path != canonical_gate_path.resolve():
        raise ContractError("GGEN_COLLISION_GATE_DRIFT_REFUSED", "greet-demo")

    compatibility_marker = require_file(root, "ontology/queries/fieldname-collision.rq")
    marker_text = compatibility_marker.read_text(encoding="utf-8")
    if re.search(r"\b(?:ASK|SELECT)\b", marker_text, flags=re.I):
        raise ContractError("GGEN_DUPLICATE_GATE_AUTHORITY_REFUSED", str(compatibility_marker))

    generated = require_file(root, "examples/greet-demo/src/verbs/greet.rs")
    if "rendered from O* by ggen" not in generated.read_text(encoding="utf-8"):
        raise ContractError("HAND_CODED_GENERATED_OUTPUT_REFUSED", str(generated.relative_to(root)))

    workflow = require_file(root, ".github/workflows/ggen-authority.yml").read_text(
        encoding="utf-8"
    )
    if PINNED_GGEN_SHA not in workflow:
        raise ContractError("GGEN_ACTUATOR_PIN_REFUSED", PINNED_GGEN_SHA)
    for command in ("sync run", "receipt verify"):
        if command not in workflow:
            raise ContractError("GGEN_RECEIPT_REPLAY_GATE_REQUIRED", command)

    for required in ("AGENTS.md", "docs/GGEN_AUTHORITY.md"):
        require_file(root, required)

    return VerificationReport(
        state="PARTIAL_ALIVE",
        project=CANONICAL_PROJECT,
        version=str(project["version"]),
        generation_rules=len(rules),
        validation_gates=len(root_gates),
        pinned_ggen_sha=PINNED_GGEN_SHA,
        checked_surfaces=(
            "identity",
            "ontology",
            "pack-transport",
            "query-template-closure",
            "bounded-writes",
            "collision-polarity",
            "generated-ownership",
            "receipt-replay-ci",
        ),
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    try:
        report = verify(args.root)
    except ContractError as exc:
        print(str(exc), file=sys.stderr)
        return 1
    if args.json:
        print(json.dumps(asdict(report), indent=2, sort_keys=True))
    else:
        print(
            f"{report.state}: {report.project}@{report.version}; "
            f"rules={report.generation_rules}; gates={report.validation_gates}; "
            f"ggen={report.pinned_ggen_sha}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
