# Spoor

消费 **Katana 等工具提取的 JavaScript 文件**，静态输出 **路径、API 端点、敏感信息**，供下游平台集成。

**完整集成文档（命令、参数、JSON 格式）：** [docs/INTEGRATION.md](./docs/INTEGRATION.md)

**仓库**: [github.com/P0m32Kun/Spoor](https://github.com/P0m32Kun/Spoor)

---

## 工具链

```
Katana → Spoor（本工具，单 .js 文件）→ 你的平台 / ffuf / 告警
```

## 快速开始

```bash
cargo install --path crates/spoor-cli   # 需 Rust 1.93+

spoor scan app.js              # path + endpoint + secret → JSON
spoor scan app.js --jsonl      # 每行一个 finding（平台集成推荐）
spoor apis app.js              # 仅 endpoint
spoor keys app.js              # 仅 secret
```

## 输出概要

**标准 JSON：**

```json
{
  "file": "app.js",
  "findings": [
    { "kind": "endpoint", "value": "/api/users", "method": "GET", "confidence": "high", "origin": { "pattern": "fetch", "line": 10 } },
    { "kind": "secret", "value": "AKIA...", "secret_type": "aws_access_key", "severity": "critical", "origin": { "pattern": "string_literal" } }
  ]
}
```

**JSONL（`--jsonl`）：** 每行一个 `Finding`，无 `file` 字段 — 详见 [集成指南](./docs/INTEGRATION.md)。

## 文档

| 文档 | 说明 |
|------|------|
| **[docs/INTEGRATION.md](./docs/INTEGRATION.md)** | **集成指南（参数 + 完整 schema）** |
| [plan2.md](./plan2.md) | JSON 模型设计 |
| [AGENTS.md](./AGENTS.md) | 项目范围 |
| [docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md) | 能力快照 |

## 开发

```bash
cargo test --workspace
cargo test -p spoor-core jsluice_parity   # 可选，需 jsluice
```

## License

MIT
