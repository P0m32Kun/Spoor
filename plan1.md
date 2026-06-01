# js-rs 开发计划

对标 [jsluice](https://github.com/BishopFox/jsluice)（Go + tree-sitter，约 3 年未大更新），在 Rust 里做一套**更快、可扩展、面向红队信息收集**的 JavaScript 静态分析工具。仓库 `js-rs` 目前只有空的 Cargo 骨架，适合按下面计划推进。

---

## 一、目标与定位

| 维度 | jsluice | js-rs 目标 |
|------|---------|------------|
| 解析 | go-tree-sitter | **Oxc**（高性能 AST，持续维护） |
| 场景 | 通用安全扫描 | **红队收参**：端点、路径、密钥、配置、GraphQL/WebSocket 等 |
| 输出 | JSON | JSON / JSONL + 可选 SARIF，便于接 ffuf、nuclei、自定义 pipeline |
| 扩展 | Go 回调 matcher | **Rust trait + 可选 YAML/TOML 规则** |
| 性能 | 单文件尚可 | **并行批处理**、大 bundle / 目录扫描 |

**核心原则**：先做到 jsluice 能力对等且更稳，再在红队场景上明显超出（而不是一上来就堆功能）。

---

## 二、技术选型（建议）

```
┌─────────────────────────────────────────────────────────┐
│  CLI (clap)          │  批处理 / 管道 / 目录递归        │
├─────────────────────────────────────────────────────────┤
│  js-rs-core          │  Analyzer、Matcher、Findings     │
├──────────┬──────────┬──────────┬────────────────────────┤
│ extract  │ secrets  │ strings  │ html (内联 script)     │
├──────────┴──────────┴──────────┴────────────────────────┤
│  Oxc Parser + ast_visit  │  可选：regex 兜底（minified）│
├─────────────────────────────────────────────────────────┤
│  url crate / regex / serde / rayon                      │
└─────────────────────────────────────────────────────────┘
```

| 组件 | 选择 | 理由 |
|------|------|------|
| JS 解析 | `oxc_parser` + `oxc_ast` + `oxc_ast_visit` | 比 tree-sitter 更适合语义遍历；TS/JSX 支持好；生态活跃 |
| 并行 | `rayon` | 目录 / 多文件扫描 |
| CLI | `clap` + `serde_json` | 与 jsluice CLI 习惯对齐 |
| HTML | `scraper` 或 `lol_html` | 抽 `<script>` 内联 JS（对标 jsluice） |
| 规则 | 内置 + 后续 `rules/*.yaml` | 红队可自定义 matcher，无需改代码 |

**暂不引入**：完整语义分析（`oxc_semantic`）、反混淆、动态执行——放到后期可选模块。

---

## 三、模块划分

```
js-rs/
├── crates/
│   ├── js-rs-core/      # 库：Analyzer、类型、Matcher trait
│   ├── js-rs-extract/   # URL / 路径 / 端点
│   ├── js-rs-secrets/   # 密钥与敏感配置
│   └── js-rs-cli/       # 二进制 js-rs
├── rules/               # 可选：用户规则（Phase 3+）
└── tests/fixtures/      # 从 jsluice / 真实 bundle 摘样例
```

**核心类型（与 jsluice 对齐，便于迁移）**：

- `Finding::Url` — `url`, `method`, `query_params`, `body_params`, `kind`（fetch / location / xhr…）, `source`, `file`
- `Finding::Secret` — `kind`, `severity`, `data`, `context`
- `Finding::String` — 高价值字符串（可选 Phase 2）
- 字符串拼接折叠：未知表达式 → `EXPR`（与 jsluice 一致，便于爬取与 fuzz）

---

## 四、分阶段里程碑

### Phase 0 — 基础（约 1 周）

**交付**：能解析 JS，跑通一条最小链路。

- [x] Workspace：`spoor-core` + `spoor-cli`
- [x] `Analyzer::new(source)` → Oxc 解析，错误可恢复（坏 JS 仍尽量出结果）
- [x] AST 遍历骨架（`Visit`）
- [x] `collapsed_string()`：字面量拼接 + `EXPR` 占位
- [x] `maybe_url()` 启发式（移植 jsluice 逻辑）
- [x] 单元测试：拼接、`EXPR`、单字面量（转义字符串测试已替换）

### Phase 1 — URL 提取对标 jsluice（约 2 周）

**交付**：`spoor apis file.js` 输出语义级 `endpoint` finding（plan2 JSON 模型）。

内置 matcher（优先级从高到低）：

| Matcher | 触发模式 | Phase 1 状态 |
|---------|----------|--------------|
| fetch | `fetch(url, init)` | ✅ `matcher/fetch.rs` |
| location | `location` / `.href` 赋值 | ✅ |
| location.replace | `location.replace(...)` | ✅ |
| XHR | `.open(method, url)` | ⚠️ 已实现，匹配较宽 |
| string literal | 兜底 path | ✅ + 去重 |
| jQuery | `$.get/post/ajax` | ❌ Phase 2+ |
| window.open | `window.open` / `open` | ❌ Phase 2+ |
| 泛化 call | 首参像 URL 的任意调用 | ❌ Phase 2+ |

其它：

- [x] 过滤 `data:` / `tel:` / `javascript:`（`maybe_url`）；fetch 侧拒绝含 `EXPR` 的动态 URL
- [ ] 从 URL 解析 `query_params`
- [x] 去重（Endpoint > Path）
- [ ] HTML 输入：抽 `<script>` 再分析
- [x] CLI：`spoor apis` / `spoor scan` / stdin / 单文件

**验收**：`tests/fixtures/phase1/combined.js` + 单 matcher fixture；jsluice 集合对比留 Phase 1.5。

回顾文档：[docs/superpowers/plans/2026-05-31-spoor-phase-1-retro.md](./docs/superpowers/plans/2026-05-31-spoor-phase-1-retro.md)

### Phase 2 — 密钥与红队增强（约 2 周）

**交付**：`js-rs secrets`、`js-rs scan`（URL + secrets 一次跑完）。

**Secrets（移植并扩展）**：

- [ ] AWS / GCP / GitHub / Firebase（对标 jsluice）
- [ ] 常见 API Key 模式（Bearer、sk-、AKIA 等）
- [ ] 对象键启发：`apiKey`、`secret`、`token`、`password`（可控 FP，默认保守）
- [ ] `REACT_APP_*`、`VITE_*`、`process.env` 相关（可选，带 severity）

**红队向 URL/端点增强**：

- [ ] `axios` / `ky` / `got` / `superagent`
- [ ] WebSocket：`new WebSocket(...)`
- [ ] GraphQL：`/graphql`、`` gql`...` `` 粗匹配
- [ ] 路由：`react-router`、`vue-router` path 字符串
- [ ] Source map：`//# sourceMappingURL=`
- [ ] 相对路径规范化辅助（为目录 fuzz 准备，不强制做完整 base URL 解析）

### Phase 3 — 性能、规则与工程化（约 2 周）

- [ ] 目录递归 + `rayon` 并行
- [ ] 去重策略：同 URL 保留 `kind` 信息最丰富的一条
- [ ] `Matcher` trait：`UrlMatcher` / `SecretMatcher` 可注册
- [ ] 可选规则文件（YAML）：正则 + AST 查询描述（简化版）
- [ ] 基准：对比 jsluice（同机 `hyperfine`），目标 **≥5×** 单文件、**≥10×** 千文件目录
- [ ] README：安装、示例、与 Burp/ffuf 管道示例

### Phase 4 — 可选高级能力（按需）

- [ ] Minified / 语法错误严重：regex 二层兜底
- [ ] Webpack chunk 名 / 动态 `import()` 路径
- [ ] 与 `oxc_resolver` 做简单模块图（大型 SPA）
- [ ] 输出 OpenAPI 粗猜测、SARIF
- [ ] `wasm` / `napi` 供 Node/Python 调用

---

## 五、与 jsluice 的差异设计（红队价值）

1. **更全的 HTTP 客户端**：现代前端很少只用裸 `fetch`，Phase 2 覆盖主流库。
2. **可配置噪音**：`--min-severity`、`--no-literals`（只要语义 URL，不要纯字符串）。
3. **管道友好**：`find . -name '*.js' | js-rs scan --jsonl`，每行一个 finding，便于 `jq`。
4. **并行与规模**：整站 `assets/`、`.next/`、`dist/` 扫描是常态。
5. **规则可热更新**：红队 IOC、内部域名模式不必发版。

---

## 六、测试与质量

| 类型 | 内容 |
|------|------|
| 单元测试 | `collapsed_string`、`maybe_url`、各 matcher 小片段 |
| 快照测试 | CLI JSON 输出（`insta`） |
| 对比测试 | 选 jsluice 仓库 `analyzer_test.go` 中可移植用例 |
| 模糊测试（后期） | 随机 JS 片段，保证不 panic |
| 真实样本 | 脱敏的 bundle、Vue/React 构建产物各 2～3 份 |

---

## 七、CLI 草案（与 jsluice 习惯兼容）

```bash
js-rs urls app.js              # 仅 URL
js-rs secrets bundle.js        # 仅密钥
js-rs scan ./dist/             # 递归全量
js-rs scan -j --min-severity medium ./  # JSONL
cat page.html | js-rs urls -   # stdin
```

---

## 八、风险与对策

| 风险 | 对策 |
|------|------|
| Oxc MSRV 较高（当前约 1.93+） | 文档写明；你本机 1.94 可满足 |
| 误报（字符串字面量过多） | `MaybeURL` + matcher 优先级 + `--no-literals` |
| 严重混淆 JS | Phase 4 regex 兜底；不承诺 100% |
| 与 jsluice 输出不完全一致 | 以「集合等价 + 文档差异」为验收，不追求字节级相同 |

---

## 九、建议的近期执行顺序

1. **本周**：Phase 0 + Phase 1 的 `fetch` / `location` / `string` 三个 matcher + CLI `urls`
2. **下周**：XHR、jQuery、secrets 移植 + `scan` 子命令
3. **第三周**：红队增强 matcher + 并行目录扫描
4. **第四周**：基准、README、规则文件雏形

---

如果你认可这个方向，下一步可以从 **Phase 0 + Phase 1 的 workspace 骨架和 `fetch`/`location` matcher** 开始写代码；也可以先定几件事再动手：

- 项目名用 `js-rs` 还是 `jsluice-rs` / 别的 CLI 名？
- Phase 2 里你最想优先的红队能力（例如 axios、GraphQL、环境变量）？
- 是否需要默认兼容 jsluice 的 JSON 字段名，方便你现有脚本直接替换？
