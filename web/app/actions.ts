"use server";

import { runExample, type ExampleRun } from "@/lib/project";

export interface RunExampleState {
  result: ExampleRun | null;
  error: string | null;
}

/**
 * Server Action: run a named cargo example and return real stdout.
 * Invoked from <form action={runExampleAction}> — uses the POST method
 * under the hood; no client-side fetch needed.
 */
export async function runExampleAction(
  _prev: RunExampleState,
  formData: FormData,
): Promise<RunExampleState> {
  const name = formData.get("name");
  if (typeof name !== "string" || !/^[a-z0-9_-]+$/.test(name)) {
    return { result: null, error: "invalid example name" };
  }
  try {
    const result = runExample(name);
    return { result, error: null };
  } catch (e: unknown) {
    const err = e as { message?: string };
    return { result: null, error: err.message ?? String(e) };
  }
}
