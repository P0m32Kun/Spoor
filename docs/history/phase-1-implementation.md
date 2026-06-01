# Spoor Phase 1 — API Endpoint Matchers Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 `spoor apis` 与 `spoor scan` 能输出语义级 `endpoint` finding（fetch / location / XHR），对标 jsluice URL 提取的核心能力，同时保留 Phase 0 的 `path` 字面量兜底。

**Architecture:** 在 `spoor-core` 引入 `Matcher` trait 与 `MatchContext`（源码、span→行列、snippet）。`Analyzer` 缓存单次 Oxc parse 的 `Program`，按优先级注册 matcher visitor；语义 URL 产出 `FindingKind::Endpoint`，纯字面量仍走 `string_literal` → `Path`。去重时同 URL 保留 origin 信息更丰富的一条（method > pattern 优先级）。

**Tech Stack:** Rust 1.93+、Oxc 0.133、现有 `collapsed_string` / `maybe_url`、serde JSON（plan2 模型）

**Spec 来源:** [ROADMAP.md](../ROADMAP.md) § Phase 1、[INTEGRATION.md](../INTEGRATION.md) endpoint JSON 模型

**Phase 0 前置:** 已完成（8/8 tests，`history/phase-0-retro.md`)

**Implementation status (2026-05-31):** ✅ 全部 7 Task 已完成 + 审查修复 `3feda14`。回顾见 [phase-1-retro](./2026-05-31-spoor-phase-1-retro.md)。

---

## 文件结构（Phase 1 新增/修改）

| 文件 | 职责 |
|------|------|
| `crates/spoor-core/src/matcher/mod.rs` | `MatchContext`、`EndpointMatcher` trait、matcher 注册表 |
| `crates/spoor-core/src/matcher/fetch.rs` | `fetch(url, init?)` |
| `crates/spoor-core/src/matcher/location.rs` | `location.href` / `.assign` / `.replace` |
| `crates/spoor-core/src/matcher/xhr.rs` | `XMLHttpRequest.open(method, url)` |
| `crates/spoor-core/src/matcher/literal.rs` | 从 Phase 0 `LiteralCollector` 迁移 |
| `crates/spoor-core/src/dedup.rs` | 同 value 去重，保留高 confidence / 有 method 者 |
| `crates/spoor-core/src/analyzer.rs` | 缓存 Program；`collect_findings()` 统一入口 |
| `crates/spoor-core/src/finding.rs` | 新增 `Finding::endpoint(...)` 构造器 |
| `crates/spoor-cli/src/main.rs` | `apis`/`scan` 调用 `collect_findings()` |
| `tests/fixtures/fetch.js` | fetch 单测 fixture |
| `tests/fixtures/location.js` | location fixture |
| `tests/fixtures/xhr.js` | XHR fixture |

---

### Task 1: Analyzer 单次 parse + MatchContext 基础设施

**Files:**
- Create: `crates/spoor-core/src/matcher/mod.rs`
- Modify: `crates/spoor-core/src/lib.rs`
- Modify: `crates/spoor-core/src/analyzer.rs`

- [x] **Step 1–6:** 完成 — `904190e`（Oxc 生命周期：每次 `collect_findings()` parse 一次，`parse_outcome` 经 `OnceCell` 缓存）

---

### Task 2: fetch matcher

**Files:**
- Create: `crates/spoor-core/src/matcher/fetch.rs`
- Create: `tests/fixtures/fetch.js`
- Modify: `crates/spoor-core/src/matcher/mod.rs`
- Modify: `crates/spoor-core/src/finding.rs`
- Modify: `crates/spoor-core/src/analyzer.rs`

**Fixture `tests/fixtures/fetch.js`:**

```javascript
fetch("/api/v1/users");
fetch("https://api.example.com/data", { method: "POST" });
const u = "/ignored";
```

- [x] **Step 1–5:** 完成 — `1d455e4`

```rust
// matcher/fetch.rs #[cfg(test)] 或 analyzer tests
#[test]
fn fetch_matcher_finds_endpoints() {
    let src = include_str!("../../../../tests/fixtures/fetch.js");
    let a = Analyzer::new(src, Some("fetch.js"));
    let findings = a.collect_findings()
        .into_iter()
        .filter(|f| f.kind == FindingKind::Endpoint)
        .collect::<Vec<_>>();
    assert_eq!(findings.len(), 2);
    assert!(findings.iter().any(|f| f.value == "/api/v1/users" && f.method.as_deref() == Some("GET")));
    assert!(findings.iter().any(|f| f.value.contains("api.example.com") && f.method.as_deref() == Some("POST")));
}
```

- [ ] **Step 2:** 运行确认 FAIL

- [ ] **Step 3: 实现 fetch matcher**

遍历 `CallExpression`：
- callee 为 `fetch`（Identifier 或 MemberExpression 末段为 fetch）
- 首参经 `collapsed_string` + `maybe_url` 过滤
- 次参若为 ObjectExpression，读 `method` 属性字符串；默认 `GET`
- 产出 `Finding::endpoint(value, method, origin { pattern: "fetch" })`

`finding.rs` 添加：

```rust
impl Finding {
    pub fn endpoint(value: impl Into<String>, method: impl Into<String>, origin: Origin) -> Self {
        Self {
            kind: FindingKind::Endpoint,
            value: value.into(),
            confidence: Confidence::High,
            method: Some(method.into()),
            origin,
            ../* 其余 None/empty */ 
        }
    }
}
```

- [ ] **Step 4:** `cargo test -p spoor-core fetch_matcher` — PASS

- [ ] **Step 5: Commit**

```bash
git commit -m "feat: fetch endpoint matcher"
```

---

### Task 3: location matcher

