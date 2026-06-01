# Spoor Phase 2 回顾与完成度报告

> **For agentic workers:** Phase 2 已签 off。Spoor 定位为 **Katana 下游 JS 单文件分析器**，只做 path / endpoint / secret 三类提取，勿扩展爬取、HTML、目录扫描等能力。后续见 [ROADMAP.md](../ROADMAP.md) § 范围锁定。

**Goal:** 对照 [ROADMAP.md](../ROADMAP.md) 与 [ROADMAP.md](../ROADMAP.md) Phase 2，记录 secret matcher、现代 HTTP 客户端、协议类 URL、路由 path 的交付度与验证证据。

**Architecture:** `collect_findings()` 流水线 — 语义 endpoint matcher → router / secret → literal path → `dedup_findings()`；`spoor scan` 一次输出三类 finding。

**文档日期:** 2026-05-31  
**执行方式:** 迭代开发（Phase 1 收尾 + plan3 Task A–D/C）

---

## 1. 产品范围（签 off 时锁定）

Spoor 是工具链中的**单环**，不是独立扫描平台：

```
Katana（爬取 + 抽取 JS）→ Spoor（静态分析单个 .js）→ 下游（ffuf / 手工 / 其他）
```

| 在范围内 | 不在范围内 |
|----------|------------|
| 单文件 JS/TS 输入（含 stdin `-`） | 目录递归 / 并行扫 dist（交给 shell + Katana） |
| **path** 提取 | HTML / `<script>` 抽取（Katana 已做） |
| **endpoint** 提取 | 爬站、链接发现 |
| **secret** 提取 | YAML 规则引擎、SARIF、WASM/NAPI |
| JSON / JSONL 管道输出 | 泛化「安全平台」能力 |

批量示例（由调用方负责编排）：

```bash
for f in ./katana-js/*.js; do spoor scan "$f" --jsonl -o "${f}.jsonl"; done
```

---

## 2. plan3 完成度

### 2.1 交付定义

| 项 | 状态 | 证据 |
|----|------|------|
| `spoor keys` 检出 AKIA / GCP 等 | ✅ | `secrets.js` fixture + CLI |
| `spoor scan` 三类 finding | ✅ | `scan` 命令过滤 kind |
| axios + jQuery | ✅ | Phase 1 收尾 + plan3 |
| XHR 收紧 | ✅ | 仅 `new XMLHttpRequest()` 绑定 |
| 35 tests 全绿 | ✅ | §4 fresh run |

### 2.2 Task 对照

| Task | 状态 | 说明 |
|------|------|------|
| A Secret 基础 | ✅ | `matcher/secret.rs`，AKIA / sk- / 对象键 |
| B GCP / Firebase | ✅ | AIza、service_account、firebase_config |
| C HTTP 客户端 | ✅ | axios、jQuery、ky、got、superagent、window.open |
| D 协议 / 路由 | ✅ | WebSocket、GraphQL、router.path、sourceMappingURL |
| E URL 质量 | ⚠️ 部分 | query_params ✅；全 matcher `resolved_maybe_url` 未统一 |
| F 文档 | ✅ | README、plan 更新；本文 retro |

**可选未做（不阻塞签 off）：** `REACT_APP_*` / `process.env.X` 字符串 secret。

---

## 3. Matcher 快照（Phase 2 后）

**Endpoint：** fetch → location → xhr → axios → ky → got → superagent → jquery → window_open → websocket → graphql

**Path：** router.path → sourceMappingURL → literal（兜底）

**Secret：** AKIA、AIza、Firebase、service_account、sk-、ghp_、对象键启发

```
crates/spoor-core/src/matcher/
├── fetch.rs, location.rs, xhr.rs
├── axios.rs, http_clients.rs (ky/got/superagent)
├── jquery.rs, window_open.rs
├── websocket.rs, graphql.rs
├── router.rs, source_map.rs
├── secret.rs, literal.rs
└── util.rs
```

**测试：** 35 项（spoor-core 单元 + fixture 集成）

---

## 4. 验证证据（2026-05-31 fresh run）

```bash
cargo test --workspace          # 35 passed, 0 failed
cargo clippy --workspace -- -D warnings  # 0 issues
```

**CLI 冒烟：**

```bash
spoor keys tests/fixtures/secrets.js
# → aws_access_key, gcp_api_key, firebase_api_key, gcp_service_account_key, object_literal_key

spoor scan tests/fixtures/sample.js
# → path + endpoint + secret（AKIA）

spoor apis tests/fixtures/ky.js
# → ky.get / ky.post / ky endpoints

spoor paths tests/fixtures/router.js
# → router.path: /home, /users/:id, /admin, settings, /dashboard
```

---

## 5. 已知限制与技术债（接受范围内）

| 项 | 严重度 | 说明 |
|----|--------|------|
| 单文件 only | — | **设计如此**；批量由 Katana + shell 编排 |
| 动态 URL `fetch(a + "/b")` | 中 | 不产出 endpoint；literal `/b` 可能仍为 path |
| location 未统一 `resolved_maybe_url` | 低 | Task E 遗留 |
| 无 CLI insta 快照 | 低 | 依赖 core 测试 + 手动冒烟 |
| env 变量 secret | — | 可选，未做 |
| jsluice 自动 diff | 低 | 有 `jsluice_subset.js`，无 CI 对比 |

---

## 6. Phase 2 签 off 条件

- [x] `spoor keys` 有真实 secret 输出
- [x] `spoor scan` 同时产出 path + endpoint + secret
- [x] 现代 HTTP 客户端（含 ky/got/superagent）覆盖
- [x] GCP / Firebase secret 基础覆盖
- [x] WebSocket / GraphQL / router path
- [x] 35 tests + clippy 全绿
- [x] 产品范围写入 plan1（Katana 下游、不扩平台）

**结论：Phase 2 已签 off。** 可选 env 与 matcher 精度项归入「维护 backlog」，不新开 Phase 3 平台功能。

---

## 7. 维护 backlog（在范围内，按需）

1. 全 matcher 统一 `resolved_maybe_url`
2. JS 内 `process.env.*` / 构建时 env 字符串（secret 类）
3. Katana 真实 bundle fixture 回归
4. GitHub Actions：test + fmt + clippy

**明确不做：** 见 [ROADMAP.md](../ROADMAP.md) / [ROADMAP.md](../ROADMAP.md)（已标记 out of scope）。

---

## 8. Commit 历史（Phase 2 段）

```
bfd3fe8 feat: add ky, got, and superagent HTTP client matchers
e853528 feat: add GCP/Firebase secrets and router path matcher
7893a15 feat: add WebSocket, GraphQL, and source map matchers
9e0d592 feat: complete Phase 1 matchers and start Phase 2 secrets
```

---

*Generated 2026-05-31. Scope locked: Katana → Spoor (JS path/endpoint/secret only).*
