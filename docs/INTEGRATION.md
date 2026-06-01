# Spoor 集成指南

面向工具平台、流水线编排（如 Katana → Spoor → 下游）的完整说明：安装、命令参数、输出格式、批量调用与字段参考。

**版本：** 0.1.0 · **CLI 名称：** `spoor`

---

## 1. 工具定位

```
Katana（爬取 + 抽取 JS）→ Spoor（单文件静态分析）→ 你的平台 / ffuf / 告警
```

Spoor **只处理单个 JavaScript/TypeScript 文件**（或 stdin），输出三类结构化结果：

| kind | 含义 | 典型用途 |
|------|------|----------|
| `path` | 路径、路由、静态资源 | 目录爆破、路由梳理 |
| `endpoint` | 可发起的 HTTP/WebSocket 等 URL | API 清单、流量模拟 |
| `secret` | 密钥、Token、凭证 | 泄漏排查 |

**不做：** 爬站、HTML 解析、目录递归（由 Katana + shell 负责）。

---

## 2. 安装

### 2.1 从源码安装

```bash
git clone https://github.com/P0m32Kun/Spoor.git
cd Spoor
cargo install --path crates/spoor-cli
spoor --version
```

**要求：** Rust 1.93+（推荐 1.94）

### 2.2 验证

```bash
spoor scan tests/fixtures/sample.js
```

---

## 3. 命令与参数

### 3.1 命令总览

| 命令 | 作用 | 输出 kind |
|------|------|-----------|
| `spoor scan <PATH>` | 全量扫描 | path + endpoint + secret |
| `spoor paths <PATH>` | 仅路径 | path |
| `spoor apis <PATH>` | 仅 API 端点 | endpoint |
| `spoor keys <PATH>` | 仅敏感信息 | secret |

### 3.2 全局参数

| 参数 | 短选项 | 适用命令 | 说明 |
|------|--------|----------|------|
| `<PATH>` | — | 全部 | 输入文件路径；**`-` 表示从 stdin 读取** |
| `--output <FILE>` | `-o` | 全部 | 写入文件；省略则输出到 stdout |
| `--jsonl` | `-j` | **仅 `scan`** | 每行一个 `Finding` JSON（见 §4.2） |
| `--help` | `-h` | 全部 | 帮助 |
| `--version` | `-V` | 全部 | 版本 |

### 3.3 使用示例

```bash
# 全量 JSON（pretty-print）
spoor scan ./app.chunk.js

# 写入文件
spoor scan ./app.chunk.js -o result.json

# JSONL（平台集成推荐）
spoor scan ./app.chunk.js --jsonl
spoor scan ./app.chunk.js --jsonl -o findings.jsonl

# 按类型过滤
spoor apis ./app.chunk.js
spoor paths ./app.chunk.js
spoor keys ./app.chunk.js

# stdin 管道
cat ./app.chunk.js | spoor scan -
cat ./app.chunk.js | spoor apis -
```

### 3.4 Katana 批量编排（平台侧实现）

Spoor 不内置目录扫描；在平台或 shell 中循环调用：

```bash
for f in ./katana-output/*.js; do
  spoor scan "$f" --jsonl >> "./spoor-out/$(basename "$f").jsonl"
done
```

---

## 4. 输出格式

### 4.1 标准 JSON（默认）

顶层为 **`ScanResult`** 对象：

