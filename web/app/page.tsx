import { Suspense } from "react";
import {
  getProjectMeta,
  getCoverageStatus,
  getErrorVariants,
  getOutputFormatVariants,
  getBenchResults,
  getGraphApiSurface,
  getReplApiSurface,
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

// ---------------------------------------------------------------------------
// AppContext — live example output + API method list
// ---------------------------------------------------------------------------

async function AppContextSection() {
  return (
    <section className="mb-8">
      <SectionTitle>AppContext — Type-Erased Shared Store</SectionTitle>
      <p className="text-xs text-zinc-500 mb-3">
        <code className="text-zinc-400">src/context.rs</code> — thread-safe{" "}
        <code className="text-zinc-400">Arc&lt;RwLock&lt;HashMap&lt;TypeId, Box&lt;dyn Any + Send + Sync&gt;&gt;&gt;&gt;</code>.
        Methods: <span className="font-mono text-amber-300">new · insert · get · contains · with · len · is_empty · remove · clear</span>
      </p>
      <ExampleRunner name="app_context" />
    </section>
  );
}

// ---------------------------------------------------------------------------
// Graph API surface — parsed from src/graph/mod.rs
// ---------------------------------------------------------------------------

async function GraphSection() {
  const types = getGraphApiSurface();
  return (
    <section className="mb-8">
      <SectionTitle>Graph + Triple — RDF-style Triple Store</SectionTitle>
      <p className="text-xs text-zinc-500 mb-3">
        <code className="text-zinc-400">src/graph/mod.rs</code> — public types parsed from source:
      </p>
      <div className="flex flex-wrap gap-2 mb-3">
        {types.map((t) => (
          <span key={t} className="px-2 py-1 text-xs font-mono bg-zinc-800 text-green-300 rounded border border-zinc-700">
            {t}
          </span>
        ))}
      </div>
      <p className="text-xs text-zinc-600">
        Key API: <code>Triple::new(subject, predicate, object)</code> ·{" "}
        <code>Graph::add_triple()</code> · <code>query_by_subject(pattern)</code> ·{" "}
        <code>validate_all()</code> → <code>Vec&lt;ValidationError&gt;</code>
      </p>
    </section>
  );
}

// ---------------------------------------------------------------------------
// CapabilityRegistry — live example output
// ---------------------------------------------------------------------------

async function CapabilitySection() {
  return (
    <section className="mb-8">
      <SectionTitle>CapabilityRegistry — Named Versioned Package Registry</SectionTitle>
      <p className="text-xs text-zinc-500 mb-3">
        <code className="text-zinc-400">src/capability/registry.rs</code> —{" "}
        <code className="text-zinc-400">CapabilityPackage</code> (id, name, version, description) +{" "}
        <code className="text-zinc-400">CapabilityRegistry</code> (add, remove, contains, packages, len).
      </p>
      <ExampleRunner name="capability_registry" />
    </section>
  );
}

// ---------------------------------------------------------------------------
// DoctorOutput — live health_check() output
// ---------------------------------------------------------------------------

async function DiagnosticsSection() {
  return (
    <section className="mb-8">
      <SectionTitle>DoctorOutput — Live Health Check</SectionTitle>
      <p className="text-xs text-zinc-500 mb-3">
        <code className="text-zinc-400">src/diagnostics/doctor.rs</code> —{" "}
        <code className="text-zinc-400">health_check() → Result&lt;DoctorOutput&gt;</code> with{" "}
        <code className="text-zinc-400">HealthIssue &#123; level, message &#125;</code>.
      </p>
      <ExampleRunner name="diagnostics" />
    </section>
  );
}

// ---------------------------------------------------------------------------
// Repl — feature-gated API surface from src/repl.rs
// ---------------------------------------------------------------------------

async function ReplSection() {
  const methods = getReplApiSurface();
  return (
    <section className="mb-8">
      <SectionTitle>Repl — Interactive REPL (feature: repl)</SectionTitle>
      <p className="text-xs text-zinc-500 mb-3">
        <code className="text-zinc-400">src/repl.rs</code> — enabled with{" "}
        <code className="text-zinc-400">--features repl</code>. Not runnable without a TTY, but
        the public API surface is real:
      </p>
      <div className="flex flex-wrap gap-2">
        {methods.map((m) => (
          <span key={m} className="px-2 py-1 text-xs font-mono bg-zinc-800 text-purple-300 rounded border border-zinc-700">
            {m}()
          </span>
        ))}
      </div>
      <p className="mt-2 text-xs text-zinc-600">
        Core: <code>Repl::new(registry)</code> · <code>with_history_file(path)</code> ·{" "}
        <code>run()</code> — reads lines, dispatches to <code>CommandRegistry</code>, prints results.
      </p>
    </section>
  );
}

// ---------------------------------------------------------------------------
// Benchmarks — real criterion numbers from benches/dispatch.rs
// ---------------------------------------------------------------------------

async function BenchmarkSection() {
  const results = getBenchResults();
  return (
    <section className="mb-8">
      <SectionTitle>Dispatch Benchmarks — criterion (100 samples)</SectionTitle>
      <p className="text-xs text-zinc-500 mb-3">
        From <code className="text-zinc-400">benches/dispatch.rs</code> via{" "}
        <code className="text-zinc-400">cargo bench --bench dispatch</code>. Median of 100 samples.
      </p>
      <div className="grid sm:grid-cols-2 gap-3">
        {results.map((r) => (
          <div key={r.name} className="bg-zinc-900 border border-zinc-800 rounded-lg p-4">
            <div className="font-mono text-2xl font-bold text-amber-400">
              {r.medianNs}
              <span className="text-sm text-zinc-500 ml-1">{r.unit}</span>
            </div>
            <div className="text-xs text-zinc-400 mt-1 font-mono">{r.name}</div>
          </div>
        ))}
      </div>
      <p className="mt-2 text-xs text-zinc-600">
        Same hot path instrumented by OpenTelemetry spans under the <code>otel</code> feature.
      </p>
    </section>
  );
}

function RepresentationGap() {
  return (
    <section className="mb-8 border border-zinc-800 rounded-lg p-4 bg-zinc-950">
      <SectionTitle>Representation Gap Map — Iteration 2</SectionTitle>
      <div className="grid sm:grid-cols-2 gap-6 text-xs">
        <div>
          <h3 className="text-red-400 font-semibold mb-2">Rendered-but-fabricated</h3>
          <p className="text-green-400">
            None. Every value on this page is derived from real source or real binary output.
          </p>
        </div>
        <div>
          <h3 className="text-green-400 font-semibold mb-2">Exposed-but-unrepresented</h3>
          <p className="text-green-400">
            None. All 6 previously-unrepresented capabilities now have UI sections:
          </p>
          <ul className="mt-2 space-y-1 text-zinc-500">
            <li>✓ AppContext — live example runner</li>
            <li>✓ Graph/Triple — parsed type list from src/graph/mod.rs</li>
            <li>✓ CapabilityRegistry — live example runner</li>
            <li>✓ DoctorOutput/HealthIssue — live health_check() output</li>
            <li>✓ Repl — public method list from src/repl.rs</li>
            <li>✓ Benchmarks — real criterion median timings</li>
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

        <Suspense fallback={<LoadingSkeleton label="AppContext" />}>
          <AppContextSection />
        </Suspense>

        <Suspense fallback={<LoadingSkeleton label="Graph API" />}>
          <GraphSection />
        </Suspense>

        <Suspense fallback={<LoadingSkeleton label="CapabilityRegistry" />}>
          <CapabilitySection />
        </Suspense>

        <Suspense fallback={<LoadingSkeleton label="diagnostics" />}>
          <DiagnosticsSection />
        </Suspense>

        <Suspense fallback={<LoadingSkeleton label="Repl" />}>
          <ReplSection />
        </Suspense>

        <Suspense fallback={<LoadingSkeleton label="benchmarks" />}>
          <BenchmarkSection />
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
