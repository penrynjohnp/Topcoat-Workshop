# HTML/HTTP in 30 min for Rustaceans

You do not need to become a front-end specialist before starting these labs. You need a working model of five things: HTML documents, HTTP requests and responses, cookies, forms, and what the browser does after receiving a response.

Topcoat renders on the server. The browser receives ordinary HTML first, so the platform fundamentals remain visible even when you add signals, shards, procedures, or htmx.

## Minutes 0–6: HTML is a document, not a drawing API

HTML describes the meaning and structure of a document. The browser parses that text into a Document Object Model (DOM), applies its built-in behaviour, then paints the result.

Lab 02 returns a complete document:

```rust
{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:page}}
```

Read the structure from the outside inward:

- `<!DOCTYPE html>` selects the modern HTML parsing mode.
- `<html lang="en">` identifies the document language.
- `<head>` contains metadata and resources, not page content.
- `<meta charset="utf-8">` declares the text encoding.
- `<title>` names the browser tab and history entry.
- `<body>` contains the rendered page.
- `<header>`, `<nav>`, `<main>`, and `<footer>` communicate document landmarks.

Elements can have attributes. An attribute such as `href="/berths"` configures an element's behaviour, while `class="nav-link"` supplies tokens commonly used by CSS and scripts. An `id` must identify one element in the document; labels, fragment links, scripts, and accessibility relationships can refer to it.

HTML elements also have native semantics. An `<a href="…">` navigates. A `<button>` activates an action. A `<form>` submits controls. Prefer those elements over a generic `<div>` with custom click behaviour because keyboard, accessibility, and no-JavaScript behaviour come with the platform.

Topcoat's `view!` macro produces this HTML from checked Rust-shaped markup. Interpolated text is rendered as content rather than treated as new markup, which keeps data separate from document structure.

## Minutes 6–12: HTTP is one request followed by one response

The browser sends an HTTP request containing a method, target URL, headers, and sometimes a body. The server returns a status code, headers, and a body.

The method states the request's intent:

| Method | Use it for | Typical browser source |
|---|---|---|
| `GET` | Read a page or resource | link, address bar, GET form |
| `POST` | Submit data or request a state change | POST form, procedure, script |

A URL has separate parts. In `/berths/a1?tab=history#latest`, `/berths/a1` is the path, `tab=history` is the query string sent to the server, and `latest` is the fragment handled by the browser and normally omitted from the HTTP request.

Lab 04 maps a path segment into a typed request value and returns a `404 Not Found` error when no berth matches:

```rust
{{#include ../../../labs/lab-04-routing/solution/src/app/_marketing/berths/id.rs:path-param}}
```

Status codes are part of the response contract:

| Range | Meaning | Examples used in these labs |
|---|---|---|
| `2xx` | request succeeded | `200 OK` page or JSON response |
| `3xx` | client should follow another location | `303 See Other` after a form, redirect to login |
| `4xx` | request is invalid or unavailable to this caller | `404 Not Found` |
| `5xx` | server failed to fulfil a valid request | unhandled server error |

A route can return data instead of an HTML document. Lab 04's health endpoint serializes a Rust value as JSON:

```rust
{{#include ../../../labs/lab-04-routing/solution/src/app/_marketing/api/health.rs:health-route}}
```

The response's content type tells the client how to interpret the body. A JSON health response is not a separate page-data architecture; it is one operational route alongside server-rendered pages.

> [!TIP]
> Treat the method, path, status, headers, and body as one observable interface. A response body that looks correct can still be wrong if it uses the wrong status or content type.

## Minutes 12–17: cookies make later requests carry small pieces of state

HTTP requests are independent. A cookie lets a server ask the browser to send a small name/value pair on later matching requests.

The server writes a `Set-Cookie` response header. The browser stores the cookie and later adds it to the `Cookie` request header when the cookie's domain, path, expiry, and security rules match. JavaScript does not need to copy it between requests.

Lab 06 creates a cookie interface with signing, encryption, and defaults:

```rust
{{#include ../../../labs/lab-06-auth-sessions-mail/solution/src/auth.rs:cookie-access}}
```

The important attributes are:

- `HttpOnly` prevents browser JavaScript from reading the cookie.
- `Secure` restricts it to HTTPS requests.
- `SameSite` limits when cross-site navigations or requests carry it.
- `Path` limits which paths receive it.
- expiry or maximum age determines whether it persists beyond the browser session.

Signing detects modification; it does not hide the value. Encryption hides the value from the browser user as well. Neither makes arbitrary client-provided data trustworthy without validation.

Lab 06 writes a remembered email as owned cookie data:

