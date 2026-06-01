# Spoor 命名与 JSON 输出模型

> **总体计划：** [plan1.md](./plan1.md) · **进度快照：** [docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md)  
> **已定名：** Spoor · **仓库：** https://github.com/P0m32Kun/Spoor

以下为命名决策记录与 **plan2 JSON 模型**（当前 CLI 输出格式）。

---

## 命名原则

- **短、好敲**：CLI 最好是 4–6 个字母，管道里好用  
- **语义贴场景**：收参、挖路径、找接口、扫密钥，而不是泛泛的 “js 解析”  
- **Rust 友好**：crate 名用 `kebab-case`，二进制一个主命令  
- **避免**：`js-*` 太泛、`recon-*` 太像通用框架、和现有知名工具撞名（如 `truffle`、`harvest` 等需先搜一下）

---

## 推荐（按优先级）

### 1. **Spoor**（首推）

| 项 | 内容 |
|----|------|
| 含义 | 踪迹、足迹（动物/目标留下的痕迹） |
| 隐喻 | 在 JS 资产里找「路径足迹、API 调用痕迹、密钥泄漏痕迹」 |
| CLI | `spoor scan ./dist` |
| Crate | `spoor`（lib）、`spoor-cli`（bin） |
| 仓库 | `spoor-rs` 或 `spoor` |

红队语境自然，不局限 JS，以后加 HTML/CSS map 也不违和。

---

### 2. **Vein**

| 项 | 内容 |
|----|------|
| 含义 | 矿脉 |
| 隐喻 | 从打包后的 JS 里「开矿」：路径矿脉、接口矿脉、密钥矿脉 |
| CLI | `vein ./app.js` |
| Crate | `vein-core` + `vein` |

偏技术感，适合强调「从 bundle 里挖东西」。

---

### 3. **Chaff**

| 项 | 内容 |
|----|------|
| 含义 | 糠秕 / 干扰物（相对「真情报」） |
| 隐喻 | 从大量噪音字符串里筛出真路径、真 API、真 Key |
| CLI | `chaff scan` |

略冷僻，但安全圈里 “signal vs noise” 很好记。

---

### 4. **Sift**

| 项 | 内容 |
|----|------|
| 含义 | 筛选 |
| 隐喻 | 直白：筛路径、筛接口、筛密钥 |
| CLI | `sift` |

最朴素，好懂，但辨识度略低（搜 crate 可能重名多）。

---

## 备选（可作副项目名 / 子命令）

| 名称 | 特点 |
|------|------|
| **Rift** | 裂缝 → 信息泄漏 |
| **Trail** | 路径、路由痕迹 |
| **Dredge** | 清淤式挖掘 minified JS |
| **Scry** | 占卜式「看穿」混淆代码（略玄学） |

---

## 我的建议

**主选：`Spoor`**  
- 和红队「跟痕迹」一致  
- 覆盖路径 + API + Key 三类产出  
- CLI 短、好记：`spoor scan`、`spoor paths`、`spoor keys`  

**次选：`Vein`** — 若你更想强调「从 JS bundle 挖矿」而不是「跟踪踪迹」。

当前目录 `js-rs` 可在定名后整体重命名为 `spoor` 或 `spoor-rs`。

---

## 与命名配套的产出模型（全新 JSON）

不按 jsluice，按你的三类能力设计：

```json
{
  "file": "dist/app.abc123.js",
  "findings": [
    {
      "kind": "path",
      "value": "/api/v2/users/{id}",
      "confidence": "high",
      "origin": {
        "pattern": "fetch",
        "snippet": "fetch(`/api/v2/users/${id}`)",
        "line": 1024,
        "column": 12
      },
      "tags": ["relative", "rest"]
    },
    {
      "kind": "endpoint",
      "value": "https://api.example.com/v2/auth/login",
      "method": "POST",
      "params": { "query": [], "body": ["email", "password"] },
      "confidence": "high",
      "origin": { "pattern": "axios.post", "snippet": "..." }
    },
    {
      "kind": "secret",
      "value": "AKIA****************",
      "secret_type": "aws_access_key",
      "severity": "critical",
      "context": { "nearby_keys": ["region", "bucket"] },
      "origin": { "pattern": "object_literal", "snippet": "..." }
    }
  ]
}
```

**三类 `kind` 分工：**

| kind | 用途 | 示例 |
|------|------|------|
| `path` | 站内路径、路由、静态资源路径 | `/admin`, `/api/v1/...` |
| `endpoint` | 可发起的 API（含 method、参数名） | `POST /graphql`, `GET https://...` |
| `secret` | 泄漏的 Key/Token/凭证 | AWS、JWT、sk-、自定义 API Key |

**CLI 子命令建议（Spoor 为例）：**

```bash
spoor scan ./dist/          # 三类全扫
spoor paths ./dist/         # 仅路径
spoor apis ./dist/          # 仅接口
spoor keys ./dist/          # 仅密钥
spoor scan -o out.jsonl     # 一行一个 finding，便于 jq | ffuf
```

---

## 需要你拍板的两件事

1. **最终名字**：`Spoor` / `Vein` / 其他（或你给偏好：中文谐音 / 更低调 / 更攻击性）  
2. **仓库与二进制**：例如 GitHub `yourname/spoor`，安装后命令 `spoor`

你选定名字后，我可以按 **Spoor + 上述 JSON 模型** 从 Phase 0 开始改 workspace（`Cargo.toml`、crate 划分、类型定义），不再使用 `js-rs` 和 jsluice 字段名。你更倾向哪一个？
