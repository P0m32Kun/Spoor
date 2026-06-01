# Spoor 整体开发进度与功能规划

**整体进度与功能规划：** [docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md)（进度快照）

**分阶段计划（plan 文件）：**

| 文件 | 内容 |
|------|------|
| [plan1.md](../plan1.md) | 总体计划、进度、Phase 0–1 清单 |
| [plan2.md](../plan2.md) | 命名 + JSON 输出模型 |
| [plan3.md](../plan3.md) | Phase 2：密钥 + 红队 URL 增强 |
| [plan4.md](../plan4.md) | Phase 3：并行、规则、工程化 |
| [plan5.md](../plan5.md) | Phase 4：高级能力（按需） |

阶段回顾：

- [Phase 0 retro](./superpowers/plans/2026-05-31-spoor-phase-0-retro.md)
- [Phase 1 retro](./superpowers/plans/2026-05-31-spoor-phase-1-retro.md)

**仓库：** https://github.com/P0m32Kun/Spoor  
**最后更新：** 2026-05-31  
**当前版本：** 0.1.0（未打 tag）  
**测试：** 19 passed · **Commits：** 13 on `main`

---

## 1. 项目是什么

Spoor 是在 JavaScript / TypeScript 资产里做**静态信息收集**的 CLI 工具，面向红队场景：

| 产出类型 | `kind` | 用途示例 |
|----------|--------|----------|
| 路径足迹 | `path` | `/api/v1/users`、静态资源路径 → ffuf、目录爆破 |
| API 端点 | `endpoint` | `fetch` / `location` / XHR 可发起的请求 → 接口梳理 |
| 密钥泄漏 | `secret` | AKIA、sk-、API Key → 凭证排查 |

