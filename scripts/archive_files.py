#!/usr/bin/env python3
import os
import shutil
import glob
import sys

def main():
    repo_root = "/Users/sac/clap-noun-verb"
    os.chdir(repo_root)

    # 1. Define explicit moves
    explicit_moves = {
        "CONVO.txt": "archive/CONVO.txt",
        "RESEARCH_THESIS.tex": "archive/thesis/RESEARCH_THESIS.tex",
        "SPARQL_QUERIES_SUMMARY.txt": "archive/research/SPARQL_QUERIES_SUMMARY.txt",
        "check_errors.log": "archive/check_errors.log",
        "concept_coverage.json": "archive/research/concept_coverage.json",
        "concept_gaps.json": "archive/research/concept_gaps.json",
        "concept_ruleset.yaml": "archive/research/concept_ruleset.yaml",
        "evidence_graph.json": "archive/research/evidence_graph.json",
        "evidence_graph_extended.json": "archive/research/evidence_graph_extended.json",
        "implementation_receipt.yaml": "archive/receipts/implementation_receipt.yaml",
        "validation_receipt.yaml": "archive/receipts/validation_receipt.yaml",
        "portfolio_evidence_mcpp.tar.gz": "archive/receipts/portfolio_evidence_mcpp.tar.gz",
        "ralph_plan.json": "archive/research/ralph_plan.json",
        "refactor_ggen_v2.sh": "archive/research/refactor_ggen_v2.sh",
        "run-sparql-queries.py": "archive/research/run-sparql-queries.py",
        "docs/ggen-quickstart.md": "archive/docs/ggen-quickstart.md",
        "docs/ggen-manufacturing-system.md": "archive/docs/ggen-manufacturing-system.md",
        "docs/abb_governance_ontology.ttl": "archive/docs/abb_governance_ontology.ttl",
        "docs/index.md": "archive/docs/index.md",
        "docs/_internal": "archive/docs/_internal",
        "docs/explanation": "archive/docs/explanation",
        "docs/howto": "archive/docs/howto",
        "docs/reference": "archive/docs/reference",
        "docs/tutorial": "archive/docs/tutorial",
    }

    # 2. Find all root EVIDENCE.* files
    evidence_files = glob.glob("EVIDENCE.*")
    for f in evidence_files:
        if os.path.isfile(f):
            explicit_moves[f] = f"archive/receipts/{f}"

    print(f"Total entries to move: {len(explicit_moves)}")

    # 3. Perform moves
    errors = 0
    moved_count = 0
    for src, dst in explicit_moves.items():
        if not os.path.exists(src):
            print(f"WARNING: Source path '{src}' does not exist.")
            # If target already exists, maybe it was already moved
            if os.path.exists(dst):
                print(f"  Target '{dst}' already exists. Assuming already moved.")
                continue
            else:
                print(f"  ERROR: Neither '{src}' nor '{dst}' exists.")
                errors += 1
                continue

        # Ensure parent directory of dst exists
        dst_dir = os.path.dirname(dst)
        if dst_dir:
            os.makedirs(dst_dir, exist_ok=True)

        try:
            print(f"Moving '{src}' -> '{dst}'")
            # If target exists and is a directory (and source is a directory), shutil.move will nest it inside
            # So if target exists, we remove it first to avoid nesting.
            if os.path.exists(dst):
                if os.path.isdir(dst):
                    shutil.rmtree(dst)
                else:
                    os.remove(dst)
            shutil.move(src, dst)
            moved_count += 1
        except Exception as e:
            print(f"ERROR: Failed to move '{src}' to '{dst}': {e}")
            errors += 1

    # 4. Verification
    print("\n--- Verification ---")
    verification_failed = False
    for src, dst in explicit_moves.items():
        # Check source is gone
        if os.path.exists(src):
            print(f"ERROR: Source '{src}' still exists!")
            verification_failed = True
        else:
            print(f"OK: Source '{src}' is gone.")

        # Check target exists
        if not os.path.exists(dst):
            print(f"ERROR: Target '{dst}' does not exist!")
            verification_failed = True
        else:
            print(f"OK: Target '{dst}' exists.")

    if errors > 0 or verification_failed:
        print("\nMove failed verification or encountered errors.", file=sys.stderr)
        sys.exit(1)
    else:
        print("\nAll moves completed and verified successfully!")
        print(f"Successfully moved {moved_count} items.")

if __name__ == "__main__":
    main()
