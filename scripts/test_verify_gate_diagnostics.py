from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from scripts.verify_gate_diagnostics import build_report

SHA = "1" * 40


def fixture(root: Path, exits: dict[str, int] | None = None) -> tuple[Path, Path]:
    exits = exits or {"format": 1, "clippy": 1, "docs-benchmark": 1}
    diagnostics = root / "diagnostics"
    diagnostics.mkdir()
    outcomes = {}
    for gate, code in exits.items():
        (diagnostics / f"{gate}.log").write_text(f"{gate}: diagnostic\n", encoding="utf-8")
        outcomes[gate] = {"command": f"run-{gate}", "exit_code": code}
    outcomes_path = root / "outcomes.json"
    outcomes_path.write_text(json.dumps(outcomes, sort_keys=True), encoding="utf-8")
    return outcomes_path, diagnostics


class GateDiagnosticsTests(unittest.TestCase):
    def test_admits_complete_failed_gate_diagnostics(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            outcomes, diagnostics = fixture(Path(directory))
            report = build_report(SHA, outcomes, diagnostics)
            self.assertEqual(report["admission"], "ADMITTED")
            self.assertEqual(report["standing"], "BUILD_BROKEN")
            self.assertEqual(report["failing_gates"], ["clippy", "docs-benchmark", "format"])

    def test_alive_only_when_observed_gates_succeed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            outcomes, diagnostics = fixture(Path(directory), {g: 0 for g in ("format", "clippy", "docs-benchmark")})
            report = build_report(SHA, outcomes, diagnostics)
            self.assertEqual(report["standing"], "ALIVE")

    def test_refuses_missing_failed_gate_log(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            outcomes, diagnostics = fixture(Path(directory))
            (diagnostics / "clippy.log").unlink()
            report = build_report(SHA, outcomes, diagnostics)
            self.assertEqual(report["admission"], "REFUSED")
            self.assertTrue(any(v["rule"] == "DIAGNOSTIC_LOG_REQUIRED" for v in report["violations"]))

    def test_refuses_empty_failed_gate_log(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            outcomes, diagnostics = fixture(Path(directory))
            (diagnostics / "format.log").write_text("", encoding="utf-8")
            report = build_report(SHA, outcomes, diagnostics)
            self.assertEqual(report["admission"], "REFUSED")
            self.assertTrue(any(v["rule"] == "FAILED_GATE_REQUIRES_DIAGNOSTIC_PAYLOAD" for v in report["violations"]))

    def test_refuses_non_exact_subject(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            outcomes, diagnostics = fixture(Path(directory))
            report = build_report("main", outcomes, diagnostics)
            self.assertEqual(report["admission"], "REFUSED")

    def test_replay_is_deterministic(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            outcomes, diagnostics = fixture(Path(directory))
            self.assertEqual(build_report(SHA, outcomes, diagnostics), build_report(SHA, outcomes, diagnostics))


if __name__ == "__main__":
    unittest.main()