**对标：** [jsluice](https://github.com/BishopFox/jsluice)（Go，维护放缓）  
**解析引擎：** [Oxc](https://oxc.rs/)（Rust，高性能 AST）  
**设计原则：** 先对齐 jsluice 核心能力，再在红队场景扩展（axios、GraphQL、规则文件等）

---

## 2. 整体进度一览

```
Phase 0  基础解析 + 字面量路径     ████████████████████ 100%  ✅ 已签 off
Phase 1  语义 endpoint matcher      ███████████████░░░░░  75%  ✅ 核心已签 off
Phase 2  密钥 + 红队 URL 增强      ░░░░░░░░░░░░░░░░░░░░   0%  📋 未开始
Phase 3  并行 / 规则 / 工程化      ░░░░░░░░░░░░░░░░░░░░   0%  📋 未开始
Phase 4  高级能力（可选）          ░░░░░░░░░░░░░░░░░░░░   0%  📋 按需
```

| 阶段 | 目标（一句话） | 状态 | 文档 |
|------|----------------|------|------|
| **Phase 0** | 能解析 JS，从字符串字面量提取 path | ✅ 完成 | [retro](./superpowers/plans/2026-05-31-spoor-phase-0-retro.md) |
| **Phase 1** | `spoor apis` 输出 fetch/location/XHR endpoint | ✅ 核心完成 | [retro](./superpowers/plans/2026-05-31-spoor-phase-1-retro.md) · [plan](./superpowers/plans/2026-05-31-spoor-phase-1.md) |
| **Phase 2** | `spoor keys` + axios/WebSocket/GraphQL 等 | 📋 未开始 | [plan3.md](../plan3.md) |
| **Phase 3** | 目录并行扫描、YAML 规则、性能基准 | 📋 未开始 | [plan4.md](../plan4.md) |
| **Phase 4** | regex 兜底、SARIF、WASM/NAPI | 📋 按需 | [plan5.md](../plan5.md) |

**Phase 1 为何写 75%：** fetch / location / XHR / 去重 / CLI 已交付；jQuery、window.open、HTML 抽取、jsluice 全量对比、query_params 解析仍属 plan1 范围但未做。

---

## 3. 功能矩阵（你现在能用什么）

### 3.1 CLI 命令

| 命令 | 作用 | 当前能力 | 计划 |
|------|------|----------|------|
| `spoor scan <file>` | 全类 finding | path + endpoint | + secret（Phase 2） |
| `spoor paths <file>` | 仅 path | ✅ 字面量路径 | 不变 |
| `spoor apis <file>` | 仅 endpoint | ✅ fetch / location / XHR | + axios 等（Phase 2） |
| `spoor keys <file>` | 仅 secret | ❌ 占位，无输出 | Phase 2 |
| `-` stdin | 管道输入 | ✅ | — |
| `-o` / `--jsonl` | 文件输出 / 行式 JSON | ✅ | — |
| 目录递归 `./dist/` | 批量扫描 | ❌ 仅单文件 | Phase 3 + rayon |
| `--no-literals` 等过滤 | 降噪 | ❌ | Phase 3 |

### 3.2 Endpoint Matcher（`spoor apis`）

| Matcher | 触发模式 | 状态 | 代码 |
|---------|----------|------|------|
| **fetch** | `fetch(url, { method })` | ✅ | `matcher/fetch.rs` |
| **location.href** | `location.href = url` | ✅ | `matcher/location.rs` |
| **location.replace / assign** | `location.replace(url)` | ✅ | 同上 |
| **window.location** | `window.location = url` | ✅ | 同上 |
| **XHR.open** | `xhr.open(method, url)` | ⚠️ 偏宽 | `matcher/xhr.rs` |
| **string literal** | 兜底 → `path` | ✅ | `matcher/literal.rs` |
| jQuery | `$.get` / `$.ajax` | ❌ Phase 2 | — |
| window.open | `window.open(url)` | ❌ Phase 2 | — |
| axios / ky / got | 现代 HTTP 库 | ❌ Phase 2 | — |
| WebSocket | `new WebSocket(url)` | ❌ Phase 2 | — |
| GraphQL | `/graphql`、`gql`… | ❌ Phase 2 | — |
| 泛化 call | 首参像 URL 的任意调用 | ❌ Phase 2+ | — |

### 3.3 Secret 检测（`spoor keys`）— 全部未实现

| 类型 | 状态 |
|------|------|
| AWS / GCP / GitHub / Firebase | ❌ Phase 2 |
| Bearer、sk-、AKIA 等模式 | ❌ Phase 2 |
| 对象键启发（apiKey、token…） | ❌ Phase 2 |
| REACT_APP_* / VITE_* / process.env | ❌ Phase 2（可选） |

### 3.4 基础设施

| 能力 | 状态 | 说明 |
|------|------|------|
| Oxc JS/TS 解析 | ✅ | 坏 JS 可部分恢复 |
| `collapsed_string` + `EXPR` | ✅ | 字符串拼接折叠 |
| `maybe_url` / `resolved_maybe_url` | ✅ | URL 启发式；fetch 拒绝 EXPR |
| 去重 Endpoint > Path | ✅ | `dedup.rs` |
| HTML `<script>` 抽取 | ❌ | Phase 1+ / Phase 3 |
| 并行目录扫描 | ❌ | Phase 3 |
| YAML 自定义规则 | ❌ | Phase 3 |
| jsluice 对比测试集 | ⚠️ | 有 fixture，无自动对比 |

---

## 4. 架构（当前实现）

```mermaid
flowchart TB
    subgraph CLI["spoor-cli"]
        scan[scan / paths / apis / keys]
    end

    subgraph Core["spoor-core"]
        A[Analyzer::collect_findings]
        P[Oxc Parser]
        D[dedup_findings]

        subgraph Matchers["Matcher 流水线"]
            F[fetch]
            L[location]
            X[xhr]
            Lit[literal → path]
        end
    end

    scan --> A
    A --> P
    P --> F --> L --> X --> Lit --> D
    D --> JSON["ScanResult JSON / JSONL"]
```

**输出模型（plan2）：** 每条 finding 含 `kind`、`value`、`confidence`、`origin`（pattern / snippet / line）；endpoint 额外有 `method`；secret 将有 `secret_type` / `severity`（Phase 2）。

---

## 5. 分阶段规划详情

### Phase 2 — 密钥与红队增强（约 2 周，下一步）

**交付目标：** `spoor keys` 有真实输出；`spoor scan` 一次跑 path + endpoint + secret。

| 工作包 | 内容 |
|--------|------|
| Secrets | AWS/GCP/GitHub/Firebase、API Key 模式、对象键启发 |
| URL 增强 | axios、WebSocket、GraphQL、vue/react-router 路径 |
| 其它 | Source map URL、`query_params` 从 URL 解析 |
| 质量 | 收紧 XHR 匹配、sample 里 AKIA 字符串应被 keys 检出 |

### Phase 3 — 性能与工程化（约 2 周）

| 工作包 | 内容 |
|--------|------|
| 规模 | 目录递归 + `rayon` 并行 |
| 扩展 | `Matcher` trait 注册、YAML 规则文件 |
| DX | `--no-literals`、`--min-severity`、Burp/ffuf 管道示例 |
| 质量 | hyperfine 对比 jsluice（目标 ≥5× 单文件） |

### Phase 4 — 高级能力（按需）

Minified regex 兜底、Webpack 动态 import、模块图、SARIF、WASM/NAPI。

---

## 6. 测试与质量现状

| 类型 | Phase 0 | Phase 1 | 目标 |
|------|---------|---------|------|
| 单元测试 | ✅ string_fold, url | ✅ matcher, dedup, analyzer | 持续 |
| 集成 fixture | sample.js, broken.js | fetch/location/xhr, phase1/combined | + jsluice 对比 |
| CLI 测试 | ❌ | ❌ | insta 快照 |
| fmt / clippy | ✅ | ✅ | CI 固化 |
| 模糊测试 | ❌ | ❌ | Phase 4 |

---

## 7. 已知限制（使用前要知）

1. **单文件扫描** — 暂不支持递归目录。
2. **动态 URL** — `fetch(base + "/path")` 不会产出 endpoint（仅 literal `/path` 可能作为 path）。
3. **XHR 误报** — 任意 `.open(method, url)` 都可能命中。
4. **密钥** — `sample.js` 里的 `AKIA…` 目前不会被 `spoor keys` 报告。
5. **无 HTML** — 不能直接丢 `.html` 抽 script。

---

## 8. 文档索引

| 文档 | 用途 |
|------|------|
| [README.md](../README.md) | 安装、用法、Phase 1 能力摘要 |
| **本文档** | 整体进度与功能全景 |
| [plan1.md](../plan1.md) | 总体计划与 Phase 0–1 进度 |
| [plan2.md](../plan2.md) | 命名 + JSON 模型 |
| [plan3.md](../plan3.md) | Phase 2 详细计划 |
| [plan4.md](../plan4.md) | Phase 3 详细计划 |
| [plan5.md](../plan5.md) | Phase 4 详细计划 |
| [Phase 0 retro](./superpowers/plans/2026-05-31-spoor-phase-0-retro.md) | Phase 0 签 off 证据 |
| [Phase 1 retro](./superpowers/plans/2026-05-31-spoor-phase-1-retro.md) | Phase 1 签 off 证据 |
| [Phase 1 plan](./superpowers/plans/2026-05-31-spoor-phase-1.md) | Phase 1 实施任务（已完成） |

---

## 9. 建议的下一步优先级

1. **Phase 2 实施计划** — `writing-plans` 写 `docs/superpowers/plans/spoor-phase-2.md`
2. **secrets matcher** — 让 `spoor keys` 与 `scan` 真正可用
3. **axios + jQuery** — 覆盖现代前端最常见客户端
4. **目录扫描** — 红队扫 `dist/` / `.next/` 的刚需
5. **jsluice 对比测试** — 量化「对标」完成度

---

*维护说明：每完成一个 Phase 签 off 时，更新 §2 进度条、§3 功能矩阵，并新增对应 retro 文档链接。*
