"use client";

import { useActionState, startTransition } from "react";
import { runExampleAction, type RunExampleState } from "@/app/actions";

const INITIAL_STATE: RunExampleState = { result: null, error: null };

/**
 * ExampleRunner — Client Component wired to a Server Action.
 *
 * Uses React's `useActionState` + `startTransition` pattern (Next.js 16 / React 19).
 * The form submission POSTs to the server action `runExampleAction`, which runs
 * `cargo run --example <name>` server-side and returns real stdout.
 *
 * No manual fetch(), no client-side state machine — the server action owns
 * the mutation lifecycle. `pending` comes from React's transition system.
 */
export function ExampleRunner({ name }: { name: string }) {
  const [state, action, pending] = useActionState(runExampleAction, INITIAL_STATE);

  return (
    <div className="border border-zinc-700 rounded-lg p-4 bg-zinc-900">
      <form
        action={action}
        onSubmit={(e) => {
          // Wrap in startTransition so React treats this as a non-urgent update
          // and the pending state is reflected immediately.
          e.preventDefault();
          const form = e.currentTarget;
          startTransition(() => action(new FormData(form)));
        }}
        className="flex items-center justify-between mb-3"
      >
        <code className="text-amber-400 text-sm font-mono">cargo run --example {name}</code>
        <input type="hidden" name="name" value={name} />
        <button
          type="submit"
          disabled={pending}
          className="px-3 py-1 text-xs bg-amber-500 hover:bg-amber-400 disabled:bg-zinc-600 text-black font-semibold rounded transition-colors"
        >
          {pending ? "running…" : "▶ Run"}
        </button>
      </form>

      {state.result && (
        <div>
          <pre className="text-green-400 text-xs font-mono whitespace-pre-wrap bg-black rounded p-3 mt-2 leading-relaxed">
            {state.result.output || "(no stdout)"}
          </pre>
          <div className="mt-2 flex gap-4 text-xs text-zinc-500">
            <span>exit {state.result.exitCode}</span>
            <span>{state.result.durationMs}ms</span>
            <span className="text-zinc-700">via server action</span>
          </div>
        </div>
      )}

      {state.error && (
        <pre className="text-red-400 text-xs font-mono bg-black rounded p-3 mt-2 whitespace-pre-wrap">
          {state.error}
        </pre>
      )}
    </div>
  );
}
