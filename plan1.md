# Spoor 总体开发计划

> **进度快照入口：** [docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md)  
> **JSON 输出模型：** [plan2.md](./plan2.md)  
> **仓库：** https://github.com/P0m32Kun/Spoor

**最后更新：** 2026-05-31 · **版本：** 0.1.0 · **测试：** 19 passed

---

## 一、目标与定位

| 维度 | jsluice | Spoor 目标 |
|------|---------|------------|
| 解析 | go-tree-sitter | **Oxc**（高性能 AST，持续维护） |
| 场景 | 通用安全扫描 | **红队收参**：path、endpoint、secret |
| 输出 | JSON | JSON / JSONL（plan2 三类 `kind`） |
| 扩展 | Go 回调 | Rust matcher 模块 + Phase 3 YAML 规则 |
| 性能 | 单文件尚可 | Phase 3 并行目录扫描 |

**核心原则：** 先对齐 jsluice 核心能力，再在红队场景超出（axios、GraphQL、规则热更新等）。

---

## 二、整体进度

| 阶段 | 进度 | 状态 | 详细计划 |
|------|------|------|----------|
| Phase 0 | 100% | ✅ 已签 off | 下文 § Phase 0 |
| Phase 1 | ~75% | ✅ 核心已签 off | 下文 § Phase 1 · [retro](./docs/superpowers/plans/2026-05-31-spoor-phase-1-retro.md) |
| Phase 2 | 0% | 📋 未开始 | [plan3.md](./plan3.md) |
| Phase 3 | 0% | 📋 未开始 | [plan4.md](./plan4.md) |
| Phase 4 | 0% | 📋 按需 | [plan5.md](./plan5.md) |

---

## 三、技术架构

```
┌─────────────────────────────────────────────────────────┐
│  spoor-cli (clap)    scan / paths / apis / keys         │
├─────────────────────────────────────────────────────────┤
│  spoor-core          Analyzer · MatchContext · dedup    │
├──────────┬──────────┬──────────┬────────────────────────┤
│ matcher/ │ string   │ url      │ finding (plan2 JSON)   │
│ fetch    │ _fold    │ maybe_url│                        │
│ location │          │          │                        │
│ xhr      │          │          │                        │
│ literal  │          │          │                        │
├──────────┴──────────┴──────────┴────────────────────────┤
│  Oxc Parser + ast_visit                                 │
├─────────────────────────────────────────────────────────┤
│  Phase 3+: rayon · scraper(HTML) · rules/*.yaml         │
└─────────────────────────────────────────────────────────┘
```

**当前 matcher 流水线：** fetch → location → xhr → literal → dedup

**Crate 布局（当前）：**

```
Spoor/
├── crates/spoor-core/
├── crates/spoor-cli/
├── tests/fixtures/
└── docs/
```

---

## 四、功能矩阵

### 4.1 CLI

| 命令 / 选项 | 当前 | 计划阶段 |
|-------------|------|----------|
| `spoor scan` | path + endpoint | + secret → Phase 2 |
| `spoor paths` | ✅ 字面量 path | — |
| `spoor apis` | ✅ fetch/location/XHR | + axios 等 → Phase 2 |
| `spoor keys` | ❌ 占位 | Phase 2 |
| stdin `-` | ✅ | — |
| `-o` / `--jsonl` | ✅ | — |
| 目录递归 | ❌ 单文件 | Phase 3 |
| `--no-literals` | ❌ | Phase 3 |
| `--min-severity` | ❌ | Phase 3 |

### 4.2 三类 Finding（plan2）

| kind | 用途 | 当前 |
|------|------|------|
| `path` | 站内路径、静态资源 | ✅ literal matcher |
| `endpoint` | 可发起的 API（含 method） | ✅ 部分 matcher |
| `secret` | 密钥 / 凭证 | ❌ Phase 2 |

---

## 五、分阶段里程碑

### Phase 0 — 基础 ✅

**交付：** 能解析 JS，从字面量提取 path。

- [x] Workspace：`spoor-core` + `spoor-cli`
- [x] `Analyzer` + Oxc 解析，坏 JS 可部分恢复
- [x] AST `Visit` 骨架
- [x] `collapsed_string()` + `EXPR`
- [x] `maybe_url()` 启发式
- [x] 单元测试 + fixture

**回顾：** [Phase 0 retro](./docs/superpowers/plans/2026-05-31-spoor-phase-0-retro.md)

---

### Phase 1 — 语义 Endpoint ✅（核心）

**交付：** `spoor apis` 输出 fetch / location / XHR endpoint；去重；CLI 接入。

**Matcher：**

- [x] fetch
- [x] location（href / replace / assign / window.location）
- [x] XHR `.open`（⚠️ 匹配偏宽，Phase 2 收紧）
- [x] string literal → path
- [ ] jQuery → Phase 2
- [ ] window.open → Phase 2
- [ ] 泛化 call → Phase 2+

**其它：**

- [x] 过滤 data:/tel:/javascript:；fetch 拒绝 EXPR 动态 URL
- [x] 去重 Endpoint > Path
- [x] CLI apis / scan / stdin
- [ ] query_params 解析 → Phase 2
- [ ] HTML `<script>` → Phase 3
- [ ] jsluice 全量对比 fixture → Phase 2/3

**回顾：** [Phase 1 retro](./docs/superpowers/plans/2026-05-31-spoor-phase-1-retro.md)

---

### Phase 2 — 密钥与红队增强 📋

**交付：** `spoor keys` 可用；`spoor scan` 含 secret；更多 HTTP 客户端 matcher。

**详见 [plan3.md](./plan3.md)**

---

### Phase 3 — 性能与工程化 📋

**交付：** 目录并行扫描；YAML 规则；CLI 过滤；性能基准。

**详见 [plan4.md](./plan4.md)**

---

### Phase 4 — 高级能力（按需）📋

**详见 [plan5.md](./plan5.md)**

---

## 六、测试与质量

| 类型 | 状态 | 目标阶段 |
|------|------|----------|
| 单元测试（core） | ✅ 19 tests | 持续 |
| matcher fixture | ✅ phase1/combined 等 | + jsluice 对比 |
| CLI 快照测试 | ❌ | Phase 3 |
| fmt / clippy | ✅ | CI → Phase 3 |
| fuzz 不 panic | ❌ | Phase 4 |

---

## 七、已知限制

1. 仅单文件，不支持目录递归
2. 动态 URL `fetch(a + "/b")` 不产出 endpoint
3. XHR 任意 `.open` 可能误报
4. `spoor keys` 未实现
5. 不支持 HTML 输入

---

## 八、文档索引

| 文档 | 用途 |
|------|------|
| [plan1.md](./plan1.md) | 本文 — 总体计划与进度 |
| [plan2.md](./plan2.md) | 命名 + JSON 模型 |
| [plan3.md](./plan3.md) | Phase 2 详细计划 |
| [plan4.md](./plan4.md) | Phase 3 详细计划 |
| [plan5.md](./plan5.md) | Phase 4 详细计划 |
| [docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md) | 进度快照（与 plan 同步维护） |

---

## 九、建议执行顺序

1. Phase 2：secrets → axios/jQuery → 收紧 XHR
2. Phase 3：目录扫描 → YAML 规则 → 性能基准
3. Phase 4：按需求选用
