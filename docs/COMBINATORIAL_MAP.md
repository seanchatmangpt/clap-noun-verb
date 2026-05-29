# Clap-Noun-Verb Combinatorial Feature Map & Gap Analysis

This document provides a rigorous, systems-level analysis of the interaction space between the six primary features of the `clap-noun-verb` framework. It maps out all theoretical user interactions, identifies functional incompatibilities and gaps in the current implementation, and provides a documentation audit against the existing Diátaxis-based directory.

---

## 1. Core Feature Catalog

To analyze the feature interactions, we define the six core capabilities ($F_1$ through $F_6$) as follows:

| ID | Feature | Description | Implementation Path |
|---|---|---|---|
| **$F_1$** | **Completions** | Dynamic shell completion script generation (`bash`, `zsh`, `fish`, `powershell`) with environment auto-detection. | `src/clap_ext/completions.rs` |
| **$F_2$** | **Chaining** | Multi-command step execution within a single CLI invocation using the `++` delimiter. | `src/registry.rs`, `src/cli/registry.rs` |
| **$F_3$** | **Preprocessing** | Argument expansion of stdin bindings (`@-`, `@-::json_path`) and step references (`@{step_index.json_path}`). | `src/cli/preprocessor.rs` |
| **$F_4$** | **Introspection** | Exporting the CLI command tree as a JSON Schema array of tools via the global `--introspect` flag. | `src/registry.rs`, `src/cli/registry.rs` |
| **$F_5$** | **REPL** | Interactive prompt loop featuring history persistence and custom noun/verb tab autocompletion. | `src/repl.rs` |
| **$F_6$** | **Error Handlers** | Structured JSON errors (`--structured-errors`), Levenshtein command-correction, and autonomic MAPE-K loops. | `src/error.rs` |

---

## 2. Combinatorial Interaction Matrix

The matrix below maps the compatibility of all pairwise feature combinations.

* **Compatible (Full)**: The features interact seamlessly as designed.
* **Compatible (Partial)**: The features can be used together but have design limitations or require special configuration.
* **Incompatible (None)**: The features are mutually exclusive, override one another, or fail to interact due to architectural limitations.

| | $F_1$ (Comp) | $F_2$ (Chain) | $F_3$ (Preproc) | $F_4$ (Intro) | $F_5$ (REPL) | $F_6$ (Errors) |
|---|---|---|---|---|---|---|
| **$F_1$ (Completions)** | — | **Partial** | **None** | **Full** | **Partial** | **Full** |
| **$F_2$ (Chaining)** | **Partial** | — | **Full** | **None** | **None** | **Partial** |
| **$F_3$ (Preprocessing)**| **None** | **Full** | — | **None** | **None** | **Partial** |
| **$F_4$ (Introspection)**| **Full** | **None** | **None** | — | **Partial** | **Full** |
| **$F_5$ (REPL)** | **Partial** | **None** | **None** | **Partial** | — | **Partial** |
| **$F_6$ (Error Handlers)**| **Full** | **Partial** | **Partial** | **Full** | **Partial** | — |

---

## 3. In-Depth Pairwise Analysis & Interaction Mapping

### $F_1$ (Completions) × $F_2$ (Chaining) — **Partially Compatible**
* **User Interaction**: A user types a chained command sequence: `myapp user create --name Alice ++ auth l[TAB]`.
* **System Behavior**: The shell completion script attempts to complete the token `l` as an option or argument of the initial command (`user create`) because the standard completion script generator does not understand the `++` delimiter as a command separator.
* **Gaps**: Shell autocompletion cannot reset its state machine back to the root command structure after encountering `++`.
* **Resolution**: Custom shell completion functions must be drafted to explicitly split the command line on `++` and complete only the current segment relative to the root CLI parser.

### $F_1$ (Completions) × $F_3$ (Preprocessing) — **Incompatible**
* **User Interaction**: A user types `myapp message send --body @-[TAB]` or `myapp user get --id @{1.[TAB]}`.
* **System Behavior**: The completion engine treats `@-` and `@{...}` as literal strings. It has no access to the `stdin` buffer or the runtime JSON outputs of prior steps.
* **Gaps**: There is no mechanism to autocomplete JSON paths dynamically from either standard input or previous execution outputs.
* **Resolution**: Document this as a fundamental limitation. Static help text should display the expected formats.