**Files:**
- Create: `crates/spoor-core/src/matcher/location.rs`
- Create: `tests/fixtures/location.js`
- Modify: `crates/spoor-core/src/matcher/mod.rs`

**Fixture:**

```javascript
location.href = "https://cdn.example.com/app.js";
location.replace("/login");
window.location = "/dashboard";
```

- [ ] **Step 1: failing test** — 断言 3 条 endpoint，pattern 含 `location.href` / `location.replace` / `location.assign`

- [ ] **Step 2:** 实现 AssignmentExpression + CallExpression：
  - `location.href = url`、`location = url`（Member/Identifier location）
  - `location.replace(url)`、`location.assign(url)`
  - 右侧必须 `collapsed_string` 后以 `/` 或 `http` 开头（plan1：「以字符串开头」）

- [ ] **Step 3:** `cargo test -p spoor-core location` — PASS

- [ ] **Step 4: Commit** — `feat: location endpoint matcher`

---

### Task 4: XHR matcher

**Files:**
- Create: `crates/spoor-core/src/matcher/xhr.rs`
- Create: `tests/fixtures/xhr.js`

**Fixture:**

```javascript
const xhr = new XMLHttpRequest();
xhr.open("GET", "/api/v1/status");
xhr.open("POST", "https://example.com/submit");
```

- [ ] **Step 1: failing test** — 2 endpoints，method GET/POST

- [ ] **Step 2:** 匹配 `MemberExpression.open` 且 object 链含 `XMLHttpRequest` 或变量；arg0=method 字符串，arg1=url

- [ ] **Step 3:** PASS + commit `feat: XMLHttpRequest.open endpoint matcher`

---

### Task 5: 去重与 collect_findings 集成

**Files:**
- Create: `crates/spoor-core/src/dedup.rs`
- Modify: `crates/spoor-core/src/analyzer.rs`

- [ ] **Step 1: failing test**

同 URL `/api/v1` 同时被 fetch（endpoint, method GET）与 string literal（path）命中 → 去重后保留 endpoint。

```rust
#[test]
fn dedup_prefers_endpoint_over_literal() {
    let src = r#"fetch("/api/v1"); const x = "/api/v1";"#;
    let findings = Analyzer::new(src, Some("t.js")).collect_findings();
    let api_v1: Vec<_> = findings.iter().filter(|f| f.value == "/api/v1").collect();
    assert_eq!(api_v1.len(), 1);
    assert_eq!(api_v1[0].kind, FindingKind::Endpoint);
}
```

- [ ] **Step 2:** 实现 `dedup_findings(findings) -> Vec<Finding>`：
  - key = normalized `value`
  - 优先级：Endpoint > Path；同 kind 时 confidence High > Medium > Low；有 method 优先

- [ ] **Step 3:** `collect_findings()` 按序运行 matcher：fetch → location → xhr → literal → dedup

- [ ] **Step 4:** 更新 `sample.js` 集成测试 — `fetch(base + "/users")` 仍只 literal `/users`（Phase 1 不折叠变量拼接为 endpoint，文档注明限制）

- [ ] **Step 5:** `cargo test -p spoor-core` — 全绿

- [ ] **Step 6: Commit** — `feat: dedup and unified collect_findings pipeline`

---

### Task 6: CLI 接入 + 移除 placeholder 提示

**Files:**
- Modify: `crates/spoor-cli/src/main.rs`

- [ ] **Step 1:** `run_scan` 改调 `analyzer.collect_findings()` 替代 `collect_literal_paths()`

- [ ] **Step 2:** 删除 `Endpoint` 的 "not implemented yet" eprintln

- [ ] **Step 3:** 手动冒烟

```bash
cargo run -p spoor-cli -- apis tests/fixtures/fetch.js
cargo run -p spoor-cli -- scan tests/fixtures/sample.js
```

Expected: `apis` 仅 endpoint；`scan` 含 path + endpoint

- [ ] **Step 4: Commit** — `feat: wire CLI to collect_findings pipeline`

---

### Task 7: 验收 fixture 集 + README

**Files:**
- Create: `tests/fixtures/phase1/` 目录，汇总 5–10 个 snippet
- Modify: `README.md`

- [ ] **Step 1:** 添加 `tests/fixtures/phase1/combined.js` 覆盖 fetch+location+xhr 同文件

- [ ] **Step 2:** analyzer 集成测试断言 endpoint 数量与 value 集合

- [ ] **Step 3:** README Phase 1 能力说明：`spoor apis` 已支持 fetch/location/XHR

- [ ] **Step 4:** 全量验证

```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

- [ ] **Step 5: Commit** — `docs: Phase 1 matcher coverage and README`

---

## Phase 1 验收标准

| 标准 | 验证命令 |
|------|----------|
| fetch/location/XHR 产出 endpoint | `cargo test -p spoor-core` |
| `spoor apis` 有真实输出 | `cargo run -p spoor-cli -- apis tests/fixtures/phase1/combined.js` |
| 去重生效 | dedup unit test |
| fmt + clippy 干净 | `cargo fmt --check && cargo clippy --workspace -- -D warnings` |
| Phase 0 测试仍 pass | `cargo test --workspace` |

## Phase 1 明确不做（留给 Phase 2+）

- jQuery / window.open / 泛化 call matcher
- HTML `<script>` 提取
- 目录递归 / rayon 并行
- query_params 从 URL 解析（可 Phase 1.5 小任务）
- secrets / keys

## 风险

| 风险 | 对策 |
|------|------|
| Oxc AST 版本差异 | fixture 测试锁行为 |
| fetch 动态 URL 误报 | `maybe_url` + 仅首参折叠 |
| 去重丢失 context | 保留 origin 更丰富条目 |

---

*Plan authored via Superpowers `writing-plans` on 2026-05-31.*
