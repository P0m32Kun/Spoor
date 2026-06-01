# Agent guidance — Spoor

Instructions for AI coding agents working in this repository.

## What Spoor is

Spoor statically analyzes **JavaScript/TypeScript files** (typically from [Katana](https://github.com/projectdiscovery/katana) or similar) and emits structured findings for downstream tools.

**Pipeline role:**

```
Katana → JS URL + 页面 URL 列表
Spoor  → JS：path + endpoint + secret
         页面：secret only（补 Katana 未覆盖的泄漏面）
You    → ffuf, Burp, notes, etc.
```

Spoor does **not** crawl, fetch pages, or walk directories.

## In scope

| Area | Detail |
|------|--------|
| **Inputs** | One file path or stdin; extensions `.js`, `.mjs`, `.cjs`, `.ts`, `.tsx` |
| **Outputs** | `ScanResult` JSON / JSONL ([INTEGRATION.md](./docs/INTEGRATION.md)): `path`, `endpoint`, `secret` only |
| **CLI** | `scan`, `paths`, `apis`, `keys`; `-o`, `--jsonl` |
| **Core** | `Analyzer::collect_findings()` + matchers under `crates/spoor-core/src/matcher/` |
| **Acceptable changes** | New/improved matchers for JS patterns; dedup/URL heuristics; tests/fixtures; docs |

### The three extraction types

1. **path** — URL-like strings, router configs (`router.path`), source maps
2. **endpoint** — fetch, XHR, axios, ky, got, superagent, jQuery, location, WebSocket, GraphQL, etc.
3. **secret** — AKIA, GCP/Firebase keys, tokens, sensitive object keys

Every feature must fit one of these three. If it does not, **do not implement it**.

## Out of scope (do not add without explicit user request)

| Feature | Why |
|---------|-----|
| Recursive directory scan | Katana lists files; use `xargs`/`for` + `spoor scan` |
| HTML / page URLs | **Secret-only scan** when URL is not a JS asset (user-requested); no endpoint/path on pages |
| Crawling, spidering, katana-like behavior | Wrong tool |
| YAML/rule files, plugin systems | Not a rule platform |
| SARIF, OpenAPI export, WASM, NAPI | Out of toolchain role |
| `--no-literals` / severity filters | Optional backlog only; not a new product phase |
| “Phase 3/4” platform features | **Cancelled** — see [ROADMAP.md](./docs/ROADMAP.md) |

## Batch usage (caller’s job)

```bash
for f in ./katana-js/*.js; do
  spoor scan "$f" --jsonl >> findings.jsonl
done
```

Do not implement batching inside Spoor unless the user explicitly asks.

## Code conventions

- Match existing matcher style: `Visit` + `collect()` + fixture in `tests/fixtures/`
- Use `endpoint_from_url()` / `resolved_maybe_url` for URL endpoints where applicable
- Run `cargo test --workspace` and `cargo clippy --workspace -- -D warnings` before claiming done
- Minimize diff scope; do not refactor unrelated code
- Do not commit unless the user asks

## 文档 map

| File | Use |
|------|-----|
| [docs/INTEGRATION.md](./docs/INTEGRATION.md) | CLI + JSON schema |
| [docs/ROADMAP.md](./docs/ROADMAP.md) | Scope + progress |
| [docs/history/phase-2-retro.md](./docs/history/phase-2-retro.md) | Phase 2 sign-off |

## Maintenance backlog (allowed, low priority)

- Unify `resolved_maybe_url` across matchers
- Optional `process.env` / build-time env string secrets
- CI: test + fmt + clippy
- Katana-sourced regression fixtures
- **jsluice parity:** `cargo test -p spoor-core jsluice_parity` (needs `jsluice` in PATH; skips if absent)

## Red flags — stop and confirm with user

- Adding dependencies for HTML, HTTP client, filesystem walk, or job queues
- New top-level CLI subcommands unrelated to scan/paths/apis/keys
- Expanding cancelled platform features (see ROADMAP.md)
- README/marketing that positions Spoor as a full scanner
