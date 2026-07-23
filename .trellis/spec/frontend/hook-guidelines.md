# Entry Point And Side-Effect Guidelines

React-style hooks and client-side data fetching do not exist in this project.
There are two explicit execution boundaries: the Clash host entry point and the
Node local-renderer CLI.

## Clash Host Entry Point

Clash Verge Rev calls `main(config, profileName)` from
`clash-verge-ai-residential.js`. Keep it synchronous and deterministic apart
from guarded logging. It validates reserved names and the upstream graph before
injecting the proxy, rules, DNS, TUN, and sniffer configuration.

```js
function warn(message) {
  if (typeof console !== "undefined" && typeof console.warn === "function") {
    console.warn(message);
  }
}
```

The extension performs no network requests, timers, subscriptions, caching, or
background work. Clash/Mihomo consumes the returned configuration and owns the
actual networking lifecycle.

## Node CLI Entry Point

`scripts/sync-local-config.js` separates reusable functions from process side
effects. Tests can require its exports without running the CLI:

```js
if (require.main === module) {
  try {
    main();
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
```

Keep filesystem reads and writes inside the renderer boundary. Validate the
complete local configuration before writing, reject output paths that overwrite
the template or TOML input, and retain `writeFileAtomically`'s temporary-file
cleanup. The command reports failures in Chinese and sets a nonzero exit code.

## Test Loading

`tests/regression.test.js` loads the public extension through CommonJS and allows
`CLASH_SCRIPT_PATH` to target a generated local script. It temporarily suppresses
`console.info` and `console.warn` in `quietMain`, restoring both in `finally`.
Use the same cleanup discipline for any temporary process state.

## Anti-Patterns

- Do not introduce React hooks, event buses, observers, or async fetch layers.
- Do not execute the local renderer on `require`; preserve the
  `require.main === module` guard.
- Do not make the Clash entry point depend on `process`, filesystem APIs, module
  loading, or persistent background state.
- Do not swallow configuration errors or leave partial output after a failed
  local render.
