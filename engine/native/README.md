# native

`raea-native`: raea's native calls on JavaScriptCore. TypeScript reaches Rust through one function,
`__raea_call(name, json)`; the names it can call live in the registry ([`core/calls/`](../../core/calls)), each with the
TypeScript types of its arguments and answer, registered by whichever crate implements it. `raea types` writes the
types `raea` imports, so a call that doesn't exist or takes the wrong arguments fails to type-check.

It holds the JavaScriptCore binding (`src/jsc.rs`), the web shims bundles expect (`src/web/`: TextEncoder, URL,
AbortController, crypto, Blob, ...), and seeds the registry with the shell's native tools
([`io/shell/`](../../io/shell): the filesystem, HTTP under a policy, SQLite, tar, HTML to Markdown).

[`runtime/js/`](../raea-runtime-js) and [`sdk/react/`](../../sdk/react) depend on it.
