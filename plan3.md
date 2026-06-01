# Spoor Phase 2 计划 — 密钥与红队增强

> **上级：** [plan1.md](./plan1.md) § Phase 2  
> **前置：** Phase 0、Phase 1 核心已签 off  
> **预估：** 约 2 周

**Goal：** `spoor keys` 有真实输出；`spoor scan` 一次产出 path + endpoint + secret；补齐现代前端 HTTP 客户端与路由类 URL。

---

## 交付定义

- [ ] `spoor keys tests/fixtures/sample.js` 检出 AKIA 类样本
- [ ] `spoor scan` 三类 finding 同屏输出
- [ ] 至少新增 axios + jQuery matcher
- [ ] XHR matcher 收紧（减少任意 `.open` 误报）
- [ ] `cargo test --workspace` 全绿

---

## Task A：Secret Matcher 基础

**Files（预期）：**
- Create: `crates/spoor-core/src/matcher/secret.rs`
- Create: `crates/spoor-core/src/secrets/`（或单文件模式）
- Modify: `crates/spoor-core/src/finding.rs` — `Finding::secret()`
- Modify: `crates/spoor-core/src/analyzer.rs` — 接入 secret 流水线

**Checklist：**

- [ ] AWS Access Key（AKIA…）模式
- [ ] 常见 API Key：`sk-`、Bearer 前缀、GitHub token 等
- [ ] 对象字面量键启发：`apiKey`、`secret`、`token`、`password`（保守，可配置 severity）
- [ ] 输出 `secret_type`、`severity`、`context.nearby_keys`
- [ ] Fixture：`tests/fixtures/secrets.js`
- [ ] 测试：`sample.js` 中 AKIA 被检出

---

## Task B：云厂商与扩展 Secret（对标 jsluice）

- [ ] GCP 凭证模式
- [ ] Firebase 配置
- [ ] GitHub PAT / fine-grained token 粗匹配
- [ ] （可选）`REACT_APP_*` / `VITE_*` / `process.env.X` 字符串

---

## Task C：HTTP 客户端 Matcher

| Matcher | 触发 | 优先级 |
|---------|------|--------|
| axios | `axios.get/post/request(...)` | P0 |
| jQuery | `$.get` / `$.post` / `$.ajax` | P0 |
| ky / got / superagent | 常见调用形式 | P1 |
| window.open | `window.open(url)` | P1 |

**Checklist：**

- [ ] 各 matcher 独立 fixture + 测试
- [ ] 接入 `collect_findings()`（endpoint 优先级高于 literal）
- [ ] method 解析（与 fetch 一致）

---

## Task D：协议与路由类 URL

- [ ] WebSocket：`new WebSocket(url)`
- [ ] GraphQL：`` gql`...` ``、`/graphql` 路径粗匹配
- [ ] react-router / vue-router path 字符串（高价值 path finding）
- [ ] `//# sourceMappingURL=` 提取

---

## Task E：URL 增强与质量

- [ ] 从 endpoint value 解析 `params.query`（query string 参数名）
- [ ] 收紧 `xhr.rs`：关联 `XMLHttpRequest` 实例或 `new XMLHttpRequest()`
- [ ] jsluice 可移植用例对比测试（允许集合等价，顺序无关）
- [ ] 统一各 matcher 使用 `resolved_maybe_url`（拒绝 EXPR）

---

## Task F：CLI 与文档

- [ ] 移除 `spoor keys` placeholder 提示
- [ ] README / plan1 checklist 更新
- [ ] Phase 2 retro 文档

---

## 验收命令

```bash
cargo test --workspace
cargo run -p spoor-cli -- keys tests/fixtures/secrets.js
cargo run -p spoor-cli -- scan tests/fixtures/sample.js
cargo clippy --workspace -- -D warnings
```

---

## 明确不做（留 Phase 3+）

- 目录递归扫描
- YAML 规则文件
- HTML 输入
