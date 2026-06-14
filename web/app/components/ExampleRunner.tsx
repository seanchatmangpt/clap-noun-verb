"use client";

import { useState } from "react";

interface ExampleRun {
  name: string;
  output: string;
  exitCode: number;
  durationMs: number;
}

export function ExampleRunner({ name }: { name: string }) {
  const [state, setState] = useState<"idle" | "running" | "done" | "error">("idle");
  const [result, setResult] = useState<ExampleRun | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function run() {
    setState("running");
    setResult(null);
    setError(null);
    try {
      const res = await fetch(`/api/run-example?name=${encodeURIComponent(name)}`);
      const json = await res.json();
      if (!res.ok) {
        setError(json.error ?? "unknown error");
        setState("error");
      } else {
        setResult(json);
        setState("done");
      }
    } catch (e) {
      setError(String(e));
      setState("error");
    }
  }

  return (
    <div className="border border-zinc-700 rounded-lg p-4 bg-zinc-900">
      <div className="flex items-center justify-between mb-3">
        <code className="text-amber-400 text-sm font-mono">cargo run --example {name}</code>
        <button
          onClick={run}
          disabled={state === "running"}
          className="px-3 py-1 text-xs bg-amber-500 hover:bg-amber-400 disabled:bg-zinc-600 text-black font-semibold rounded transition-colors"
        >
          {state === "running" ? "running…" : "▶ Run"}
        </button>
      </div>

      {state === "done" && result && (
        <div>
          <pre className="text-green-400 text-xs font-mono whitespace-pre-wrap bg-black rounded p-3 mt-2 leading-relaxed">
            {result.output || "(no stdout)"}
          </pre>
          <div className="mt-2 flex gap-4 text-xs text-zinc-500">
            <span>exit {result.exitCode}</span>
            <span>{result.durationMs}ms</span>
          </div>
        </div>
      )}

      {state === "error" && (
        <pre className="text-red-400 text-xs font-mono bg-black rounded p-3 mt-2 whitespace-pre-wrap">
          {error}
        </pre>
      )}
    </div>
  );
}
