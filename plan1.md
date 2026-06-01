# Spoor 总体开发计划

> **进度快照：** [docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md)  
> **JSON 输出模型：** [plan2.md](./plan2.md)  
> **仓库：** https://github.com/P0m32Kun/Spoor

**最后更新：** 2026-05-31 · **版本：** 0.1.0 · **测试：** 35 passed

---

## 一、定位与范围（锁定）

### 1.1 在工具链中的位置

Spoor 是**众多工具中的一环**，专门消费 **Katana（等）已提取的 JS 文件**：

```
Katana（爬取 + 抽 JS）→ Spoor（单文件静态分析）→ 下游（ffuf / Burp / 手工）
```

### 1.2 只做三件事

| 产出 | `kind` | 说明 |
|------|--------|------|
| 路径提取 | `path` | 字面量路径、路由 path、source map 等 |
| API 提取 | `endpoint` | fetch / XHR / axios / WebSocket 等可发起请求 |
| 敏感信息提取 | `secret` | AKIA、GCP、Firebase、token、对象键等 |

**输入：** 单个 `.js` / `.mjs` / `.ts` 文件，或 stdin（`-`）。  
**输出：** JSON / JSONL（[plan2](./plan2.md)），供管道消费。

### 1.3 明确不做（不扩功能范围）

| 不做 | 原因 |
|------|------|
| 目录递归 / 并行扫 dist | Katana + `xargs`/`for` 编排即可 |
| HTML / 抽 `<script>` | Katana 已产出 JS |
| 爬站、链接发现 | Katana 职责 |
| YAML 规则引擎 | 非 JS 语义分析核心 |
| SARIF / WASM / NAPI | 非当前工具链需求 |

历史草案见 [plan4.md](./plan4.md)、[plan5.md](./plan5.md)（**已归档，不实施**）。

### 1.4 与 jsluice 的关系

| 维度 | jsluice | Spoor |
|------|---------|-------|
| 解析 | go-tree-sitter | **Oxc** |
| 场景 | 通用 JS 扫描 | **Katana 下游收参** |
| 输出 | JSON | JSON / JSONL，三类 `kind` |
| 扩展 | Go 回调 | Rust matcher 模块（仅 JS 提取） |

---

## 二、整体进度

| 阶段 | 进度 | 状态 | 文档 |
|------|------|------|------|
| Phase 0 | 100% | ✅ 已签 off | § Phase 0 · [retro](./docs/superpowers/plans/2026-05-31-spoor-phase-0-retro.md) |
| Phase 1 | 100% | ✅ 已签 off | § Phase 1 · [retro](./docs/superpowers/plans/2026-05-31-spoor-phase-1-retro.md) |
| Phase 2 | 100% | ✅ 已签 off | [plan3.md](./plan3.md) · [retro](./docs/superpowers/plans/2026-05-31-spoor-phase-2-retro.md) |
| 维护 backlog | — | 📋 按需 | § 六 |

~~Phase 3 / Phase 4~~ → **已取消**（见 §1.3）

---

## 三、技术架构

```
┌─────────────────────────────────────────────────────────┐
│  spoor-cli     scan / paths / apis / keys  （单文件 in） │
├─────────────────────────────────────────────────────────┤
│  spoor-core    Analyzer · MatchContext · dedup          │
├──────────┬──────────┬──────────┬────────────────────────┤
│ matcher/ │ string   │ url      │ finding (plan2 JSON)   │
│ (见 §五) │ _fold    │ maybe_url│ path / endpoint/secret │
├──────────┴──────────┴──────────┴────────────────────────┤
│  Oxc Parser + ast_visit                                 │
└─────────────────────────────────────────────────────────┘
         ▲
    Katana 输出的 .js 文件
```

**matcher 流水线：** fetch → location → xhr → axios → ky → got → superagent → jquery → window_open → websocket → graphql → router → secret → source_map → literal → dedup

---

## 四、CLI 功能矩阵

| 命令 | 作用 | 状态 |
|------|------|------|
| `spoor scan <file>` | path + endpoint + secret | ✅ |
| `spoor paths <file>` | 仅 path | ✅ |
| `spoor apis <file>` | 仅 endpoint | ✅ |
| `spoor keys <file>` | 仅 secret | ✅ |
| `-` stdin | 管道 | ✅ |
| `-o` / `--jsonl` | 文件 / 行式输出 | ✅ |

**Katana 批量示例：**

```bash
for f in ./katana-out/*.js; do spoor scan "$f" --jsonl >> all-findings.jsonl; done
```

---

## 五、分阶段里程碑

### Phase 0 — 基础 ✅

- [x] Workspace、Oxc 解析、`collapsed_string`、`maybe_url`、fixture

[Phase 0 retro](./docs/superpowers/plans/2026-05-31-spoor-phase-0-retro.md)

---

### Phase 1 — 语义 Endpoint ✅

- [x] fetch / location / XHR / literal / dedup / CLI
- [x] jQuery / window.open / axios（收尾）
- [x] XHR 收紧、query_params、jsluice 子集 fixture

[Phase 1 retro](./docs/superpowers/plans/2026-05-31-spoor-phase-1-retro.md)

---

### Phase 2 — Secret + 红队 URL ✅

- [x] Secret：AKIA、sk-、GCP、Firebase、service account、对象键
- [x] HTTP：ky、got、superagent（+ Phase 1 axios/jQuery）
- [x] 协议：WebSocket、GraphQL、router.path、sourceMappingURL
- [x] `spoor keys` / `spoor scan` 三类齐全

[plan3.md](./plan3.md) · [Phase 2 retro](./docs/superpowers/plans/2026-05-31-spoor-phase-2-retro.md)

---

## 六、维护 backlog（在范围内，按需）

不新开「Phase 3 平台」；仅 JS 提取质量与管道体验：

| 项 | 优先级 | 说明 |
|----|--------|------|
| 统一 `resolved_maybe_url` | 中 | 减少 EXPR 误报 |
| `process.env` / 构建 env 字符串 | 低 | 仍属 secret 提取 |
| Katana 真实 bundle fixture | 低 | 回归误报/漏报 |
| CI（test + fmt + clippy） | 低 | 质量门禁 |
| CLI JSON 快照（insta） | 低 | 可选 |

---

## 七、已知限制

1. **单文件** — 设计如此；批量由 Katana + shell 负责。
2. **动态 URL** — `fetch(base + "/path")` 不产出可靠 endpoint。
3. **非 JS 输入** — 不支持 HTML；请先用 Katana 抽 JS。
4. **env 变量 secret** — 可选 backlog，未实现。

---

## 八、文档索引

| 文档 | 用途 |
|------|------|
| [plan1.md](./plan1.md) | 本文 — 范围锁定 + 进度 |
| [plan2.md](./plan2.md) | JSON 模型 |
| [plan3.md](./plan3.md) | Phase 2 实施清单（已完成） |
| [plan4.md](./plan4.md) | ~~Phase 3~~ 归档，不实施 |
| [plan5.md](./plan5.md) | ~~Phase 4~~ 归档，不实施 |
| [docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md) | 进度快照 |
