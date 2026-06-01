# Spoor

在 JavaScript / TypeScript 资产里追踪**路径足迹、API 调用痕迹与密钥泄漏痕迹**的静态分析工具，面向红队信息收集场景。对标 [jsluice](https://github.com/BishopFox/jsluice)，基于 [Oxc](https://oxc.rs/) 解析，目标更快、可扩展、管道友好。

**仓库**: [github.com/P0m32Kun/Spoor](https://github.com/P0m32Kun/Spoor)

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
spoor scan tests/fixtures/phase1/combined.js   # 路径 + API endpoint（Phase 1）
spoor paths tests/fixtures/sample.js           # 仅 path finding
spoor apis tests/fixtures/phase1/combined.js   # 仅 endpoint（fetch / location / XHR）
spoor keys ./config.js                         # 密钥（Phase 2+）
cat page.js | spoor paths -                    # 从 stdin 读取
spoor scan ./dist -o out.json                  # 写入文件
spoor scan ./dist --jsonl                      # 每行一个 finding
```

### Phase 1 能力

`spoor apis` 与 `spoor scan` 中的 endpoint 类 finding 已支持：

- **fetch** — 首参为可折叠字面量 URL
- **location** — `href`、`replace`、`assign` 及 `window.location = …`
- **XMLHttpRequest** — `xhr.open(method, url)`

验收 fixture：`tests/fixtures/phase1/combined.js`（同文件混合上述三种调用）。

**限制**：动态拼接 URL（如 `fetch(base + "/path")`）在 Phase 1 可能折叠为 `EXPR` 占位符，不会产出可靠 endpoint；仅静态字面量或简单拼接会被识别。

## 输出模型

三类 finding：`path`（站内路径）、`endpoint`（可发起的 API）、`secret`（密钥/凭证）。示例见 [plan2.md](./plan2.md)。

## 开发

```bash
cargo test --workspace
cargo run -p spoor-cli -- paths tests/fixtures/sample.js
cargo run -p spoor-cli -- apis tests/fixtures/phase1/combined.js
```

## 路线图

见 [plan1.md](./plan1.md)（阶段划分）与 [plan2.md](./plan2.md)（命名与 JSON 设计）。

阶段回顾与完成度：

- [Phase 0 retro](./docs/superpowers/plans/2026-05-31-spoor-phase-0-retro.md)
- [Phase 1 retro](./docs/superpowers/plans/2026-05-31-spoor-phase-1-retro.md)

## License

MIT
