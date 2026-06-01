# Spoor 集成指南

面向工具平台、流水线编排（如 Katana → Spoor → 下游）的完整说明：安装、命令参数、输出格式、批量调用与字段参考。

**版本：** 0.2.0 · **CLI 名称：** `spoor`

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

### 2.1 预编译二进制（Docker / 平台集成推荐）

发布页：[GitHub Releases](https://github.com/P0m32Kun/Spoor/releases)

| 环境 | 资产 |
|------|------|
| glibc Linux x86_64（Debian/Ubuntu 等） | `spoor-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `spoor-aarch64-unknown-linux-gnu.tar.gz` |
| Alpine / musl x86_64 | `spoor-x86_64-unknown-linux-musl.tar.gz` |

```bash
VERSION=0.2.0
ARCH=x86_64-unknown-linux-gnu   # 或 aarch64-unknown-linux-gnu / x86_64-unknown-linux-musl

curl -fsSL -o /tmp/spoor.tgz \
  "https://github.com/P0m32Kun/Spoor/releases/download/v${VERSION}/spoor-${ARCH}.tar.gz"
curl -fsSL -o /tmp/spoor.tgz.sha256 \
  "https://github.com/P0m32Kun/Spoor/releases/download/v${VERSION}/spoor-${ARCH}.tar.gz.sha256"
sha256sum -c /tmp/spoor.tgz.sha256

tar xzf /tmp/spoor.tgz -C /usr/local/bin
spoor --version
```

**Dockerfile 示例（多阶段，仅拷贝二进制）：**

```dockerfile
FROM debian:bookworm-slim AS spoor
ARG SPOOR_VERSION=0.2.0
RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates \
    && curl -fsSL -o /tmp/spoor.tgz \
       "https://github.com/P0m32Kun/Spoor/releases/download/v${SPOOR_VERSION}/spoor-x86_64-unknown-linux-gnu.tar.gz" \
    && tar xzf /tmp/spoor.tgz -C /usr/local/bin \
    && rm /tmp/spoor.tgz \
    && apt-get purge -y curl && apt-get autoremove -y && rm -rf /var/lib/apt/lists/*

FROM your-platform-image
COPY --from=spoor /usr/local/bin/spoor /usr/local/bin/spoor
```

Alpine 镜像请将 URL 中的资产改为 `spoor-x86_64-unknown-linux-musl.tar.gz`。

### 2.2 从源码安装

```bash
git clone https://github.com/P0m32Kun/Spoor.git
cd Spoor
cargo install --path crates/spoor-cli
spoor --version
```

**要求：** Rust 1.93+（推荐 1.94）

### 2.3 验证

```bash
spoor scan tests/fixtures/sample.js
```

---

## 3. 命令与参数

### 3.1 命令总览

| 命令 | 作用 | 输入 |
|------|------|------|
| `spoor scan <TARGET>` | 全量扫描 | **JS URL**、URL 列表文件、或 `-`（stdin URL 列表） |
| `spoor paths <PATH>` | 仅路径 | 本地 JS 文件 |
| `spoor apis <PATH>` | 仅 API 端点 | 本地 JS 文件 |
| `spoor keys <PATH>` | 仅敏感信息 | 本地 JS 文件 |

### 3.2 `spoor scan` 目标（`TARGET`）

| 形式 | 示例 | 模式 |
|------|------|------|
| 单个 URL | `http://192.168.1.8:18080/1.js` | **JS**：path + endpoint + secret |
| 单个 URL | `http://192.168.1.8:18080/admin` | **页面**：**仅 secret**（不与 Katana 抢 endpoint） |
| URL 列表文件 | `./katana-urls.txt` | 按 URL 扩展名自动分流 |

列表文件示例：

```text
# Katana 混合输出 — JS 与 HTML 可放在同一列表
http://192.168.1.8:18080/1.js
http://192.168.1.8:18080/admin
http://192.168.1.8:18080/static/app.chunk.js
```

**分流规则（按 URL 路径扩展名）：**

| 扩展名 | 模式 | 产出 |
|--------|------|------|
| `.js` `.mjs` `.cjs` `.ts` `.tsx` `.jsx` `.vue` `.map` | JS 资产 | path + endpoint + secret |
| 其他（HTML、API 页面等） | 页面 | **secret only** |

页面模式：扫描 HTML 正文 + 内联 `<script>` 中的密钥，**不**提取 endpoint/path，避免与 Katana 功能重叠。

### 3.3 全局参数

| 参数 | 短选项 | 适用 | 说明 |
|------|--------|------|------|
| `<TARGET>` / `<PATH>` | — | 见上 | scan 用 URL；paths/apis/keys 用本地路径 |
| `--output <FILE>` | `-o` | 全部 | 写入文件 |
| `--no-verify` | — | 全部 | 跳过对发现 URL 的 HTTP 探测 |
| `--from-url <URL>` | — | paths/apis/keys | 本地文件扫描时指定 JS 来源 URL |
| `--jsonl` | `-j` | **scan** | 每行一个 Finding（含 `file`=JS URL） |
| `--help` | `-h` | 全部 | 帮助 |

### 3.4 使用示例

```bash
# 单个 JS URL（推荐）
spoor scan "http://192.168.1.8:18080/1.js" --jsonl

# URL 列表批量
spoor scan katana-urls.txt --jsonl -o findings.jsonl

# 管道
cat katana-urls.txt | spoor scan - --jsonl

# 离线：只拼接、不探测
spoor scan "http://192.168.1.8:18080/1.js" --no-verify --jsonl

# 本地文件（paths/apis/keys 或 legacy scan 本地 JS）
spoor paths ./app.js --from-url "http://192.168.1.8:18080/app.js"
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
| `file` | string | 输入 JS 的绝对路径（stdin 时为 `"<stdin>"`） |
| `findings` | array | 去重后的 finding 列表 |

**示例（节选）：**

```json
{
  "file": "tests/fixtures/katana/spa_bundle.js",
  "findings": [
    {
      "kind": "endpoint",
      "value": "https://target.example.com/api/v2/users?id=1&role=admin",
      "raw": "/api/v2/users?id=1&role=admin",
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
- **每行含 `file` 字段** — 分析来源 JS 的绝对路径（stdin 时为 `"<stdin>"`）
- **`endpoint.value` 默认为完整 URL**（自动从同文件绝对 URL / 路径推断 origin，见 §4.4）

```jsonl
{"file":"/abs/path/app.chunk.js","kind":"endpoint","value":"https://target.example.com/api/v2/users?id=1&role=admin","raw":"/api/v2/users?id=1&role=admin","confidence":"high","method":"GET","params":{"query":["id","role"],"body":[]},"origin":{"pattern":"fetch","snippet":"...","line":9,"column":3}}
{"file":"/abs/path/app.chunk.js","kind":"secret","value":"AKIAIOSFODNN7EXAMPLE","confidence":"high","secret_type":"aws_access_key","severity":"critical","origin":{"pattern":"string_literal","snippet":"...","line":7,"column":15}}
```

### 4.3 Finding 对象 schema

所有 finding 共有字段：

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `file` | string | JSONL | 来源 JS 绝对路径（仅 `--jsonl` 时每行都有） |
| `kind` | string | ✅ | `"path"` \| `"endpoint"` \| `"secret"` |
| `value` | string | ✅ | **endpoint：** 完整 URL；**secret：** 密钥内容；**path：** 路径 |
| `raw` | string | endpoint | 源码中的原始字符串（相对路径），仅在与 `value` 不同时出现 |
| `confidence` | string | ✅ | `"high"` \| `"medium"` \| `"low"` |
| `origin` | object | ✅ | 来源上下文（见下表） |
| `method` | string | endpoint | HTTP 方法或 `"WS"`（WebSocket） |
| `params` | object | endpoint | `{ "query": string[], "body": string[] }` |
| `secret_type` | string | secret | 密钥类型标识 |
| `severity` | string | secret | `"critical"` \| `"high"` \| `"medium"` 等 |
| `context` | object | secret | `{ "nearby_keys": string[] }` 可选 |
| `http_status` | number | path/endpoint | HTTP 探测状态码（`--from-url` 且未 `--no-verify`） |
| `tags` | string[] | 可选 | 如 `["router"]`, `["graphql"]`, `["literal"]` |

**空字段省略：** `file`、`raw`、`method`、`params`、`secret_type`、`severity`、`context`、`http_status`、`tags` 在无值时不出现在 JSON 中。

### 4.4 来源 URL 拼接与 HTTP 探测

`spoor scan <JS_URL>` 时，**目标 URL 即来源 URL**：相对 path `/api/admin` 会拼接为 `http://192.168.1.8:18080/api/admin`（与 JS 同 origin）。

**流程：**

1. GET 拉取 JS 源码
2. 静态分析
3. 相对 path/endpoint 按 JS URL 拼接为完整 HTTP(S) URL
4. 对每个完整 URL 探测（HEAD → GET）
5. 仅输出探测成功的 path/endpoint；**secret 始终输出**

**保留状态码：** 2xx、301、302、307、308、401、403、405

输出 `file` 字段 = JS 的 URL（非本地路径）。`--no-verify` 跳过步骤 4。

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

def spoor_scan_url(js_url: str) -> list[dict]:
    proc = subprocess.run(
        ["spoor", "scan", js_url, "--jsonl"],
        capture_output=True,
        text=True,
        check=True,
    )
    findings = []
    for line in proc.stdout.splitlines():
        if line.strip():
            findings.append(json.loads(line))
    return findings

# 按 kind 分流 — 每行已含 file / value
for f in findings:
    if f["kind"] == "endpoint":
        url = f["value"]          # 完整 URL
        source = f["file"]        # JS 绝对路径
    elif f["kind"] == "secret":
        secret = f["value"]       # 密钥内容
        source = f["file"]
        kind = f["secret_type"]
```

### 6.2 Shell + jq

```bash
# 所有 endpoint 完整 URL
spoor scan app.js --jsonl | jq -r 'select(.kind=="endpoint") | .value'

# 所有 secret：文件 + 内容
spoor scan app.js --jsonl \
  | jq 'select(.kind=="secret") | {file, secret: .value, type: .secret_type}'
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
| [AGENTS.md](../AGENTS.md) | 项目范围（勿扩功能） |

---

## 10. 变更记录

| 版本 | 说明 |
|------|------|
| 0.1.0 | 初始 CLI：scan / paths / apis / keys；JSON + JSONL |
| 0.1.3 | 非 JS URL 自动走页面模式（secret only）；Katana 混合 URL 列表 |
| 0.2.0 | `spoor scan` URL 列表 + HTTP 拉取/探测；GitHub Releases 预编译二进制 |

如有字段变更，以 `crates/spoor-core/src/finding.rs` 为准。
