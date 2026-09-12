# Learning with Copilot

Use an AI tool as a patient pair programmer, not as the authority on Topcoat. Give it the repository rules, the exact compiler output, and the pinned version. Then compile and test every suggestion yourself.

> [!WARNING]
> Topcoat is pre-1.0. The API surface changes between releases, and an AI model may confidently combine examples from different versions. Read the documentation for the pinned release tag, never `main`: for this workshop that means [Topcoat v0.8.0 on docs.rs](https://docs.rs/topcoat/0.8.0/) and the [v0.8.0 source guides](https://github.com/tokio-rs/topcoat/tree/v0.8.0).

## A reliable prompt has four parts

When you ask Copilot to change code, include:

1. **The goal.** Say what the page or interaction should do.
2. **The boundary.** State whether data must stay server-rendered, whether a browser-only signal is enough, and which endpoint may run.
3. **The evidence.** Paste the compiler error, the relevant function, and the pinned version.
4. **The check.** Ask for a minimal change, then name the test or build command that proves it.

A useful opening is:

> You are working in this Topcoat Workshop repository. Follow `AGENTS.md`. The project is pinned to Topcoat 0.8.0 and Toasty 0.10.0. Read v0.8.0 documentation, never `main`. Topcoat is server-rendered: do not move page data into browser storage, a client bundle, or a separate API layer. Explain the smallest change, show why it matches the compiler error, and give the existing validation command.

Do not ask for “the current Topcoat syntax” without the version. That prompt invites an answer assembled from unrelated releases.

## Ask it to explain a `view!` error

Start with the smallest failing region. Include the surrounding Rust signature because ownership and return-type errors often come from outside the markup.

For example, Lab 07 deliberately keeps an unsupported runtime expression as a comment:

```rust
{{#include ../../../labs/lab-07-signals-expressions/solution/src/app/_marketing.rs:unsupported-expression}}
```

Use a prompt like this:

> Explain this Topcoat 0.8.0 `view!` compiler error as if I am a Rust web developer who knows HTML but not Topcoat's runtime-expression vocabulary. First identify whether the error comes from ordinary server-side Rust or from `$(...)` cross-compilation. Then point to the exact unsupported syntax, explain why the expression must work in both Rust and JavaScript, and propose two v0.8.0-compatible rewrites. Do not invent an API. Check the release-tagged `expr!` guide. End with the smallest code change and the command I should run to verify it.

A good answer distinguishes these cases:

- Ordinary Rust inside `view!` can use normal Rust control flow and types.
- `$(...)` must belong to the shared runtime vocabulary because Topcoat evaluates it on the server and cross-compiles it for the browser.
- A signal expression can update browser-local state, but it cannot query a database or observe a later server change.
- A captured server value is a render-time snapshot. Use a shard when fresh server markup is required.

The answer should not “fix” a runtime error by moving all logic into JavaScript or by suggesting `localStorage`. That would violate this workshop's server-rendered model.

When the error is unfamiliar, ask the tool to classify it before editing:

> Classify this diagnostic as (a) Rust ownership/type checking, (b) `view!` markup syntax, or (c) Topcoat runtime-expression code generation. Quote the relevant v0.8.0 guide section and explain what evidence in the error supports your classification. Do not propose a workaround until the classification is clear.

Then run the smallest relevant check, usually the lab's package tests:

```bash
cargo test -p lab07-solution
```

## Ask it to generate a shard

A shard is a server endpoint that returns a fragment. It is not a client-side component and it does not automatically inherit page or layout authorization. Ask for the data boundary, input validation, and endpoint test explicitly.

Lab 08 provides a compact model:

```rust
{{#include ../../../labs/lab-08-shards-procedures-streaming/solution/src/app/_marketing/vessels.rs:vessel-results-shard}}
```

The page creates the browser signal in its body and passes the *handle* to the shard:

```rust
{{#include ../../../labs/lab-08-shards-procedures-streaming/solution/src/app/_marketing/vessels.rs:vessel-search-page}}
```

Use a prompt with concrete acceptance criteria:

> Generate a Topcoat 0.8.0 shard for this requirement: render a server-side HTML fragment containing the vessels whose names contain the current search string. The page already creates `let query = signal(cx, String::new);` in its body and should call the shard with `vessel_results(query: $(query))`. Accept a `Signal<String>` parameter, trim the value, reject or render a clear message for input longer than 80 characters, and return `Result<impl View>`. Keep the database or repository lookup on the server. Do not use `localStorage`, a WASM bundle, an invented `Shard` trait, or a separate JSON API. Repeat any authorization needed by the endpoint. Show the source file, the page call, and the integration-test cases. Verify all names against the v0.8.0 `#[shard]` guide.

> [!IMPORTANT]
> `$(query)` and `$(query.get())` are both valid Rust, and they mean different things. `$(query)`
> passes the signal **handle** to a `Signal<String>` parameter, so the runtime re-renders only that
> shard when the value changes. `$(query.get())` passes the current **value**, which makes the
> *caller* track the signal — the whole page body re-runs instead. An AI tool trained on pre-0.8
> examples will often produce the second form. (v0.8.0 `runtime.md`, §Shards and §Reading signals on
> the server.)

Before accepting generated code, check:

- `#[shard]` is imported from the pinned runtime API.
- The argument type is supported by the runtime endpoint and matches the call site. A `Signal<T>` parameter takes `$(signal)`, not `$(signal.get())`.
- The function returns a fragment view, not a full document layout.
- Validation happens inside the shard, because callers can invoke the endpoint directly.
- Authorization is repeated inside the shard when the data is private.
- The page owns the signal, so the state survives: 0.8.0 morphs the shard's new markup into the existing DOM rather than replacing it, which keeps focus and partly-typed input alive. Reorderable list items need a stable `id` for the morph to track them.
- Tests call the endpoint or exercise the rendered page and assert the fragment behaviour.

Ask for a diff rather than a rewrite when the lab already has working code:

> Modify only the existing vessel search. Preserve its route, shared data function, anchors, and tests. Add the smallest shard needed for server-side filtering. Show a unified diff and explain every changed line. Do not refactor unrelated code.

## Ask it to investigate a failing suggestion

When generated code does not compile, give the tool the new evidence instead of asking it to guess again:

> This suggestion fails against Topcoat 0.8.0. Here is the exact compiler output: `<paste compiler output>`. Here is the source at `<path>`. Compare the suggestion with the release-tagged v0.8.0 guide and identify the first incorrect assumption. Do not switch to a newer API or change the dependency version. Return either a minimal patch or state that the requested behaviour is unavailable in v0.8.0.

That last sentence matters. “Unavailable in this version” is a useful result; an invented compatibility layer is not.

## Where AI tools hallucinate

Pre-1.0 frameworks create several predictable traps:

| Trap | What an AI tool may claim | How you verify it |
|---|---|---|
| Version drift | A `main`-branch macro, extractor, or helper exists in 0.8.0 | Read the v0.8.0 docs.rs page and release-tagged guide |
| Pre-0.8 signal syntax | A signal is declared as a statement inside `view!`, as `signal query = String::new();` | Compile it. 0.8.0 fails with `error: expected view node`. Signals are created with `signal(cx, String::new)` in the component body, and the component takes `cx: &Cx` (v0.8.0 `runtime.md`, §Signals) |
| Passing a value where a handle belongs | `$(query.get())` is the way to give a shard a signal | `$(query.get())` makes the caller track the value and re-runs the page; a `Signal<T>` shard parameter takes `$(query)` |
| Familiar framework transfer | Topcoat has Axum middleware, a client router, or a Leptos-style component model | Check the pinned API and the lab's existing pattern |
| Runtime-expression overreach | Any Rust expression works inside `$(...)` | Read the v0.8.0 `expr!` shared vocabulary and compile the lab |
| Browser-state shortcut | Put server data in `localStorage`, a client bundle, or a separate API | Apply the repository rule: page data stays server-rendered |
| Endpoint security omission | A shard or procedure inherits the page's guard automatically | Invoke the endpoint directly and repeat validation/authentication inside it |
| Streaming confusion | `live!` is a later request like a shard, or headers can change after streaming starts | Compare the v0.8.0 `live!` guide with the reactivity lab |
| Tooling drift | `topcoat --version`, `topcoat fmt --check`, or `topcoat build` exists | Read `topcoat --help`, the CLI reference, and `COMPATIBILITY.md` |

Treat links to `main` as a warning sign, not as proof. The workshop deliberately records version-specific breakages and workarounds in [`COMPATIBILITY.md`](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/COMPATIBILITY.md).

> [!NOTE]
> The date and version matter. A correct answer for a future Topcoat release can still be wrong for this lab. Pin the question, read the tag, and compile the result.

## Ground an AI tool with repository evidence

You can make answers more reliable by giving the tool local evidence in a short sequence:

1. Open `AGENTS.md`, `COMPATIBILITY.md`, and the target lab README.
2. Show the relevant solution file and its integration test.
3. State the exact package command you will run.
4. Ask for an explanation before asking for an edit.
5. Review the diff, run the command, and paste failures back into the conversation.

A useful review prompt is:

> Review this proposed patch against `AGENTS.md`, `PLAN.md`, `COMPATIBILITY.md`, and the v0.8.0 Topcoat guide. Check server-rendering boundaries, endpoint authorization, supported runtime expressions, source includes, formatting, and test coverage. Report only concrete mismatches with file and line references. Do not redesign working code.

For a larger change, ask the tool to produce a plan first and keep implementation separate. For a small compiler error, direct evidence is faster than a broad architectural prompt.

## A short loop you can reuse

Use this loop for every AI-assisted change:

```text
Read the repository rules.
Pin the framework version.
Show the smallest relevant source and the exact error.
Ask for an explanation.
Ask for a minimal patch.
Run the existing formatter, build, or test.
Review the diff and the generated behaviour.
Record any version-specific workaround in COMPATIBILITY.md.
```

Copilot helps you explore unfamiliar Rust and Topcoat. You remain responsible for checking the release tag, the generated diff, the HTTP behaviour, and the tests.

**See also:** [Rust in 30 min for web devs](rust-for-web-devs.md), [HTML/HTTP in 30 min for Rustaceans](web-for-rustaceans.md), [How `$(...)` reaches the browser](../01-concepts/dual-expressions.md), [Shards, procedures, live regions, htmx](../01-concepts/reactivity-options.md), [Topcoat CLI commands](../03-reference/cli.md), and [How-to: pin and upgrade Topcoat](../03-reference/howto/upgrade.md).
