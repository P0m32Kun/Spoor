# Spoor

消费 **Katana 等工具提取的 JavaScript 文件**，静态输出 **路径、API 端点、敏感信息**。

| 文档 | 说明 |
|------|------|
| **[docs/INTEGRATION.md](./docs/INTEGRATION.md)** | 平台集成：命令、参数、JSON 格式 |
| [docs/ROADMAP.md](./docs/ROADMAP.md) | 范围、能力、进度 |
| [docs/README.md](./docs/README.md) | 文档索引 |
| [AGENTS.md](./AGENTS.md) | AI Agent 范围约束 |

**仓库:** [github.com/P0m32Kun/Spoor](https://github.com/P0m32Kun/Spoor)

## 快速开始

```bash
cargo install --path crates/spoor-cli   # Rust 1.93+

spoor scan app.js              # path + endpoint + secret
spoor scan app.js --jsonl      # 平台集成推荐
spoor apis app.js              # 仅 endpoint
spoor keys app.js              # 仅 secret
```

Katana 批量：

```bash
for f in ./katana-out/*.js; do spoor scan "$f" --jsonl >> findings.jsonl; done
```

## 输出概要

标准 JSON：`{ "file": "...", "findings": [ { "kind": "path|endpoint|secret", ... } ] }`

完整 schema → [INTEGRATION.md](./docs/INTEGRATION.md)

## 开发

```bash
cargo test --workspace
```

## License

MIT
