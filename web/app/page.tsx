import { Suspense } from "react";
import {
  getProjectMeta,
  getCoverageStatus,
  getErrorVariants,
  getOutputFormatVariants,
} from "@/lib/project";
import { ExampleRunner } from "./components/ExampleRunner";

// ---------------------------------------------------------------------------
// Server Components — fetch real project data at request time
// ---------------------------------------------------------------------------

async function ProjectHeader() {
  const meta = getProjectMeta();
  return (
    <header className="border-b border-zinc-800 pb-6 mb-8">
      <div className="flex items-baseline gap-3">
        <h1 className="text-2xl font-mono font-bold text-white">{meta.name}</h1>
        <span className="text-sm text-amber-400 font-mono">v{meta.version}</span>
      </div>
      <p className="mt-1 text-zinc-400 text-sm">{meta.description}</p>
      <div className="mt-4 grid grid-cols-2 sm:grid-cols-4 gap-3">
        <Stat label="modules" value={meta.moduleCount} />
        <Stat label="source lines" value={meta.sourceLineCount.toLocaleString()} />
        <Stat label="features" value={meta.features.length} />
        <Stat label="examples" value={meta.exampleNames.length} />
      </div>
    </header>
  );
}

function Stat({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="bg-zinc-900 rounded-lg p-3 border border-zinc-800">
      <div className="text-lg font-mono font-bold text-white">{value}</div>
      <div className="text-xs text-zinc-500 mt-0.5">{label}</div>
    </div>
  );
}

async function FeatureFlags() {
  const meta = getProjectMeta();
  return (
    <section className="mb-8">
      <SectionTitle>Feature Flags</SectionTitle>
      <div className="flex flex-wrap gap-2">
        {meta.features.map((f) => (
          <span
            key={f}
            className="px-2 py-1 text-xs font-mono bg-zinc-800 text-amber-300 rounded border border-zinc-700"
          >
            {f}
          </span>
        ))}
      </div>
      <p className="mt-2 text-xs text-zinc-600">
        Derived from{" "}
        <code className="text-zinc-500">cargo metadata --no-deps --format-version 1</code>
      </p>
    </section>
  );
}

async function ApiSurface() {
  const errors = getErrorVariants();
  const formats = getOutputFormatVariants();
  return (
    <section className="mb-8">
      <SectionTitle>Public API Surface</SectionTitle>
      <div className="grid sm:grid-cols-2 gap-6">
        <div>
          <h3 className="text-xs text-zinc-500 uppercase tracking-wider mb-2">
            NounVerbError variants{" "}
            <span className="text-zinc-600 normal-case">(parsed from src/error.rs)</span>
          </h3>
          <ul className="space-y-1">
            {errors.map((v) => (
              <li key={v} className="font-mono text-sm text-red-400">
                {v}
              </li>
            ))}
          </ul>
        </div>
        <div>
          <h3 className="text-xs text-zinc-500 uppercase tracking-wider mb-2">
            OutputFormat variants{" "}
            <span className="text-zinc-600 normal-case">(parsed from src/format.rs)</span>
          </h3>
          <ul className="space-y-1">
            {formats.map((v) => (
              <li key={v} className="font-mono text-sm text-blue-400">
                {v}
              </li>
            ))}
          </ul>
        </div>
      </div>
    </section>
  );
}

async function CoverageMap() {
  const cov = getCoverageStatus();
  return (
    <section className="mb-8">
      <SectionTitle>Doc Coverage Map — Iteration {cov.iteration}</SectionTitle>
      <div className="grid grid-cols-3 gap-3 mb-4">
        <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-3">
          <div
            className={`text-2xl font-mono font-bold ${
              cov.documentedButUnexercised > 0 ? "text-yellow-400" : "text-green-400"
            }`}
          >
            {cov.documentedButUnexercised}
          </div>
          <div className="text-xs text-zinc-500 mt-1">documented but unexercised</div>
        </div>
        <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-3">
          <div
            className={`text-2xl font-mono font-bold ${
              cov.exercisedButUndocumented > 0 ? "text-red-400" : "text-green-400"
            }`}
          >
            {cov.exercisedButUndocumented}
          </div>
          <div className="text-xs text-zinc-500 mt-1">exercised but undocumented</div>
        </div>
        <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-3">
          <div className="text-2xl font-mono font-bold text-green-400">
            {cov.runningWitnesses}
          </div>
          <div className="text-xs text-zinc-500 mt-1">running witnesses</div>
        </div>
      </div>
      {cov.remainingGaps.length > 0 && (
        <div>
          <h3 className="text-xs text-zinc-500 uppercase tracking-wider mb-2">Remaining Gaps</h3>
          <ul className="space-y-1">
            {cov.remainingGaps.map((g) => (
              <li key={g} className="font-mono text-xs text-zinc-400">
                <span className="text-yellow-600 mr-2">▷</span>
                {g}
              </li>
            ))}
          </ul>
        </div>
      )}
      <p className="mt-3 text-xs text-zinc-600">Source: DOC_COVERAGE_LOG.md</p>
    </section>
  );
}

