/**
 * Real project data — derived by reading source and running the binary.
 * No fixtures. If a value can't be sourced from the actual project, it is absent.
 */
import { execSync } from "child_process";
import path from "path";

const REPO = path.resolve(process.cwd(), "..");

function run(cmd: string, cwd = REPO): string {
  try {
    return execSync(cmd, { cwd, timeout: 30_000, encoding: "utf8" }).trim();
  } catch (e: unknown) {
    const err = e as { stdout?: string; stderr?: string; message?: string };
    throw new Error(err.stderr ?? err.message ?? String(e));
  }
}

// ---------------------------------------------------------------------------
// Static metadata — from Cargo.toml via `cargo metadata`
// ---------------------------------------------------------------------------

export interface ProjectMeta {
  name: string;
  version: string;
  description: string;
  features: string[];
  exampleNames: string[];
  moduleCount: number;
  sourceLineCount: number;
}

export function getProjectMeta(): ProjectMeta {
  const raw = run("cargo metadata --no-deps --format-version 1");
  const meta = JSON.parse(raw);
  const pkg = meta.packages.find((p: { name: string }) => p.name === "clap-noun-verb");
  if (!pkg) throw new Error("clap-noun-verb not found in cargo metadata");

  const features = Object.keys(pkg.features).filter((f) => f !== "default");
  const exampleNames = pkg.targets
    .filter((t: { kind: string[] }) => t.kind.includes("example"))
    .map((t: { name: string }) => t.name);

  // Module count: public mods in src/lib.rs
  const libRs = run("grep -c '^pub mod' src/lib.rs");
  const moduleCount = parseInt(libRs, 10);

  // Source line count
  const lineOut = run("find src/ -name '*.rs' | xargs wc -l | tail -1");
  const sourceLineCount = parseInt(lineOut.trim().split(/\s+/)[0], 10);

  return {
    name: pkg.name,
    version: pkg.version,
    description: pkg.description,
    features,
    exampleNames,
    moduleCount,
    sourceLineCount,
  };
}

// ---------------------------------------------------------------------------
// Live: test suite run
// ---------------------------------------------------------------------------

export interface TestResult {
  passed: number;
  failed: number;
  ignored: number;
  durationSec: number;
}

export function runTests(): TestResult {
  const out = run("cargo test --quiet 2>&1", REPO);
  const m = out.match(/(\d+) passed; (\d+) failed; (\d+) ignored.*finished in ([\d.]+)s/);
  if (!m) throw new Error(`Unexpected test output: ${out.slice(0, 200)}`);
  return {
    passed: parseInt(m[1]),
    failed: parseInt(m[2]),
    ignored: parseInt(m[3]),
    durationSec: parseFloat(m[4]),
  };
}

// ---------------------------------------------------------------------------
// Live: run a named example, capture stdout
// ---------------------------------------------------------------------------

export interface ExampleRun {
  name: string;
  output: string;
  exitCode: number;
  durationMs: number;
}

export function runExample(name: string): ExampleRun {
  const t0 = Date.now();
  let output = "";
  let exitCode = 0;
  try {
    output = run(`cargo run --example ${name} 2>/dev/null`);
  } catch (e: unknown) {
    const err = e as { stdout?: string; message?: string; status?: number };
    output = err.stdout ?? err.message ?? String(e);
    exitCode = err.status ?? 1;
  }
  return { name, output, exitCode, durationMs: Date.now() - t0 };
}

// ---------------------------------------------------------------------------
// Coverage map — from DOC_COVERAGE_LOG.md
// ---------------------------------------------------------------------------

export interface CoverageStatus {
  iteration: number;
  documentedButUnexercised: number;
  exercisedButUndocumented: number;
  runningWitnesses: number;
  remainingGaps: string[];
}

export function getCoverageStatus(): CoverageStatus {
  const { readFileSync } = require("fs");
  const log = readFileSync(path.join(REPO, "DOC_COVERAGE_LOG.md"), "utf8") as string;

  // Find last "Coverage Status After" block
  const blocks = [...log.matchAll(/## Coverage Status After Iteration (\d+)([\s\S]+?)(?=\n##|$)/g)];
  const last = blocks[blocks.length - 1];
  const iteration = parseInt(last[1]);
  const block = last[2];

  const dbu = block.match(/Documented-but-unexercised.*?(\d+)\s*$/m);
  const ebu = block.match(/Exercised-but-undocumented.*?(\d+)\s/m);
  const rw = block.match(/Running examples.*?(\d+)\s*$/m);

  // Remaining gaps table
  const gapRows = [...block.matchAll(/\| (?:HIGH|MEDIUM|LOW) \| `([^`]+)`/g)].map((m) => m[1]);

  return {
    iteration,
    documentedButUnexercised: dbu ? parseInt(dbu[1]) : 0,
    exercisedButUndocumented: ebu ? parseInt(ebu[1]) : 0,
    runningWitnesses: rw ? parseInt(rw[1]) : 0,
    remainingGaps: gapRows,
  };
}

// ---------------------------------------------------------------------------
// Real public error variant names
// ---------------------------------------------------------------------------

export function getErrorVariants(): string[] {
  const { readFileSync } = require("fs");
  const src = readFileSync(path.join(REPO, "src/error.rs"), "utf8") as string;
  return [...src.matchAll(/#\[error\("[^"]+"\)]\n\s+(\w+)/g)].map((m) => m[1]);
}

// ---------------------------------------------------------------------------
// Real OutputFormat variant names
// ---------------------------------------------------------------------------

export function getOutputFormatVariants(): string[] {
  const { readFileSync } = require("fs");
  const src = readFileSync(path.join(REPO, "src/format.rs"), "utf8") as string;
  const block = src.match(/pub enum OutputFormat \{([\s\S]+?)\}/)?.[1] ?? "";
  return [...block.matchAll(/^\s+(\w+),/gm)].map((m) => m[1]);
}
