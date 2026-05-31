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
spoor scan ./dist/app.js          # 扫描（当前 Phase 0：字符串字面量路径）
spoor paths ./dist/               # 仅路径
spoor apis ./bundle.js            # 接口（Phase 1+）
spoor keys ./config.js            # 密钥（Phase 2+）
cat page.js | spoor paths -        # 从 stdin 读取
spoor scan ./dist -o out.json     # 写入文件
spoor scan ./dist --jsonl         # 每行一个 finding
```

## 输出模型

三类 finding：`path`（站内路径）、`endpoint`（可发起的 API）、`secret`（密钥/凭证）。示例见 [plan2.md](./plan2.md)。

## 开发

```bash
cargo test --workspace
cargo run -p spoor-cli -- paths tests/fixtures/sample.js
```

## 路线图

见 [plan1.md](./plan1.md)（阶段划分）与 [plan2.md](./plan2.md)（命名与 JSON 设计）。

## License

MIT