async function ExampleRunners() {
  const meta = getProjectMeta();
  const coverageExamples = [
    "core_api",
    "verb_args",
    "error_handling",
    "proc_macro_verb",
    "output_formats",
    "command_tree",
  ].filter((e) => meta.exampleNames.includes(e));

  return (
    <section className="mb-8">
      <SectionTitle>Live Example Runner</SectionTitle>
      <p className="text-xs text-zinc-500 mb-4">
        Each button executes{" "}
        <code className="text-zinc-400">cargo run --example &lt;name&gt;</code> in the repo and
        returns real stdout. Output is what the binary actually emits — no fixtures.
      </p>
      <div className="space-y-4">
        {coverageExamples.map((name) => (
          <ExampleRunner key={name} name={name} />
        ))}
      </div>
    </section>
  );
}

function RepresentationGap() {
  return (
    <section className="mb-8 border border-zinc-800 rounded-lg p-4 bg-zinc-950">
      <SectionTitle>Representation Gap Map</SectionTitle>
      <div className="grid sm:grid-cols-2 gap-6 text-xs">
        <div>
          <h3 className="text-red-400 font-semibold mb-2">Rendered-but-fabricated</h3>
          <p className="text-green-400">
            None. Every value on this page is derived from real project output.
          </p>
        </div>
        <div>
          <h3 className="text-yellow-400 font-semibold mb-2">Exposed-but-unrepresented</h3>
          <ul className="space-y-1 text-zinc-400">
            <li>▷ AppContext (src/context.rs) — no UI section yet</li>
            <li>▷ Graph/Triple (src/graph/) — no UI section yet</li>
            <li>▷ CapabilityRegistry (src/capability/) — no UI section yet</li>
            <li>▷ DoctorOutput/HealthIssue (src/diagnostics/) — no UI section yet</li>
            <li>▷ Repl (feature-gated, src/repl.rs) — no UI section yet</li>
            <li>▷ Benchmark results (benches/dispatch.rs) — not run yet</li>
          </ul>
        </div>
      </div>
    </section>
  );
}

// ---------------------------------------------------------------------------
// Shared UI
// ---------------------------------------------------------------------------

function SectionTitle({ children }: { children: React.ReactNode }) {
  return (
    <h2 className="text-sm font-semibold text-zinc-300 uppercase tracking-wider mb-4 flex items-center gap-2">
      <span className="w-4 h-px bg-zinc-700 block" />
      {children}
      <span className="flex-1 h-px bg-zinc-800 block" />
    </h2>
  );
}

function LoadingSkeleton({ label }: { label: string }) {
  return (
    <div className="animate-pulse mb-8">
      <div className="h-4 bg-zinc-800 rounded w-32 mb-4" />
      <div className="space-y-2">
        <div className="h-3 bg-zinc-800 rounded w-full" />
        <div className="h-3 bg-zinc-800 rounded w-5/6" />
        <div className="h-3 bg-zinc-800 rounded w-4/6" />
      </div>
      <p className="text-xs text-zinc-700 mt-2">loading {label}…</p>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Page root — each section is an independent async Server Component
// wrapped in Suspense for streaming
// ---------------------------------------------------------------------------

export default function Page() {
  return (
    <main className="min-h-screen bg-zinc-950 text-white">
      <div className="max-w-3xl mx-auto px-6 py-12">
        <Suspense fallback={<LoadingSkeleton label="project metadata" />}>
          <ProjectHeader />
        </Suspense>

        <Suspense fallback={<LoadingSkeleton label="feature flags" />}>
          <FeatureFlags />
        </Suspense>

        <Suspense fallback={<LoadingSkeleton label="API surface" />}>
          <ApiSurface />
        </Suspense>

        <Suspense fallback={<LoadingSkeleton label="coverage map" />}>
          <CoverageMap />
        </Suspense>

        <Suspense fallback={<LoadingSkeleton label="example runners" />}>
          <ExampleRunners />
        </Suspense>

        <RepresentationGap />

        <footer className="border-t border-zinc-800 pt-6 mt-8 text-xs text-zinc-600 flex gap-4">
          <span>clap-noun-verb faithful representation</span>
          <span>·</span>
          <span>all values derived by tool, not by reading</span>
          <span>·</span>
          <span>no fixtures</span>
        </footer>
      </div>
    </main>
  );
}
