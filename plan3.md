# Spoor Phase 2 计划 — 密钥与红队增强

> **上级：** [plan1.md](./plan1.md) § Phase 2  
> **前置：** Phase 0、Phase 1 已签 off  
> **预估：** 约 2 周 · **当前进度：** ~70%

**Goal：** `spoor keys` 有真实输出；`spoor scan` 一次产出 path + endpoint + secret；补齐现代前端 HTTP 客户端与路由类 URL。

---

## 交付定义

- [x] `spoor keys tests/fixtures/secrets.js` 检出 AKIA 类样本
- [x] `spoor scan` 三类 finding 同屏输出
- [x] 至少新增 axios + jQuery matcher
- [x] XHR matcher 收紧（减少任意 `.open` 误报）
- [x] `cargo test --workspace` 全绿（32 passed）

---

## Task A：Secret Matcher 基础 ✅

**Files：**
- `crates/spoor-core/src/matcher/secret.rs`
- `crates/spoor-core/src/finding.rs` — `Finding::secret()`
- `crates/spoor-core/src/analyzer.rs` — 接入 secret 流水线

**Checklist：**

- [x] AWS Access Key（AKIA…）模式
- [x] 常见 API Key：`sk-`、GitHub token 粗匹配
- [x] 对象字面量键启发：`apiKey`、`secret`、`token`、`password`
- [x] 输出 `secret_type`、`severity`、`context.nearby_keys`
- [x] Fixture：`tests/fixtures/secrets.js`
- [x] 测试：AKIA / sk- / 对象键被检出

---

## Task B：云厂商与扩展 Secret（对标 jsluice）✅（基础）

- [x] GCP API Key（`AIza…` 39 字符）
- [x] GCP Service Account（`type: service_account` + `private_key`）
- [x] Firebase 配置（`projectId` + `authDomain` + `apiKey`）
- [x] GitHub PAT / fine-grained token 粗匹配（Task A 已有）
- [ ] （可选）`REACT_APP_*` / `VITE_*` / `process.env.X` 字符串

---

## Task C：HTTP 客户端 Matcher ✅（P0 部分）

| Matcher | 触发 | 优先级 | 状态 |
|---------|------|--------|------|
| axios | `axios.get/post/request(...)` | P0 | ✅ |
| jQuery | `$.get` / `$.post` / `$.ajax` | P0 | ✅ |
| window.open | `window.open(url)` | P1 | ✅ |
| ky / got / superagent | 常见调用形式 | P1 | ❌ |

**Checklist：**

- [x] 各 matcher 独立 fixture + 测试
- [x] 接入 `collect_findings()`（endpoint 优先级高于 literal）
- [x] method 解析（与 fetch 一致）

---

## Task D：协议与路由类 URL（大部分完成）

- [x] WebSocket：`new WebSocket(url)`
- [x] GraphQL：`` gql`...` `` 提示 + `graphql(url, …)` 请求
- [x] react-router / vue-router path 字符串（`router.path`）
- [x] `//# sourceMappingURL=` 提取

---

## Task E：URL 增强与质量（部分完成）

- [x] 从 endpoint value 解析 `params.query`（query string 参数名）
- [x] 收紧 `xhr.rs`：关联 `new XMLHttpRequest()` 实例
- [x] jsluice 可移植用例对比 fixture（`jsluice_subset.js`）
- [ ] 统一各 matcher 使用 `resolved_maybe_url`（location 仍用 `is_location_url`）

---

## Task F：CLI 与文档（部分完成）

- [x] 移除 `spoor keys` placeholder 提示
- [x] README / plan1 checklist 更新
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
