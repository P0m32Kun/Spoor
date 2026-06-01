# Spoor

消费 **Katana 等工具提取的 JavaScript 文件**，静态输出 **路径、API 端点、敏感信息**。

| 文档 | 说明 |
|------|------|
| **[docs/INTEGRATION.md](./docs/INTEGRATION.md)** | 平台集成：命令、参数、JSON 格式 |
| [docs/ROADMAP.md](./docs/ROADMAP.md) | 范围、能力、进度 |
| [docs/README.md](./docs/README.md) | 文档索引 |
| [AGENTS.md](./AGENTS.md) | AI Agent 范围约束 |

**仓库:** [github.com/P0m32Kun/Spoor](https://github.com/P0m32Kun/Spoor)

## 安装

### 预编译二进制（推荐：Docker / 平台集成）

在 [GitHub Releases](https://github.com/P0m32Kun/Spoor/releases) 下载对应平台包（共 6 个：Linux / macOS / Windows × x86_64 / ARM64），解压后将 `spoor` 放入 `PATH`：

|  | x86_64 | ARM64 |
|--|--------|-------|
| Linux | `spoor-x86_64-unknown-linux-gnu.tar.gz` | `spoor-aarch64-unknown-linux-gnu.tar.gz` |
| macOS | `spoor-x86_64-apple-darwin.tar.gz` | `spoor-aarch64-apple-darwin.tar.gz` |
| Windows | `spoor-x86_64-pc-windows-msvc.zip` | `spoor-aarch64-pc-windows-msvc.zip` |

```bash
# 示例：Linux x86_64
VERSION=0.2.0
curl -fsSL -o /tmp/spoor.tgz \
  "https://github.com/P0m32Kun/Spoor/releases/download/v${VERSION}/spoor-x86_64-unknown-linux-gnu.tar.gz"
tar xzf /tmp/spoor.tgz -C /usr/local/bin
spoor --version
```

每个资产附带 `.sha256` 校验文件。

### 从源码

```bash
cargo install --path crates/spoor-cli   # Rust 1.93+
```

## 快速开始

```bash
cargo install --path crates/spoor-cli   # 或见上方 Releases 预编译包

# 扫描 JS URL（拉取 → 分析 → 拼接 → 探测）
spoor scan "http://192.168.1.8:18080/1.js" --jsonl

# URL 列表文件（每行一个 URL）
spoor scan katana-urls.txt --jsonl

spoor apis ./local.js          # 本地文件用 paths/apis/keys
spoor keys ./local.js
```

Katana 批量：把 JS URL 写入列表文件后一次扫描：

```bash
spoor scan katana-urls.txt --jsonl >> findings.jsonl
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
