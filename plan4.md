# Spoor Phase 3 计划 — 性能、规则与工程化

> **上级：** [plan1.md](./plan1.md) § Phase 3  
> **前置：** Phase 2 完成  
> **预估：** 约 2 周

**Goal：** 能扫整个 `dist/` / `.next/`；规则可配置；管道友好；性能可量化。

---

## 交付定义

- [ ] `spoor scan ./dist/` 递归多文件
- [ ] `rayon` 并行，千文件可接受耗时
- [ ] `--no-literals`、`--min-severity` 可用
- [ ] 可选 `rules/*.yaml` 加载自定义 matcher
- [ ] hyperfine 对比 jsluice 有记录（目标 ≥5× 单文件）
- [ ] CI：test + fmt + clippy

---

## Task A：目录与并行扫描

**Files（预期）：**
- Modify: `crates/spoor-cli/src/main.rs` — 目录 walk
- Add dependency: `rayon`
- Modify: `Analyzer` 或 CLI 层批处理

**Checklist：**

- [ ] 单文件行为不变
- [ ] 目录：递归 `.js` / `.mjs` / `.cjs` / `.ts` / `.tsx`（可配置扩展名）
- [ ] 并行 file-level scan
- [ ] 输出：每文件一个 `ScanResult` 或 JSONL 每行一个 finding（已有 `--jsonl`）
- [ ] 大目录 smoke test fixture

---

## Task B：Matcher 架构 refactor

- [ ] 抽象 `EndpointMatcher` / `SecretMatcher` trait
- [ ] 注册表：按优先级运行 matcher
- [ ] 为 Phase 3 YAML 规则预留 hook

---

## Task C：YAML 规则（简化版）

**目录：** `rules/` 或用户指定 `--rules path`

- [ ] 规则格式：名称、正则、kind、severity、pattern 标签
- [ ] 对源码做 regex 二层匹配（补 AST 漏网）
- [ ] 文档：如何写红队 IOC 规则

---

## Task D：HTML 输入

- [ ] 依赖 `scraper` 或 `lol_html`
- [ ] 抽 `<script>` 内联与 `src` 外链（外链可选只报 URL）
- [ ] `spoor scan page.html` 可用
- [ ] stdin HTML 管道

---

## Task E：CLI 降噪与 DX

- [ ] `--no-literals`：跳过 literal path matcher
- [ ] `--min-severity medium`：过滤 low confidence / low severity secret
- [ ] README：Burp → spoor、ffuf 管道示例
- [ ] `find . -name '*.js' | spoor scan --jsonl -` 示例

---

## Task F：测试与 CI

- [ ] CLI JSON 快照（`insta`）
- [ ] GitHub Actions：`cargo test` + `clippy` + `fmt --check`
- [ ] 基准脚本或文档记录 hyperfine 结果

---

## Task G：去重与输出增强

- [ ] 跨文件去重策略（同 URL 跨 chunk）
- [ ] 可选 SARIF 输出（若时间够，否则 Phase 4）

---

## 验收命令

```bash
spoor scan tests/fixtures/ --jsonl
hyperfine 'spoor scan large.js' 'jsluice urls large.js'  # 需本地 jsluice
cargo test --workspace
```