### $F_1$ (Completions) × $F_5$ (REPL) — **Partially Compatible**
* **User Interaction**: A user presses `[TAB]` inside the REPL loop vs inside the standard OS shell.
* **System Behavior**: 
  - Standard shell completions support options (`--flags`) and nouns/verbs.
  - The interactive REPL completions (`src/repl.rs`) utilize `rustyline` with a helper that *only* parses and suggests registered nouns and verbs, ignoring flags/options entirely.
* **Gaps**: Feature asymmetry. The REPL completer lacks option/flag suggestions and does not share the same completion engine as the shell generator.
* **Resolution**: Extend `ReplHelper` to parse partial options and suggest flags by inspecting the command structure at runtime.

### $F_2$ (Chaining) × $F_3$ (Preprocessing) — **Fully Compatible**
* **User Interaction**: `myapp auth login --user admin ++ user get --id "@{1.user_id}"`
* **System Behavior**: The routing layer reads all steps first, parses the first step, executes it, captures its JSON output, and uses the preprocessor to resolve the step-reference `@{1.user_id}` before invoking the second step.
* **Gaps**: None. This is the core workflow pattern of the framework.

### $F_2$ (Chaining) × $F_4$ (Introspection) — **Incompatible**
* **User Interaction**: `myapp auth login --user admin ++ --introspect`
* **System Behavior**: The global `--introspect` flag immediately dumps the JSON tool representation of the CLI registry and terminates execution, ignoring all other steps.
* **Gaps**: Introspection outputs are atomic tool lists. The framework does not expose step-chaining syntax (`++`) or sequence planning schemas to the introspected LLM toolset.
* **Resolution**: The LLM agent wrapper must manage chaining logic outside the binary, or a meta-schema must be provided that describes the chaining syntax.

### $F_2$ (Chaining) × $F_5$ (REPL) — **Incompatible**
* **User Interaction**: Inside the REPL, a user types: `auth login ++ user get`
* **System Behavior**: The REPL parses the input line using `split_shell_words` and passes the entire token list to `self.registry.build_command().try_get_matches_from(...)` as a single command. It does not split the arguments by `++`. As a result, `clap` fails to parse the arguments and throws a "command not found" or "invalid argument" error.
* **Gaps**: **Major Architecture Gap**. Chaining is completely unsupported/disabled inside the interactive REPL.
* **Resolution**: Refactor the REPL execution loop in `src/repl.rs` to reuse the `run_with_args` step-splitting and preprocessing logic instead of routing the matches directly.

### $F_2$ (Chaining) × $F_6$ (Error Handlers) — **Partially Compatible**
* **User Interaction**: A multi-step execution fails on step 2 due to a misspelled command: `myapp auth login ++ usr get`.
* **System Behavior**: The execution loop halts immediately at step 2. A structured error is returned for step 2. However, there is no rollback or cleanup mechanism for side-effects caused by step 1.
* **Gaps**: Chained execution lacks transactional rollback support or partial-failure error propagation context.
* **Resolution**: Document the non-transactional nature of chained commands. For autonomic applications, the orchestrator must manually handle state recovery using the returned structured error details.

### $F_3$ (Preprocessing) × $F_4$ (Introspection) — **Incompatible**
* **User Interaction**: An LLM inspects the introspected JSON Schema.
* **System Behavior**: The JSON Schema generated by `--introspect` describes parameters as simple types (e.g. `string`). It does not declare support for stdin binding formats (`@-`) or step reference notations (`@{step_index.json_path}`).
* **Gaps**: The LLM cannot discover that it is allowed to pass these special preprocessing tokens unless explicitly hardcoded into its system prompt.
* **Resolution**: Inject documentation/descriptions into the introspected JSON parameter schemas highlighting that `@-` and step references are supported.

### $F_3$ (Preprocessing) × $F_5$ (REPL) — **Incompatible**
* **User Interaction**: A user pipes data or types a step reference inside the REPL: `user create --name @-`
* **System Behavior**: Since the REPL bypasses the preprocessing step-splitting execution engine, stdin binds and step references are passed raw to the handler, resulting in silent failures or invalid data.
* **Gaps**: **Major Architecture Gap**. Stdin bindings and step references do not resolve inside the interactive REPL.
* **Resolution**: Integrate `preprocessor::preprocess_args` into the REPL line execution step.

