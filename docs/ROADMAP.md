# Spoor 路线图与能力

**版本：** 0.1.0 · **测试：** 44 passed · **仓库：** https://github.com/P0m32Kun/Spoor

---

## 1. 定位（范围锁定）

```
Katana（爬取 + 抽 JS）→ Spoor（单文件分析）→ 下游平台
```

| 产出 | `kind` | 说明 |
|------|--------|------|
| 路径 | `path` | 字面量、router.path、source map |
| API | `endpoint` | fetch / XHR / axios / WebSocket 等 |
| 敏感信息 | `secret` | AKIA、GCP、Firebase、token 等 |

**输入：** 单个 JS/TS 文件或 stdin。  
**输出：** JSON / JSONL — 见 [INTEGRATION.md](./INTEGRATION.md)。

### 明确不做

目录递归、HTML 解析、爬站、YAML 规则引擎、SARIF/WASM — 由 Katana 或调用方负责，见 [AGENTS.md](../AGENTS.md)。

---

## 2. 进度

| 阶段 | 状态 | 回顾 |
|------|------|------|
| Phase 0 基础解析 + path | ✅ | [phase-0-retro.md](./history/phase-0-retro.md) |
| Phase 1 endpoint matcher | ✅ | [phase-1-retro.md](./history/phase-1-retro.md) |
| Phase 2 secret + URL 增强 | ✅ | [phase-2-retro.md](./history/phase-2-retro.md) |
| 维护 backlog | 按需 | §5 |

---

## 3. 当前能力

### CLI

`spoor scan` · `spoor paths` · `spoor apis` · `spoor keys` · `-o` · `--jsonl` · stdin `-`

### Matcher 覆盖

**Endpoint：** fetch, location, xhr, axios, ky, got, superagent, jQuery, window.open, WebSocket, GraphQL

**Path：** string literal, router.path, sourceMappingURL

**Secret：** AKIA, GCP/Firebase, service account, sk-, ghp_, 对象键启发

**流水线：** fetch → location → xhr → axios → ky → got → superagent → jquery → window_open → websocket → graphql → router → secret → source_map → literal → dedup

### 测试

- 单元/集成：`cargo test --workspace`（44 tests）
- Katana 合成 fixture：`tests/fixtures/katana/`
- jsluice parity（可选）：`cargo test -p spoor-core jsluice_parity`

---

## 4. 已知限制

1. 单文件 — 批量用 shell 循环 `spoor scan`
2. 动态 URL `fetch(a + "/b")` — 无可靠 endpoint
3. 不支持 HTML 输入
4. `process.env` secret — backlog 未做

---

## 5. 维护 backlog

| 项 | 优先级 |
|----|--------|
| 统一 `resolved_maybe_url` | 中 |
| env 字符串 secret | 低 |
| CI（test + fmt + clippy） | 低 |
| CLI JSON 快照 | 低 |

---

## 6. 与 jsluice

对标参考，输出 schema 不同（三类 `kind`）。parity 测试见 `crates/spoor-core/src/jsluice_parity.rs`。
