# What Toasty does

Toasty is a relational data layer that derives typed queries and relationships from Rust models. It is not a client-side store and it does not add an API tier: a Topcoat component reads `toasty::Db` from `Cx`, executes a query on the server, and renders HTML from the result.

## Models describe the graph

A foreign-key field and `#[belongs_to]` connect each child to its parent. `#[has_many]` adds the inverse relation, while `Deferred<T>` lets each query choose whether to load it.

```rust
{{#include ../../../labs/lab-10-toasty-sqlite/solution/src/models.rs:toasty-models}}
```

## Queries stay near rendering

`toasty::Db` is cheap to clone. Operations take `&mut Db`, so request code clones the shared handle locally and uses generated filters or relation accessors.

```rust
{{#include ../../../labs/lab-10-toasty-sqlite/solution/src/shared.rs:toasty-queries}}
```

Creating through `berth.work_orders()` fills the foreign key from the relationship. Updating a loaded model keeps mutation typed as well; no handwritten SQL or JSON boundary is required.

## Schema push is not migration

Lab 10 calls `push_schema()` only when its SQLite file is new:

```rust
{{#include ../../../labs/lab-10-toasty-sqlite/solution/src/database.rs:database-setup}}
```

That is appropriate for a disposable workshop database. Toasty 0.10.0 does not make repeated schema pushes idempotent on an existing file, and changing production schemas requires ordered migrations. Persistence also extends beyond domain rows: sessions and magic links can use ordinary Toasty models while the browser still receives only an encrypted cookie.

See the pinned [Toasty 0.10.0 documentation](https://docs.rs/toasty/0.10.0/toasty/) for the exact API used by this workshop.
