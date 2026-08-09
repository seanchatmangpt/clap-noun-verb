#!/usr/bin/env python3
"""Executable falsifiers for the unfinished-work verifier."""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("verify_no_wip.py")
SPEC = importlib.util.spec_from_file_location("verify_no_wip", MODULE_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("unable to load verifier module")
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class VerifyNoWorkInProgressTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tempdir = tempfile.TemporaryDirectory()
        self.root = Path(self.tempdir.name)
        (self.root / "src").mkdir()
        (self.root / "examples").mkdir()
        (self.root / ".github" / "workflows").mkdir(parents=True)
        (self.root / "src" / "lib.rs").write_text("pub fn admitted() -> bool { true }\n")
        (self.root / "examples" / "alive.rs").write_text(
            "fn main() { assert!(true); }\n"
        )

    def tearDown(self) -> None:
        self.tempdir.cleanup()

    def verify(self) -> dict[str, object]:
        return MODULE.verify(self.root)

    def test_clean_tree_is_admitted(self) -> None:
        report = self.verify()
        self.assertEqual(report["admission"], "ADMITTED")
        self.assertEqual(report["violation_count"], 0)

    def test_unfinished_macro_is_refused(self) -> None:
        marker = "to" + "do!" + "()"
        (self.root / "src" / "lib.rs").write_text(f"pub fn broken() {{ {marker}; }}\n")
        report = self.verify()
        self.assertEqual(report["admission"], "REFUSED")
        rules = {item["rule"] for item in report["violations"]}
        self.assertIn("TODO_MACRO", rules)

    def test_marker_in_rust_fixture_string_is_not_refused(self) -> None:
        marker = "un" + "implemented!" + "()"
        (self.root / "src" / "lib.rs").write_text(
            'pub const FIXTURE: &str = r#"fn parsed_only() { ' + marker + ' }"#;\n'
        )
        report = self.verify()
        self.assertEqual(report["admission"], "ADMITTED")
        self.assertEqual(report["violation_count"], 0)

    def test_unfinished_comment_is_refused(self) -> None:
        marker = "TO" + "DO"
        (self.root / "src" / "lib.rs").write_text(
            f"// {marker}: complete admitted behavior\npub fn broken() {{}}\n"
        )
        report = self.verify()
        rules = {item["rule"] for item in report["violations"]}
        self.assertIn("TODO_MARKER", rules)

    def test_empty_example_is_refused(self) -> None:
        empty = "fn main() " + "{" + "}"
        (self.root / "examples" / "empty.rs").write_text(empty + "\n")
        report = self.verify()
        rules = {item["rule"] for item in report["violations"]}
        self.assertIn("EMPTY_EXAMPLE_MAIN", rules)

    def test_disabled_workflow_is_refused(self) -> None:
        disabled = "jobs:\n  proof:\n    if: " + "false" + "\n    runs-on: ubuntu-latest\n"
        (self.root / ".github" / "workflows" / "disabled.yml").write_text(disabled)
        report = self.verify()
        rules = {item["rule"] for item in report["violations"]}
        self.assertIn("DISABLED_WORKFLOW", rules)


if __name__ == "__main__":
    unittest.main()
