# ~~Spoor Phase 3~~ — 归档（不实施）

> **状态：** 🚫 **OUT OF SCOPE** — 2026-05-31 范围锁定  
> **替代方案：** 见 [plan1.md §1.3](./plan1.md)

---

## 为何取消

Spoor 定位为 **Katana 下游的单文件 JS 分析器**，只做 path / endpoint / secret 提取。下列 Phase 3 目标与 Katana 职责重叠或偏离核心：

| 原 Phase 3 项 | 不做的原因 |
|---------------|------------|
| 目录递归 + rayon | Katana 产出 JS 列表；用 shell/`xargs` 调 Spoor |
| HTML `<script>` 抽取 | Katana 已抽 JS |
| YAML 规则引擎 | 非 JS AST 语义分析范围 |
| `--no-literals` 等平台 DX | 可选 backlog，非独立阶段 |

---

## 原草案（仅供历史参考）

<details>
<summary>展开原 Phase 3 任务列表</summary>

- Task A：目录与并行扫描
- Task B：Matcher trait + 注册表
- Task C：YAML 规则
- Task D：HTML 输入
- Task E：CLI 降噪
- Task F：CI / hyperfine
- Task G：跨文件去重 / SARIF

</details>

**若在范围内的小改进：** 见 [plan1.md §六 维护 backlog](./plan1.md)。
