from __future__ import annotations

import importlib.util
import tempfile
import sys
import textwrap
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("verify_ggen_contract.py")
SPEC = importlib.util.spec_from_file_location("verify_ggen_contract", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class GgenContractVerifierTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.write_fixture()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def write(self, relative: str, content: str) -> None:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(textwrap.dedent(content).lstrip(), encoding="utf-8")

    def write_fixture(self) -> None:
        gates = [
            "gates/fieldname-collision.rq",
            "gates/argument-semantics.rq",
            "gates/projection-delimiter.rq",
        ]
        gate_list = ", ".join(f'"{gate}"' for gate in gates)
        manifest = f"""
        [project]
        name = "clap-noun-verb"
        version = "26.7.62"
        [ontology]
        source = "ontology/clap-noun-verb-ontology.ttl"
        base_iri = "http://clap-noun-verb.io/ontology#"
        [ontology.prefixes]
        cnv = "http://clap-noun-verb.io/ontology#"
        [generation]
        output_dir = "."
        [[generation.rules]]
        name = "verb-wrappers"
        query = {{ file = "queries/verb-signatures.rq" }}
        template = {{ file = "templates/verb.rs.tera" }}
        output_file = "src/verbs/{{{{ verb_name }}}}.rs"
        mode = "Overwrite"
        skip_empty = true
        [[generation.rules]]
        name = "verbs-mod"
        query = {{ file = "queries/verbs-mod.rq" }}
        template = {{ file = "templates/verbs-mod.rs.tera" }}
        output_file = "src/verbs/mod.rs"
        mode = "Overwrite"
        skip_empty = true
        [validation]
        gates = [{gate_list}]
        """
        self.write("ggen.toml", manifest)
        self.write("Cargo.toml", '[package]\nname="clap-noun-verb"\nversion="26.7.62"\n')
        self.write(
            "package.toml",
            '[pack]\nname="clap-noun-verb"\n[pack.outputs]\nqueries="queries"\ntemplates="templates"\ngates="gates"\n',
        )
        ontology_predicates = "\n".join(
            [f"cnv:{predicate} a rdf:Property ." for predicate in MODULE.REQUIRED_ARGUMENT_PREDICATES]
            + [f"cnv:{argument_type} a cnv:ArgumentType ." for argument_type in MODULE.REQUIRED_ARGUMENT_TYPES]
        )
        self.write(
            "ontology/clap-noun-verb-ontology.ttl",
            """
            @prefix cnv: <http://clap-noun-verb.io/ontology#> .
            @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
            """
            + ontology_predicates
            + "\n",
        )
        self.write(
            "gates/fieldname-collision.rq",
            """
            # MESSAGE: collision
            ASK {
              ?verb <x:hasArguments> ?a, ?b .
              FILTER(?a = ?b)
            }
            """,
        )
        self.write(
            "gates/argument-semantics.rq",
            "# MESSAGE: semantics\nASK { FILTER(false) }\n",
        )
        self.write(
            "gates/projection-delimiter.rq",
            "# MESSAGE: delimiter\nASK { FILTER(false) }\n",
        )
        self.write(
            "ontology/queries/fieldname-collision.rq",
            "# MOVED: ../../gates/fieldname-collision.rq\n",
        )
        self.write(
            "queries/verb-signatures.rq",
            (Path(__file__).resolve().parents[1] / "queries/verb-signatures.rq").read_text(),
        )
        self.write("queries/verbs-mod.rq", "SELECT ?modules WHERE { BIND('x' AS ?modules) }\n")
        self.write(
            "templates/verb.rs.tera",
            (Path(__file__).resolve().parents[1] / "templates/verb.rs.tera").read_text(),
        )
        self.write("templates/verbs-mod.rs.tera", "// generated\n")
        example_manifest = manifest.replace(
            'source = "ontology/clap-noun-verb-ontology.ttl"', 'source = "ontology.ttl"'
        )
        example_manifest = example_manifest.replace(
            'query = { file = "queries/', 'query = { file = "../../queries/'
        )
        example_manifest = example_manifest.replace(
            'template = { file = "templates/', 'template = { file = "../../templates/'
        )
        for gate in gates:
            example_manifest = example_manifest.replace(f'"{gate}"', f'"../../{gate}"')
        self.write("examples/greet-demo/ggen.toml", example_manifest)
        self.write("examples/greet-demo/src/verbs/greet.rs", "// rendered from O* by ggen\n")
        self.write("AGENTS.md", "law\n")
        self.write("docs/GGEN_AUTHORITY.md", "authority\n")
        self.write(
            ".github/workflows/ggen-authority.yml",
            f"# {MODULE.PINNED_GGEN_SHA}\n"
            "# ggen sync run\n"
            "# ggen receipt verify\n"
            "# Prove full argument projection semantics\n",
        )

    def test_admits_closed_contract(self) -> None:
        report = MODULE.verify(self.root)
        self.assertEqual(report.state, "PARTIAL_ALIVE")
        self.assertEqual(report.generation_rules, 2)
        self.assertEqual(report.validation_gates, 3)

    def test_refuses_version_drift(self) -> None:
        path = self.root / "ggen.toml"
        path.write_text(path.read_text().replace('version = "26.7.62"', 'version = "0.0.0"', 1))
        with self.assertRaisesRegex(MODULE.ContractError, "GGEN_VERSION_DRIFT_REFUSED"):
            MODULE.verify(self.root)

    def test_refuses_output_escape(self) -> None:
        path = self.root / "ggen.toml"
        path.write_text(path.read_text().replace('output_file = "src/verbs/', 'output_file = "../escape/'))
        with self.assertRaisesRegex(MODULE.ContractError, "GGEN_OUTPUT_ESCAPE_REFUSED"):
            MODULE.verify(self.root)

    def test_refuses_inverted_ask(self) -> None:
        gate = self.root / "gates/fieldname-collision.rq"
        gate.write_text("# MESSAGE: collision\nASK { FILTER NOT EXISTS { ?s ?p ?o } }\n")
        with self.assertRaisesRegex(MODULE.ContractError, "GGEN_GATE_POLARITY_REFUSED"):
            MODULE.verify(self.root)

    def test_refuses_stale_position_predicate(self) -> None:
        query = self.root / "queries/verb-signatures.rq"
        query.write_text(query.read_text().replace("cnv:cliPosition", "cnv:positional"))
        with self.assertRaisesRegex(
            MODULE.ContractError, "GGEN_ONTOLOGY_QUERY_PREDICATE_DRIFT_REFUSED"
        ):
            MODULE.verify(self.root)

    def test_refuses_omitted_argument_surface(self) -> None:
        query = self.root / "queries/verb-signatures.rq"
        query.write_text(query.read_text().replace("cnv:shortName", "cnv:legacyShortName"))
        with self.assertRaisesRegex(MODULE.ContractError, "GGEN_ONTOLOGY_QUERY_CLOSURE_REFUSED"):
            MODULE.verify(self.root)

    def test_refuses_nondeterministic_argument_projection(self) -> None:
        query = self.root / "queries/verb-signatures.rq"
        query.write_text(query.read_text().replace("?sort_key", "?legacy_order"))
        with self.assertRaisesRegex(
            MODULE.ContractError, "GGEN_ARGUMENT_ORDER_NONDETERMINISTIC_REFUSED"
        ):
            MODULE.verify(self.root)

    def test_refuses_missing_gate(self) -> None:
        manifest = self.root / "ggen.toml"
        manifest.write_text(
            manifest.read_text().replace(', "gates/projection-delimiter.rq"', "")
        )
        with self.assertRaisesRegex(MODULE.ContractError, "GGEN_GATE_CLOSURE_REFUSED"):
            MODULE.verify(self.root)


if __name__ == "__main__":
    unittest.main()
