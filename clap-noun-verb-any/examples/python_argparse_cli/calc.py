#!/usr/bin/env python3
"""A trivial argparse-based CLI, wrapped by cnv-any to prove the "any
language" claim independent of autofde-lab's much heavier example -- this
one needs nothing but a system python3, no venv, no dependencies at all.

argv is the manifest's full command path plus arguments (e.g. "calc add 2
3"), not just the verb -- ggen's schema-pack always groups commands under a
real cnv:Noun (it reserves the literal noun name "root"), so the wrapped
program strips the leading noun token itself rather than the manifest
pretending that segment doesn't exist.
"""
import argparse
import sys


def main() -> int:
    sys.argv = [sys.argv[0]] + sys.argv[2:]  # drop the ggen-emitted "calc" noun token
    parser = argparse.ArgumentParser(prog="calc")
    subparsers = parser.add_subparsers(dest="command", required=True)

    add = subparsers.add_parser("add")
    add.add_argument("a", type=int)
    add.add_argument("b", type=int)

    multiply = subparsers.add_parser("multiply")
    multiply.add_argument("a", type=int)
    multiply.add_argument("b", type=int)

    args = parser.parse_args()
    if args.command == "add":
        print(args.a + args.b)
    elif args.command == "multiply":
        print(args.a * args.b)
    return 0


if __name__ == "__main__":
    sys.exit(main())
