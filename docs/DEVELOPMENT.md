# Spoor 整体开发进度与功能规划

**仓库：** https://github.com/P0m32Kun/Spoor  
**最后更新：** 2026-05-31 · **版本：** 0.1.0 · **测试：** 35 passed

---

## 1. 项目是什么

Spoor 是工具链中的**单环**：消费 **Katana（等）提取的 JS 文件**，静态产出三类信息：

| 产出 | `kind` | 用途 |
|------|--------|------|
| 路径提取 | `path` | ffuf、目录爆破、路由梳理 |
| API 提取 | `endpoint` | 接口清单（含 method） |
| 敏感信息提取 | `secret` | AKIA、GCP、token 等 |

```
Katana → Spoor（本工具）→ 下游
```

**不做：** 爬站、HTML 抽取、目录递归扫描、YAML 规则平台 — 见 [plan1.md §1.3](../plan1.md)。

**解析引擎：** [Oxc](https://oxc.rs/) · **对标参考：** [jsluice](https://github.com/BishopFox/jsluice)

---

## 2. 整体进度

```
Phase 0  基础解析 + 字面量路径     ████████████████████ 100%  ✅ 已签 off
Phase 1  语义 endpoint matcher      ████████████████████ 100%  ✅ 已签 off
Phase 2  secret + JS 内 URL 增强   ████████████████████ 100%  ✅ 已签 off
维护     质量 / 可选 backlog        ░░░░░░░░░░░░░░░░░░░░  按需
```

| 阶段 | 状态 | 文档 |
|------|------|------|
| Phase 0 | ✅ | [retro](./superpowers/plans/2026-05-31-spoor-phase-0-retro.md) |
| Phase 1 | ✅ | [retro](./superpowers/plans/2026-05-31-spoor-phase-1-retro.md) |
| Phase 2 | ✅ | [retro](./superpowers/plans/2026-05-31-spoor-phase-2-retro.md) · [plan3](../plan3.md) |
| ~~Phase 3/4~~ | 🚫 不实施 | [plan4](../plan4.md) · [plan5](../plan5.md) 归档 |

---

## 3. 功能矩阵

### 3.1 CLI（单文件 in）

| 命令 | 作用 | 状态 |
|------|------|------|
| `spoor scan <file>` | path + endpoint + secret | ✅ |
| `spoor paths <file>` | 仅 path | ✅ |
| `spoor apis <file>` | 仅 endpoint | ✅ |
| `spoor keys <file>` | 仅 secret | ✅ |
| `-` / `-o` / `--jsonl` | 管道与输出 | ✅ |

**Katana 批量：**

```bash
for f in ./katana-js/*.js; do spoor scan "$f" --jsonl >> findings.jsonl; done
```

### 3.2 Endpoint Matcher

fetch · location · xhr · axios · ky · got · superagent · jQuery · window.open · WebSocket · GraphQL

### 3.3 Path Matcher

literal · router.path · sourceMappingURL

### 3.4 Secret Matcher

AKIA · AIza / GCP · Firebase · service_account · sk- · ghp_ · 对象键启发

---

## 4. 维护 backlog（在范围内）

| 项 | 说明 |
|----|------|
| 统一 `resolved_maybe_url` | 减误报 |
| env 字符串 secret | 可选 |
| Katana bundle 回归 fixture | 可选 |
| CI | test + fmt + clippy |

---

## 5. 已知限制

1. **单文件** — 设计如此；批量交给 Katana + shell。
2. **动态 URL** — `fetch(a + "/b")` 无可靠 endpoint。
3. **非 JS** — 不支持 HTML；Katana 先抽 JS。

---

## 6. 文档索引

| 文档 | 用途 |
|------|------|
| [plan1.md](../plan1.md) | 范围锁定 + 总计划 |
| [plan2.md](../plan2.md) | JSON 模型 |
| [plan3.md](../plan3.md) | Phase 2 清单（已完成） |
| [Phase 2 retro](./superpowers/plans/2026-05-31-spoor-phase-2-retro.md) | 签 off 证据 |
| [README.md](../README.md) | 安装与用法 |

---

*维护说明：Spoor 仅维护 JS 三类提取能力；不扩展 plan4/plan5 平台功能。*
