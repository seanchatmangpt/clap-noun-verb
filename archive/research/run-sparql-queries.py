#!/usr/bin/env python3
"""
Run SPARQL queries against the clap-noun-verb ontology.
Demonstrates CLI introspection and generation queries.
"""

import sys
from pathlib import Path
from rdflib import Graph, Namespace
from rdflib.plugins.sparql import prepareQuery

# Setup paths
PROJECT_ROOT = Path(__file__).parent
ONTOLOGY_DIR = PROJECT_ROOT / "ontology"
QUERIES_DIR = PROJECT_ROOT / "queries"

# Load RDF files
graph = Graph()

# Load main ontology files
for rdf_file in ONTOLOGY_DIR.glob("*.ttl"):
    print(f"Loading {rdf_file.name}...", file=sys.stderr)
    graph.parse(str(rdf_file), format="turtle")

# Define namespaces
CNV = Namespace("http://clap-noun-verb.io/ontology#")
EX = Namespace("http://clap-noun-verb.io/examples#")
SPEC = Namespace("http://clap-noun-verb.io/spec#")

graph.bind("cnv", CNV)
graph.bind("ex", EX)
graph.bind("spec", SPEC)

print(f"✓ Loaded {len(graph)} triples from ontology", file=sys.stderr)
print("", file=sys.stderr)


def run_query(query_file: Path, description: str) -> None:
    """Run a SPARQL query and print results."""
    print(f"\n{'='*80}")
    print(f"Query: {query_file.name}")
    print(f"Description: {description}")
    print(f"{'='*80}\n")

    query_text = query_file.read_text()
    print(f"SPARQL:\n{query_text}\n")
    print("-" * 80)
    print("RESULTS:\n")

    try:
        query = prepareQuery(query_text)
        results = graph.query(query)

        # Display results
        if query_text.strip().startswith("CONSTRUCT"):
            print(f"Constructed {len(results)} triples:\n")
            for triple in list(results)[:10]:  # Show first 10
                print(f"  {triple}")
            if len(results) > 10:
                print(f"  ... and {len(results) - 10} more triples")
        else:
            # SELECT query
            if len(results) == 0:
                print("(No results)")
            else:
                # Print header
                if hasattr(results, 'vars'):
                    header = " | ".join(str(v) for v in results.vars)
                    print(header)
                    print("-" * len(header))

                # Print rows
                for row in list(results)[:15]:  # Show first 15 rows
                    values = []
                    for cell in row:
                        if cell is None:
                            values.append("(null)")
                        elif hasattr(cell, 'split') and '#' in str(cell):
                            # Extract local name from URI
                            local_name = str(cell).split('#')[-1]
                            values.append(local_name)
                        else:
                            val_str = str(cell)
                            if len(val_str) > 40:
                                val_str = val_str[:37] + "..."
                            values.append(val_str)
                    print(" | ".join(values))

                if len(results) > 15:
                    print(f"\n... and {len(results) - 15} more rows\n")
                else:
                    print()

        print(f"Total: {len(results)} results\n")

    except Exception as e:
        print(f"ERROR: {e}\n", file=sys.stderr)


# Run each query
if __name__ == "__main__":
    queries = [
        (
            QUERIES_DIR / "find-all-verbs.rq",
            "Find all verb definitions in the ontology",
        ),
        (
            QUERIES_DIR / "extract-arguments.rq",
            "Extract argument specifications for a verb (example: StatusVerb)",
        ),
        (
            QUERIES_DIR / "validate-cli-structure.rq",
            "Validate CLI structure conformance to noun-verb pattern",
        ),
        (
            QUERIES_DIR / "generate-trait-impls.rq",
            "Extract trait requirements for code generation",
        ),
        (
            QUERIES_DIR / "generate-cli-spec.rq",
            "Construct normalized CLI specification (CONSTRUCT query)",
        ),
    ]

    print(f"\n{'*'*80}")
    print(f"SPARQL Query Execution Results")
    print(f"Ontology: {ONTOLOGY_DIR}")
    print(f"Queries: {QUERIES_DIR}")
    print(f"{'*'*80}\n")

    for query_file, description in queries:
        if query_file.exists():
            run_query(query_file, description)
        else:
            print(f"Query file not found: {query_file}", file=sys.stderr)

    print("\n" + "=" * 80)
    print("Query execution complete")
    print("=" * 80)
