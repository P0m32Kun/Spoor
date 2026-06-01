# Spoor Phase 1 回顾与完成度报告

> **For agentic workers:** Phase 1 核心 matcher 已签 off。Phase 2 请新建实施计划后再 `executing-plans`。

**Goal:** 对照 [ROADMAP.md](../ROADMAP.md) Phase 1 与 [Phase 1 实施计划](./phase-1-implementation.md)，记录 endpoint matcher 交付度、流程执行、验证证据与遗留项。

**Architecture:** `collect_findings()` 流水线 — fetch → location → xhr → literal → `dedup_findings()`；语义匹配产出 `FindingKind::Endpoint`，字面量产出 `Path`。

**文档日期:** 2026-05-31  
**执行方式:** Superpowers Subagent-Driven（7 Task + 审查修复）

---

## 1. 流程执行

| 步骤 | 状态 |
|------|------|
| `writing-plans` Phase 1 计划 | ✅ `phase-1-implementation.md` |
| `subagent-driven-development` 7 Task | ✅ 逐 Task implementer + 最终 code-reviewer |
| `verification-before-completion` | ✅ 下文 §4 fresh run |
| 审查修复 commit | ✅ `3feda14` 动态 fetch EXPR 误报 |

---

## 2. plan1 Phase 1 完成度

**交付目标（plan1）：** endpoint/URL 提取对标 jsluice 核心能力 → **Spoor 映射为 `spoor apis` + plan2 `endpoint` finding**

### 2.1 Matcher（plan1 优先级表）

| Matcher | plan1 | Phase 1 状态 | 证据 |
|---------|-------|-------------|------|
| fetch | ✅ 目标 | ✅ 完成 | `matcher/fetch.rs`，`resolved_maybe_url` 拒绝 EXPR |
| location / href / assign | ✅ | ✅ 完成 | `matcher/location.rs` |
| location.replace | ✅ | ✅ 完成 | 同上 |
| window.location = | ✅ | ✅ 完成 | pattern `location` |
| XHR .open | ✅ | ⚠️ 部分 | `matcher/xhr.rs` — 匹配任意 `.open`，未校验 XMLHttpRequest 类型 |
| string literal 兜底 | ✅ | ✅ 完成 | `matcher/literal.rs` |
| jQuery | plan1 | ❌ Phase 2+ | — |
| window.open | plan1 | ❌ Phase 2+ | — |
| 泛化 call | plan1 | ❌ Phase 2+ | — |

### 2.2 其它 Phase 1 项

| 项 | 状态 | 说明 |
|----|------|------|
| 过滤 data:/tel:/javascript:/EXPR | ⚠️ 部分 | `maybe_url` + fetch 侧 `resolved_maybe_url`；非全 matcher 统一 |
| query_params 解析 | ❌ | Phase 1.5 / Phase 2 |
| HTML `<script>` 提取 | ❌ | Phase 1 plan 明确不做 |
| CLI apis / scan / stdin | ✅ | `spoor-cli` 已接 `collect_findings()` |
| 去重 | ✅ | `dedup.rs`，Endpoint > Path |
| jsluice 对比 fixture 10–20 个 | ⚠️ 部分 | `phase1/combined.js` + 单 matcher fixture，未做 jsluice 集合对比 |

### 2.3 Phase 1 计划 Task 对照

| Task | 状态 | Commit |
|------|------|--------|
| 1 MatchContext + collect_findings | ✅ | `904190e` |
| 2 fetch | ✅ | `1d455e4` |
| 3 location | ✅ | `0237fc2` |
| 4 XHR | ✅ | `3205f53` |
| 5 dedup 管线 | ✅ | `3eb6885` |
| 6 CLI | ✅ | `83a695f` |
| 7 fixture + README | ✅ | `4f14712` |
| 审查：EXPR fetch 误报 | ✅ | `3feda14` |

---

## 3. 代码库快照（Phase 1 后）

```
crates/spoor-core/src/
├── analyzer.rs          # collect_findings() 入口
├── dedup.rs
├── finding.rs           # Finding::endpoint()
├── matcher/
│   ├── mod.rs           # MatchContext
│   ├── fetch.rs
│   ├── location.rs
│   ├── xhr.rs
│   └── literal.rs
├── string_fold.rs
└── url.rs               # maybe_url, resolved_maybe_url

tests/fixtures/
├── fetch.js, location.js, xhr.js
├── phase1/combined.js   # 7 endpoint 验收
└── sample.js            # path + 动态 fetch 限制用例
```

**测试：** 19 项（spoor-core 18 + 无 CLI 单测）

---

## 4. 验证证据（2026-05-31 fresh run）

```bash
cargo test --workspace          # 19 passed, 0 failed
cargo fmt --check               # OK
cargo clippy --workspace -- -D warnings  # 0 issues
```

**CLI 冒烟：**

```bash
spoor apis tests/fixtures/phase1/combined.js
# → 7 endpoints: fetch×2, location×3, xhr×2

spoor scan tests/fixtures/sample.js
# → 1 endpoint (location.href CDN URL)
# → 2 paths (/api/v1, /users)
# → 无 EXPR/users endpoint（动态 fetch 已拒绝）
```

---

## 5. 已知限制与技术债

| 项 | 严重度 | 说明 |
|----|--------|------|
| 每次 `collect_findings()` 重新 parse | 低 | Oxc Program 生命周期限制；`parse_outcome` 经 `OnceCell` 缓存 |
| XHR 匹配过宽 | 中 | 任意 `obj.open(method, url)` 会命中 |
| 无 `EndpointMatcher` trait | 低 | 计划提及 trait；现为独立 `collect()` 方法 |
| 无 spoor-cli 集成测试 | 低 | 依赖 analyzer 测试 + 手动冒烟 |
| jQuery / window.open / HTML | — | Phase 2+ 范围 |

---

## 6. Phase 1 签 off 条件

- [x] §2.3 全部 Task 完成
- [x] fetch / location / XHR 产出 endpoint
- [x] `spoor apis` 对 combined.js 输出 7 条
- [x] 去重：同 URL endpoint 优先于 path
- [x] 动态 `fetch(base + "/users")` 不产出 EXPR endpoint
- [x] fmt + clippy + 19 tests 全绿
- [x] README Phase 1 章节

**结论：Phase 1 核心目标已签 off**（jQuery/jsluice 全量对比留 Phase 1.5/2）

---

## 7. Phase 2 交接建议

1. **secrets matcher** — plan1 Phase 2 首要项
2. **jQuery / axios** — 红队高频客户端
3. **收紧 XHR** — 校验 `new XMLHttpRequest()` 绑定
4. **CLI golden test** — `insta` 快照 JSON 输出
5. **jsluice 对比测试** — 移植可移植用例

建议新建：`docs/superpowers/plans/YYYY-MM-DD-spoor-phase-2.md`

---

## 8. Commit 历史（Phase 1 段）

```
3feda14 fix: reject dynamic fetch URLs with EXPR placeholder
4f14712 docs: Phase 1 matcher coverage and README
83a695f feat: wire CLI to collect_findings pipeline
3eb6885 feat: dedup and unified collect_findings pipeline
3205f53 feat: XMLHttpRequest.open endpoint matcher
0237fc2 feat: location endpoint matcher
1d455e4 feat: fetch endpoint matcher
904190e refactor: cache parsed Program and add MatchContext
```

---

*Generated following Superpowers `writing-plans` + `verification-before-completion` on 2026-05-31.*