### $F_3$ (Preprocessing) × $F_6$ (Error Handlers) — **Partially Compatible**
* **User Interaction**: A step reference `@{1.non_existent_key}` fails to resolve, or stdin contains invalid JSON.
* **System Behavior**: The preprocessor silently replaces the expression with an empty string (`""`) and continues execution. This leads to invalid command routing or validation failures later in the chain.
* **Gaps**: Resolution failures do not raise structured errors immediately.
* **Resolution**: Modify `preprocess_args` to return an explicit `NounVerbError::ValidationFailed` or `NounVerbError::ArgumentError` if a reference or JSON path is invalid, allowing the MAPE-K loop to catch it early.

### $F_5$ (REPL) × $F_6$ (Error Handlers) — **Partially Compatible**
* **User Interaction**: A command executed inside the REPL loop fails.
* **System Behavior**: The REPL prints the human-readable display error to `stderr` and continues the prompt loop. If structured errors are enabled globally, they are printed as JSON, which can clutter the screen. Autonomic corrections (`action_templates`) are not executed automatically.
* **Gaps**: The REPL does not hook into structured error handling to prompt the user with interactive suggestions (e.g. "Did you mean: `service`? [Y/n]").
* **Resolution**: When an error occurs in the REPL, check if it contains `action_templates` and prompt the user to auto-execute corrections.

---

## 4. High-Order Interactions (Multi-Feature Scenarios)

### Scenario A: Autonomous Agent Loop ($F_3 \times F_4 \times F_6$)
* **Description**: An AI agent fetches the tool schema via `--introspect` ($F_4$), plans a workflow using stdin piping ($F_3$), but experiences an argument validation error triggering structured error feedback with Levenshtein correction templates ($F_6$).
* **Critical Path**:
  1. Agent reads schema $\rightarrow$ 2. Agent builds arguments with `@-::path` $\rightarrow$ 3. Execution fails $\rightarrow$ 4. Structured MAPE-K error returns corrective suggestion.
* **Required Guides**: Guide detailing how agents should parse structured errors to auto-retry parameters.

### Scenario B: Interactive Pipeline Debugging ($F_2 \times F_3 \times F_5$)
* **Description**: A developer opens the REPL ($F_5$) to iteratively test a chained pipeline ($F_2$) using inputs resolved from stdin ($F_3$).
* **Constraint**: This scenario currently **fails** completely due to the mutual incompatibilities between REPL, Chaining, and Preprocessing.
* **Required Guides**: A troubleshooting guide explaining that chaining and preprocessing are strictly CLI-mode features and cannot be executed inside the current REPL.

---

## 5. Documentation Gap Audit

An audit of the current `docs/` directory reveals the following coverage and gaps:

| Document / Path | Covered Features | Gaps Identified |
|---|---|---|
| `docs/reference/api/advanced-features.md` | $F_1$, $F_2$, $F_3$, $F_4$, $F_5$ | Lists features individually; does not document any cross-feature interactions or limitations. |
| `docs/reference/api/errors.md` | $F_6$ | Does not discuss how structured errors return during chained step execution. |
| `docs/tutorial/` | Basic CLI creation | No tutorials covering **Chaining** ($F_2$), **Preprocessing** ($F_3$), or **REPL** ($F_5$) setups. |
| `docs/howto/validation.md` | Basic validator code | Does not cover how validation errors propagate during chained execution or preprocessing. |

### Missing Specific Guides Needed:
1. **How-To: Interactive REPL Limitations**: Documentation explaining the lack of chaining (`++`) and preprocessing support inside the REPL, providing shell-based alternatives.
2. **How-To: Autonomic Healing with Chaining**: Guide detailing how to handle partial step execution failures, parse `action_templates`, and recovery logic.
3. **Explanation: CLI Pipeline Execution Model**: Deep dive explaining the routing sequence (parsing $\rightarrow$ stdin resolution $\rightarrow$ step splitting $\rightarrow$ preprocessing $\rightarrow$ execution).

---

## 6. Actionable Implementation Recommendations

1. **REPL Refactoring**: Modify `src/repl.rs` to perform argument preprocessing and support step chaining (`++`).
2. **Robust Preprocessor Errors**: Replace silent failures (replacing with `""`) in `src/cli/preprocessor.rs` with descriptive errors (`NounVerbError::ArgumentError`).
3. **Interactive REPL Auto-Corrections**: Hook Levenshtein suggestions from `action_templates` into the REPL loop to offer interactive tab/y-n prompt corrections.
