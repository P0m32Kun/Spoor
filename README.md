# Spoor

消费 **Katana（等）提取的 JavaScript 文件**，静态产出 **路径、API 端点、敏感信息** 三类结果，供 ffuf / 手工排查等下游使用。基于 [Oxc](https://oxc.rs/) 解析。

**仓库**: [github.com/P0m32Kun/Spoor](https://github.com/P0m32Kun/Spoor)

**文档：** [plan1 范围与进度](./plan1.md) · [JSON 模型](./plan2.md) · [Agent 指导](./AGENTS.md) · [进度快照](./docs/DEVELOPMENT.md)

## 工具链位置

```
Katana（爬取 + 抽 JS）→ Spoor（分析单个 .js）→ 下游
```

Spoor **不负责**爬站、HTML 抽取或目录扫描；批量处理请对 Katana 输出的 JS 循环调用：

```bash
for f in ./katana-out/*.js; do spoor scan "$f" --jsonl >> all.jsonl; done
```

## 要求

- Rust **1.93+**（推荐 1.94）

## 安装

```bash
git clone git@github.com:P0m32Kun/Spoor.git
cd Spoor
cargo install --path crates/spoor-cli
```

## 用法

```bash
spoor scan app.js              # path + endpoint + secret
spoor paths app.js             # 仅 path
spoor apis app.js              # 仅 endpoint
spoor keys app.js              # 仅 secret
cat bundle.js | spoor scan -   # stdin
spoor scan app.js --jsonl       # 行式 JSON，便于管道
```

## 能力摘要

**Endpoint：** fetch、location、XHR、axios、ky、got、superagent、jQuery、window.open、WebSocket、GraphQL

**Path：** 字面量、react/vue `router.path`、sourceMappingURL

**Secret：** AKIA、GCP（AIza）、Firebase、service account、`sk-`、GitHub token、对象敏感键

**限制：** 单文件输入；动态拼接 URL（如 `fetch(base + "/x")`）通常不产出 endpoint。

## 输出模型

三类 finding：`path`、`endpoint`、`secret`。详见 [plan2.md](./plan2.md)。

## 开发

```bash
cargo test --workspace
cargo run -p spoor-cli -- scan tests/fixtures/sample.js
```

## License

MIT
