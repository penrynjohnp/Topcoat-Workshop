# Rust in 30 min for web devs

You do not need to learn all of Rust before starting these labs. You need six ideas: ownership, borrowed and owned strings, `Result`, `async`, traits, and closures.

This page gives you the minimum working model. Read it once, then return when the compiler points at one of these concepts.

## Minutes 0–6: ownership and borrowing

JavaScript values are garbage-collected. Rust instead tracks who owns each value and rejects code that could use a value after it has been moved or while it is borrowed incompatibly.

A variable usually owns its value. Passing it by value transfers ownership unless the type implements `Copy`. Prefixing a type with `&` borrows the value instead.

Lab 02 shows both forms:

```rust
{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:nav}}
```

`current_path: &str` borrows text. The component can read it without owning or freeing it. The `for` loop copies the two `&str` references from the constant array because references are cheap, copyable values.

Compare that with Lab 02's berth list:

```rust
{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:berth_list}}
```

`berths: Vec<Berth>` passes the vector by value. The component owns it, and `for berth in berths` consumes the vector into its elements. That is appropriate because the component does not need to return the collection to its caller.

Use this rule of thumb:

| You need to… | Start with… |
|---|---|
| Read a value temporarily | `&T` |
| Change a borrowed value | `&mut T` |
| Give a function ownership | `T` |
| Keep data beyond the current call | an owned type such as `String` or `Vec<T>` |

> [!TIP]
> Do not add `.clone()` automatically whenever the borrow checker objects. First decide which function should own the value. Clone only when two owners genuinely need independent values.

## Minutes 6–10: `&str` versus `String`

Both types represent UTF-8 text, but they answer different ownership questions:

- `&str` is a borrowed view into text owned elsewhere.
- `String` owns growable text allocated at runtime.
- A string literal such as `"A1"` has type `&'static str`; it lives for the whole program.

Lab 02's model stores literals but constructs a URL at runtime:

```rust
{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:model}}
```

The berth fields can use `&'static str` because the seed data contains fixed literals. `url` returns `String` because `format!` creates new text that must outlive the function's local expression.

Accept `&str` when a function only needs to inspect text. Accept or return `String` when the function must own, store, or construct it. Convert deliberately:

| Conversion | Meaning |
|---|---|
| `owned.as_str()` or `&owned` | borrow a `String` as `&str` |
| `borrowed.to_owned()` | create an owned `String` from `&str` |
| `format!(...)` | construct an owned `String` |

## Minutes 10–15: `Option`, `Result`, and `?`

Rust makes absence and failure visible in types:

- `Option<T>` is either `Some(T)` or `None`.
- `Result<T, E>` is either `Ok(T)` or `Err(E)`.
- Topcoat's `topcoat::Result<T>` fixes the error side to Topcoat's error type.

Lab 06 turns an optional current user into either an email or a redirect error:

```rust
{{#include ../../../labs/lab-06-auth-sessions-mail/solution/src/auth.rs:require-auth}}
```

The caller cannot accidentally treat authentication failure as a valid `String`. It must return, propagate, or handle the error.

The `?` operator is the concise propagation form. Lab 06 uses it throughout session operations:

```rust
{{#include ../../../labs/lab-06-auth-sessions-mail/solution/src/auth.rs:session-lifecycle}}
```

In `session::start(cx).await?`, `?` unwraps `Ok(value)`. If the result is `Err(error)`, the current function returns that error immediately. Therefore, you use `?` only inside a function whose return type can represent the propagated failure.

When you see a compiler error around `?`, check the enclosing function's return type before changing the expression.

## Minutes 15–20: `async` and `.await`

An `async fn` can pause while work is pending instead of blocking the executor thread. Calling it produces a future. `.await` drives that future until its next result is available.

Topcoat page functions and components are async even when today's body performs no I/O. This leaves room for request context, database calls, sessions, and rendering work without changing the function shape later:

```rust
{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:page}}
```

Read an async signature from right to left:

```text
async fn home() -> Result<impl View>
```

It eventually produces either a value implementing `View` or an error. Inside another async function, you call async work with `.await`, then commonly propagate failure with `?`.

At the executable boundary, Lab 01 uses `#[tokio::main]` to create the async runtime and waits for Topcoat's server:

```rust
{{#include ../../../labs/lab-01-hello-topcoat/solution/src/main.rs}}
```

Do not insert blocking filesystem, network, or sleep calls into async request code. Use async APIs so other requests can progress while the operation waits.

## Minutes 20–25: traits and `impl Trait`

A trait is a shared capability, similar to an interface. A type can implement multiple traits, and generic code can ask for a capability rather than a concrete class hierarchy.

Lab 05's application state implements `Default`, which means callers can construct it through the standard `AppState::default()` capability:

```rust
{{#include ../../../labs/lab-05-cx-memoize/solution/src/shared.rs:app-state}}
```

You encounter `impl Trait` frequently in Topcoat signatures. `Result<impl View>` means “success contains one concrete type that implements `View`, without exposing that type's long generated name.” It is not dynamic typing; the compiler still knows and checks the concrete type.

Lab 06 returns `impl Cookies` for the same reason:

```rust
{{#include ../../../labs/lab-06-auth-sessions-mail/solution/src/auth.rs:cookie-access}}
```

The caller receives something with the `Cookies` operations while the function keeps the exact chain of cookie wrappers private.

Trait methods may require the trait to be imported. If the compiler says a method exists through a trait that is not in scope, add the suggested `use` rather than rewriting the call.

## Minutes 25–30: closures and iterator chains

A closure is an unnamed function that can capture values from its surrounding scope. Its syntax uses pipes for parameters:

```rust
{{#include ../../../labs/lab-05-cx-memoize/solution/src/shared.rs:closure-find}}
```

`|berth| berth.slug == slug` borrows `slug` from the enclosing function. `find` calls the closure for each berth and returns the first match as `Some(Berth)`, or `None`.

The full chain reads left to right:

1. `berths()` creates a `Vec<Berth>`.
2. `into_iter()` consumes it and yields owned `Berth` values.
3. `find(...)` tests each value with the closure.
4. The function returns `Option<Berth>`.

Common iterator methods include `map` to transform values, `filter` to retain matches, and `collect` to build a collection. Prefer a clear `for` loop when the chain stops being easy to say aloud.

## Reading compiler messages during the labs

Rust's error usually identifies both the rejected use and the earlier move or borrow that caused it. Read the first error completely, fix it, then compile again; later errors can be consequences of the first.

Keep these translations nearby:

| Compiler wording | First question to ask |
|---|---|
| “use of moved value” | Which function should own this value? |
| “cannot borrow as mutable” | Is the binding and reference actually `mut`/`&mut`? |
| “expected `String`, found `&str`” | Does this code need ownership or only a borrow? |
| “the `?` operator can only be used…” | Does the enclosing function return `Result` or `Option`? |
| “future is not `Send`” | Is a non-thread-safe borrow or lock held across `.await`? |
| “method not found” with a trait suggestion | Is the providing trait imported? |

You now have enough Rust to start Lab 01. Let the types guide you, and learn additional language features when a lab gives you a reason to use them.

**See also:** [Lab 01](../02-labs/lab-01.md), [Lab 02](../02-labs/lab-02.md), [Lab 05](../02-labs/lab-05.md), [Lab 06](../02-labs/lab-06.md), [view! cheat sheet](../03-reference/view-cheatsheet.md), and [Cx API summary](../03-reference/cx.md).
