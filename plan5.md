# ~~Spoor Phase 4~~ — 归档（不实施）

> **状态：** 🚫 **OUT OF SCOPE** — 2026-05-31 范围锁定  
> **说明：** [plan1.md §1.3](./plan1.md)

---

Spoor 不追求独立安全平台或跨语言绑定。下列能力**不在产品范围内**：

- Regex 二层兜底（可将来作为单文件解析失败时的极小补丁，非 Phase）
- Webpack 模块图 / `oxc_resolver` 依赖图
- SARIF / OpenAPI 导出
- WASM / NAPI / PyO3
- cargo fuzz

**当前重点：** 在 Katana 提供的 JS 上，把 path / endpoint / secret 提取做准、做稳。

<details>
<summary>展开原 Phase 4 任务列表（历史）</summary>

见 git 历史中的 plan5 完整版。

</details>