```rust
{{#include ../../../labs/lab-06-auth-sessions-mail/solution/src/auth.rs:cookie-write}}
```

A cookie is not the same thing as a server-side session. In these labs, the browser carries an opaque session token while the server stores the associated user and expiry. Logging out removes or invalidates that server-side record, so replaying an old token does not recreate the session.

> [!WARNING]
> Cookies are attached according to browser rules, not according to which button created the request. Protect state-changing endpoints against cross-site requests, keep authentication cookies `HttpOnly` and `Secure` in production, and repeat authorization in every independently callable endpoint.

## Minutes 17–24: forms turn controls into an HTTP request

A form is the browser's built-in request builder. Its `action` selects the target URL and its `method` selects `GET` or `POST`. Successful controls contribute `name=value` pairs; an `id` alone does not submit a value.

Lab 10 renders a POST form with labels, stable names, retained values, and server-rendered validation messages:

```rust
{{#include ../../../labs/lab-10-toasty-sqlite/solution/src/app/_marketing/dashboard/work_orders/new.rs:work-order-form}}
```

Several details matter:

- A `<label for="work-order-title">` targets the control whose `id` matches.
- `name="title"` becomes the key decoded by the server.
- `value` restores submitted text when validation fails.
- A submit button submits the containing form; `type="button"` does not.
- `aria-invalid` exposes validation state to assistive technology.
- `novalidate` disables built-in constraint validation here because the lab demonstrates server validation explicitly.

The server must validate every submitted value. Browser constraints improve usability, but callers can omit fields, alter values, or send the HTTP request without using the form.

Lab 10 decodes the form, validates it, writes only valid data, and returns either the form again or a redirect:

```rust
{{#include ../../../labs/lab-10-toasty-sqlite/solution/src/app/_marketing/dashboard/work_orders/new.rs:manual-form-validation}}
```

The success response uses `303 See Other` with a `Location` header. The browser follows it with a `GET`, producing the Post/Redirect/Get pattern. Refreshing the destination then repeats the safe `GET` instead of resubmitting the original `POST`.

A GET form puts its fields in the query string. That makes searches bookmarkable and shareable. Do not put passwords, tokens, or other sensitive values in a GET query because URLs are commonly retained in history, logs, and links.

## Minutes 24–30: the browser has default behaviour before JavaScript

After receiving HTML, the browser parses it, loads referenced resources, builds the DOM, and enables native interactions. Links navigate, forms submit, focus moves, the back button traverses history, and controls expose keyboard behaviour without a framework.

A full navigation replaces the current document with the next response. Server-rendered state must therefore come from the URL, cookies, session, database, or response; an arbitrary DOM change does not survive a reload.

Lab 07 adds a browser-local signal to already rendered HTML:

```rust
{{#include ../../../labs/lab-07-signals-expressions/solution/src/app/_marketing.rs:work-order-panel}}
```

The server produces the initial button and panel. The Topcoat runtime then handles the click in the browser, changes attributes and text, and does not contact the server because the answer is already present. A later navigation still requests and renders a new document.

Progressive enhancement starts with working HTML and adds interception. Lab 09's vessel search remains a real GET form, but htmx can intercept input events, request an HTML fragment, and replace one target instead of navigating the whole page:

```rust
{{#include ../../../labs/lab-09-htmx-alpine/solution/src/app/_marketing/vessels/htmx.rs:htmx-search-page}}
```

If JavaScript is unavailable, submitting the form still navigates to `/vessels/htmx?q=…`. Because the server reads that URL and renders the same initial results, reload, sharing, and browser history remain meaningful.

Keep these boundaries clear:

| Behaviour | Runs where? |
|---|---|
| Route matching, database access, authorization, initial `view!` render | server |
| HTML parsing, native links/forms, focus, history | browser |
| `signal` and `$(...)` updates | browser after initial render |
| shard, procedure, or htmx request handler | server after another HTTP request |

Use the browser's Network panel when behaviour is unclear. It shows whether an interaction caused a request, which method and URL it used, what cookies and headers were involved, and which status and body came back.

You now know enough web platform behaviour to reason through the labs without treating the browser as a black box.

**See also:** [Lab 02](../02-labs/lab-02.md), [Lab 04](../02-labs/lab-04.md), [Lab 06](../02-labs/lab-06.md), [Lab 07](../02-labs/lab-07.md), [Lab 09](../02-labs/lab-09.md), [Lab 10](../02-labs/lab-10.md), [Request lifecycle](../01-concepts/request-lifecycle.md), and [Reactivity options](../01-concepts/reactivity-options.md).