```json
{
  "file": "app.chunk.js",
  "findings": [ /* Finding[] */ ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `file` | string | 输入标识：文件路径，或 stdin 时为 `"<stdin>"` |
| `findings` | array | 去重后的 finding 列表 |

**示例（节选）：**

```json
{
  "file": "tests/fixtures/katana/spa_bundle.js",
  "findings": [
    {
      "kind": "endpoint",
      "value": "/api/v2/users?id=1&role=admin",
      "confidence": "high",
      "method": "GET",
      "params": {
        "query": ["id", "role"],
        "body": []
      },
      "origin": {
        "pattern": "fetch",
        "snippet": "fetch(\"/api/v2/users?id=1&role=admin\", { method: \"GET\" });",
        "line": 9,
        "column": 3
      }
    },
    {
      "kind": "path",
      "value": "/app/dashboard",
      "confidence": "high",
      "tags": ["router"],
      "origin": {
        "pattern": "router.path",
        "snippet": "{ path: \"/app/dashboard\", element: null, loader: true }",
        "line": 16,
        "column": 5
      }
    },
    {
      "kind": "secret",
      "value": "AKIAIOSFODNN7EXAMPLE",
      "confidence": "high",
      "secret_type": "aws_access_key",
      "severity": "critical",
      "origin": {
        "pattern": "string_literal",
        "snippet": "const token = \"AKIAIOSFODNN7EXAMPLE\";",
        "line": 7,
        "column": 15
      }
    }
  ]
}
```

### 4.2 JSONL 模式（`scan --jsonl`）

- **每行一个 `Finding` 对象**（不是 `ScanResult`）
- 行末换行 `\n`
- **不含 `file` 字段** — 平台需在调用侧记录来源文件

```jsonl
{"kind":"endpoint","value":"/api/v2/users?id=1&role=admin","confidence":"high","origin":{"pattern":"fetch","snippet":"...","line":9,"column":3},"method":"GET","params":{"query":["id","role"],"body":[]}}
{"kind":"path","value":"/app/dashboard","confidence":"high","origin":{"pattern":"router.path","snippet":"...","line":16,"column":5},"tags":["router"]}
```

### 4.3 Finding 对象 schema

所有 finding 共有字段：

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `kind` | string | ✅ | `"path"` \| `"endpoint"` \| `"secret"` |
| `value` | string | ✅ | 路径、URL 或密钥字符串 |
| `confidence` | string | ✅ | `"high"` \| `"medium"` \| `"low"` |
| `origin` | object | ✅ | 来源上下文（见下表） |
| `method` | string | endpoint | HTTP 方法或 `"WS"`（WebSocket） |
| `params` | object | endpoint | `{ "query": string[], "body": string[] }` |
| `secret_type` | string | secret | 密钥类型标识 |
| `severity` | string | secret | `"critical"` \| `"high"` \| `"medium"` 等 |
| `context` | object | secret | `{ "nearby_keys": string[] }` 可选 |
| `tags` | string[] | 可选 | 如 `["router"]`, `["graphql"]`, `["literal"]` |

**空字段省略：** `method`、`params`、`secret_type`、`severity`、`context`、`tags` 在无值时不出现在 JSON 中。

#### `origin` 对象

| 字段 | 类型 | 说明 |
|------|------|------|
| `pattern` | string | 匹配器标识（见 §5） |
| `snippet` | string | 源码片段（约 80 字符） |
| `line` | number | 1-based 行号 |
| `column` | number | 1-based 列号 |

---

## 5. `origin.pattern` 参考

### 5.1 endpoint

| pattern | 触发场景 |
|---------|----------|
| `fetch` | `fetch(url, { method })` |
| `xhr.open` | `xhr.open(method, url)`（仅 `new XMLHttpRequest()` 绑定变量） |
| `location.href` | `location.href = url` |
| `location.replace` | `location.replace(url)` |
| `location.assign` | `location.assign(url)` |
| `location` | `window.location = url` 等 |
| `axios.get` / `axios.post` / `axios.put` / `axios.delete` / `axios.request` | axios 调用 |
| `ky` / `ky.get` / `ky.post` / … | ky |
| `got` / `got.get` / `got.post` / … | got |
| `superagent.get` / `superagent.post` / … | superagent |
| `superagent.*`（`request.get` 等） | superagent 别名 `request` |
| `jquery.get` / `jquery.post` / `jquery.ajax` | jQuery |
| `window.open` | `window.open(url)` |
| `websocket` | `new WebSocket(url)` |
| `gql.template` | `` gql`...` ``（启发式 `/graphql` endpoint） |
| `graphql.request` | `graphql(url, …)` |

### 5.2 path

| pattern | 触发场景 |
|---------|----------|
| `string_literal` | 字符串字面量中的 URL/路径 |
| `router.path` | react-router / vue-router 路由对象 `{ path: "..." }` |
| `sourceMappingURL` | 字符串中含 `sourceMappingURL=` |

### 5.3 secret

| pattern | 说明 |
|---------|------|
| `string_literal` | 字符串中的 AKIA、AIza、sk-、ghp_ 等 |
| `object_literal` | 对象键 `apiKey` / `token` / `password` 等 |
| `firebase_config` | Firebase 配置对象 |
| `gcp_service_account` | GCP service account JSON |

### 5.4 `secret_type` 枚举

| secret_type | 说明 |
|-------------|------|
| `aws_access_key` | AWS Access Key（AKIA…） |
| `gcp_api_key` | GCP API Key（AIza…） |
| `firebase_api_key` | Firebase 配置中的 apiKey |
| `gcp_service_account_key` | Service account 私钥 |
| `gcp_private_key` | PEM 私钥字符串 |
| `github_token` | `ghp_` / `github_pat_` |
| `api_key` | `sk-` 等通用 API key |
| `object_literal_key` | 敏感对象键上的字符串值 |

---

## 6. 平台集成示例

### 6.1 Python（subprocess + JSONL）

```python
import json
import subprocess
from pathlib import Path

