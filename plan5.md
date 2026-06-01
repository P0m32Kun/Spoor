# Spoor Phase 4 计划 — 高级能力（按需）

> **上级：** [plan1.md](./plan1.md) § Phase 4  
> **前置：** Phase 3 完成  
**性质：** 可选模块，按实际需求择项实施

---

## 目标场景

- 严重 minified / 语法损坏的 JS 仍尽量出结果
- 大型 SPA 模块关系与动态 chunk
- 与其他工具链集成（SARIF、Node/Python）

---

## Task A：Regex 二层兜底

- [ ] 解析失败或 AST 为空时，对源码跑预编译 URL/secret 正则
- [ ] 与 AST finding 合并去重
- [ ] 标记 `origin.pattern: "regex_fallback"` 降低 confidence

---

## Task B：Bundler / 动态导入

- [ ] 动态 `import("./chunk-xxx.js")` 路径提取
- [ ] Webpack magic comment 粗匹配
- [ ] `__webpack_require__` 等常见模式（低优先级）

---

## Task C：模块图（实验性）

- [ ] `oxc_resolver` 简单 resolve
- [ ] 输出模块依赖边（JSON sidecar，非默认 CLI）

---

## Task D：输出格式扩展

- [ ] SARIF 导出
- [ ] OpenAPI 粗猜测（从 endpoint 集合生成 stub，研究性质）

---

## Task E：跨语言绑定

- [ ] `wasm` 构建 + 浏览器/Node 调用 PoC
- [ ] 或 `napi` / `pyo3` 供 Python 管道

---

## Task F：模糊测试

- [ ] `cargo fuzz` 或随机 JS 片段不 panic
- [ ] 坏输入 corpus 来自真实 bundle 脱敏片段

---

## 验收原则

每项独立验收，不要求 Phase 4 全做完才发布。完成子项即 patch version + CHANGELOG。
