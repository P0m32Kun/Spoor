# Spoor Phase 2 计划 — 密钥与 JS 内 URL 增强

> **上级：** [plan1.md](./plan1.md) § Phase 2  
> **状态：** ✅ 已签 off（2026-05-31）  
> **回顾：** [Phase 2 retro](./docs/superpowers/plans/2026-05-31-spoor-phase-2-retro.md)

**Goal：** 在 **Katana 下游单 JS 文件** 上，补齐 secret 提取与现代前端 endpoint/path 模式；`spoor scan` 一次产出 path + endpoint + secret。

**范围提醒：** 不做目录扫描、HTML、YAML 规则 — 见 [plan1.md §1.3](./plan1.md)。

---

## 交付定义 ✅

- [x] `spoor keys tests/fixtures/secrets.js` 检出 AKIA / GCP / Firebase
- [x] `spoor scan` 三类 finding 同屏输出
- [x] axios + jQuery + ky + got + superagent
- [x] XHR matcher 收紧
- [x] `cargo test --workspace` 全绿（35 passed）
- [x] Phase 2 retro

---

## Task A：Secret Matcher 基础 ✅

- [x] AWS Access Key（AKIA…）
- [x] `sk-`、GitHub token 粗匹配
- [x] 对象字面量键启发
- [x] `secret_type` / `severity` / `context.nearby_keys`
- [x] Fixture：`tests/fixtures/secrets.js`

---

## Task B：云厂商 Secret ✅

- [x] GCP API Key（`AIza…` 39 字符）
- [x] GCP Service Account
- [x] Firebase 配置对象
- [ ] （可选 backlog）`REACT_APP_*` / `process.env.X`

---

## Task C：HTTP 客户端 Matcher ✅

| Matcher | 状态 |
|---------|------|
| axios / jQuery / window.open | ✅ |
| ky / got / superagent | ✅ |

---

## Task D：协议与路由 ✅

- [x] WebSocket
- [x] GraphQL（gql 模板 + graphql()）
- [x] react-router / vue-router（`router.path`）
- [x] sourceMappingURL

---

## Task E：质量（部分 → backlog）

- [x] query_params、XHR 收紧、jsluice_subset fixture
- [ ] 统一 `resolved_maybe_url` → [plan1 §六](./plan1.md)

---

## Task F：文档 ✅

- [x] README / plan 更新
- [x] Phase 2 retro

---

## 验收命令

```bash
cargo test --workspace
cargo run -p spoor-cli -- keys tests/fixtures/secrets.js
cargo run -p spoor-cli -- scan tests/fixtures/sample.js
cargo clippy --workspace -- -D warnings
```