def spoor_scan_file(js_path: Path) -> list[dict]:
    proc = subprocess.run(
        ["spoor", "scan", str(js_path), "--jsonl"],
        capture_output=True,
        text=True,
        check=True,
    )
    findings = []
    for line in proc.stdout.splitlines():
        if line.strip():
            row = json.loads(line)
            row["_source_file"] = str(js_path)  # JSONL 无 file，自行附加
            findings.append(row)
    return findings

# 按 kind 分流
for f in findings:
    if f["kind"] == "endpoint":
        ...
    elif f["kind"] == "secret":
        ...
```

### 6.2 Shell + jq

```bash
# 所有 endpoint URL
spoor scan app.js --jsonl | jq -r 'select(.kind=="endpoint") | .value'

# 所有 critical secret
spoor scan app.js --jsonl | jq 'select(.kind=="secret" and .severity=="critical")'
```

### 6.3 HTTP 微服务封装（建议）

当前 Spoor 为 CLI，平台可：

1. 子进程调用 `spoor scan <file> --jsonl`（最简单）
2. 或通过 stdin：`echo "$js" | spoor scan - --jsonl`

**约定：**

- 成功：exit code `0`，stdout 为 JSON/JSONL
- 失败：非 0（文件不存在、IO 错误等），stderr 可能有 Rust 错误信息

---

## 7. 行为说明（集成时注意）

### 7.1 去重

同一 `value` 只保留一条 finding，优先级：

1. `endpoint` > `path`
2. 同 kind 下 `confidence` 高者优先
3. 有 `method` 的 endpoint 优先

### 7.2 动态 URL

`fetch(base + "/users")` **不会**产出含 `EXPR` 的 endpoint；字面量 `/users` 可能仍作为 `path`。

### 7.3 单文件

每次调用只分析一个文件；批量由调用方循环。

### 7.4 编码

输入按 UTF-8 读取；输出 JSON 为 UTF-8。

---

## 8. 与 jsluice 的关系

Spoor 参考 [jsluice](https://github.com/BishopFox/jsluice) 语义，输出 schema **不同**（三类 `kind` + `origin`）。

可选 parity 测试（需本地安装 jsluice）：

```bash
go install github.com/BishopFox/jsluice/cmd/jsluice@latest
cargo test -p spoor-core jsluice_parity
```

---

## 9. 相关文档

| 文档 | 内容 |
|------|------|
| [README.md](../README.md) | 快速开始 |
| [docs/ROADMAP.md](./ROADMAP.md) | 范围与进度 |
| [docs/README.md](./README.md) | 文档索引 |
| [AGENTS.md](../AGENTS.md) | 项目范围 |
| [AGENTS.md](../AGENTS.md) | 项目范围（勿扩功能） |
| [docs/DEVELOPMENT.md](./DEVELOPMENT.md) | 能力快照 |

---

## 10. 变更记录

| 版本 | 说明 |
|------|------|
| 0.1.0 | 初始 CLI：scan / paths / apis / keys；JSON + JSONL |

如有字段变更，以 `crates/spoor-core/src/finding.rs` 为准。
