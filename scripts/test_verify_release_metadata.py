from __future__ import annotations

import copy
import json
import tempfile
import unittest
from pathlib import Path

from scripts.verify_release_metadata import EXPECTED_PACKAGE, verify_manifest


def render_manifest(
    package: dict[str, object] | None = None,
    dependencies: dict[str, str] | None = None,
) -> str:
    package = copy.deepcopy(package or EXPECTED_PACKAGE)
    dependencies = dependencies or {"syn": "2.0", "quote": "1.0"}
    lines = ["[package]"]
    for key, value in package.items():
        if isinstance(value, list):
            encoded = ", ".join(json.dumps(item) for item in value)
            lines.append(f"{key} = [{encoded}]")
        else:
            lines.append(f"{key} = {json.dumps(value)}")
    lines.extend(["", "[dependencies]"])
    for key, value in dependencies.items():
        lines.append(f"{key} = {json.dumps(value)}")
    return "\n".join(lines) + "\n"


class ReleaseMetadataVerifierTests(unittest.TestCase):
    def verify_text(self, text: str) -> dict[str, object]:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "Cargo.toml"
            path.write_text(text, encoding="utf-8")
            return verify_manifest(path)

    def test_admits_exact_release_metadata_without_forbidden_dependency(self) -> None:
        report = self.verify_text(render_manifest())
        self.assertEqual(report["admission"], "ADMITTED")
        self.assertEqual(report["violations"], [])
        self.assertFalse(report["actuation_performed"])

    def test_refuses_missing_msrv(self) -> None:
        package = copy.deepcopy(EXPECTED_PACKAGE)
        del package["rust-version"]
        report = self.verify_text(render_manifest(package=package))
        self.assertEqual(report["admission"], "REFUSED")
        self.assertTrue(any(v.get("field") == "rust-version" for v in report["violations"]))

    def test_refuses_license_drift(self) -> None:
        package = copy.deepcopy(EXPECTED_PACKAGE)
        package["license"] = "MIT"
        report = self.verify_text(render_manifest(package=package))
        self.assertEqual(report["admission"], "REFUSED")
        self.assertTrue(any(v.get("field") == "license" for v in report["violations"]))

    def test_refuses_reintroduction_of_proc_macro_error(self) -> None:
        report = self.verify_text(
            render_manifest(dependencies={"syn": "2.0", "proc-macro-error": "1.0"})
        )
        self.assertEqual(report["admission"], "REFUSED")
        self.assertTrue(
            any(v.get("rule") == "FORBIDDEN_UNMAINTAINED_DEPENDENCY" for v in report["violations"])
        )

    def test_report_replay_is_deterministic_for_same_subject(self) -> None:
        first = self.verify_text(render_manifest())
        second = self.verify_text(render_manifest())
        first["manifest"] = "Cargo.toml"
        second["manifest"] = "Cargo.toml"
        self.assertEqual(
            json.dumps(first, sort_keys=True),
            json.dumps(second, sort_keys=True),
        )


if __name__ == "__main__":
    unittest.main()
