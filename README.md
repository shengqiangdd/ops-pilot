# OpsPilot

> AI 驱动的模块化基础设施运维平台 —— 通过自然语言管理服务器、容器和监控。

[![CI/CD](https://github.com/shengqiangdd/ops-pilot/actions/workflows/ci.yml/badge.svg)](https://github.com/shengqiangdd/ops-pilot/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Rust](https://img.shields.io/badge/Rust-1.82+-orange?logo=rust)
![React](https://img.shields.io/badge/React-19-61DAFB?logo=react)
![GitHub Release](https://img.shields.io/github/v/release/shengqiangdd/ops-pilot?logo=github)
[![GHCR](https://img.shields.io/badge/GHCR-ops--pilot-blue?logo=docker)](https://github.com/shengqiangdd/ops-pilot/pkgs/container/ops-pilot)
[![Security Policy](https://img.shields.io/badge/Security-Policy-brightgreen?logo=shield)](SECURITY.md)

## 架构总览

```
┌─────────────────────────────────────────────────────────┐
│                    frontend/                            │
│       React 19 + Vite 6 + Tailwind CSS 3 + ReactFlow  │
│       Zustand state mgmt · React Query · React Router 7│
└──────────────────────────┬──────────────────────────────┘
                           │ REST API + WebSocket
┌──────────────────────────▼──────────────────────────────┐
│                   ops-pilot-gateway                     │
│   Axum 0.8 HTTP server · JWT auth middleware            │
│   ┌────────────┐ ┌──────────┐ ┌────────────────────┐   │
│   │ Agent ReAct│ │ LLM Chat │ │ Terminal WS→SSH    │   │
│   │   Loop     │ │ Service  │ │ Proxy              │   │
│   └─────┬──────┘ └────┬─────┘ └────────┬───────────┘   │
│         └─────────────┼────────────────┘                │
│         ┌─────────────▼────────────────┐                │
│         │      ToolRegistry            │                │
│         │  Route tool calls → modules  │                │
│         └─────────────┬────────────────┘                │
└───────────────────────┼─────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────┐
│                    ops-pilot-sdk                        │
│  OpsModule trait · EventBus · ModuleLoader · Context    │
└───────────────────────┬─────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────┐
│                    ops-pilot-core                       │
│  SSH (russh) · Docker (bollard) · Auth (argon2+JWT)   │
│  Host CRUD · SQLite · Vault · Audit · Alert            │
└─────────────────────────────────────────────────────────┘
```

## 模块

| 模块 | 说明 |
|------|------|
| **mod-core** | 核心运维：SSH 连接、Docker 容器管理、主机 CRUD、系统监控 |
| **mod-rca** | 根因分析：基于规则的诊断 + LLM 驱动的深度分析 |
| **mod-security** | 安全扫描：CIS 合规检查、漏洞检测、补丁管理、LLM 安全报告 |

## API 端点

### 认证 Authentication

| 方法 | 路径 | 说明 |
|--------|------|------|
| POST | `/api/auth/register` | 注册新用户 |
| POST | `/api/auth/login` | 登录并获取 JWT |

### 主机管理 Host Management

| 方法 | 路径 | 说明 |
|--------|------|------|
| GET | `/api/hosts` | 获取当前用户的主机列表 |
| POST | `/api/hosts` | 创建主机 |
| GET | `/api/hosts/:id` | 获取主机详情 |
| PUT | `/api/hosts/:id` | 更新主机 |
| DELETE | `/api/hosts/:id` | 删除主机 |

### 凭据加密 Vault

| 方法 | 路径 | 说明 |
|--------|------|------|
| GET | `/api/vault/status` | 检查 vault 状态 |
| POST | `/api/vault/set-passphrase` | 设置 vault 口令 |
| POST | `/api/vault/unlock` | 解锁 vault |
| POST | `/api/vault/lock` | 锁定 vault |

### 安全扫描 Security Scanning

| 方法 | 路径 | 说明 |
|--------|------|------|
| POST | `/api/security/scan` | 对主机执行 CIS 合规扫描 |
| GET | `/api/security/checks` | 列出所有可用安全检查项 |

### 模块管理 Modules

| 方法 | 路径 | 说明 |
|--------|------|------|
| GET | `/api/modules` | 列出已加载模块 |
| GET | `/api/modules/:name` | 模块详情 |
| POST | `/api/modules/:name/enable` | 启用模块 |
| POST | `/api/modules/:name/disable` | 停用模块 |
| GET | `/api/modules/:name/health` | 模块健康检查 |
| GET | `/api/health` | 全部模块聚合健康状态 |

### AI 对话 Agent

| 方法 | 路径 | 说明 |
|--------|------|------|
| POST | `/api/agent/session` | 创建 Agent 会话 |
| POST | `/api/agent/chat/:session_id` | 发送消息给 Agent |
| DELETE | `/api/agent/session/:session_id` | 关闭会话 |

### 终端 Terminal

| 方法 | 路径 | 说明 |
|--------|------|------|
| GET | `/api/terminal/:host_id` | WebSocket SSH 终端（`?token=` 鉴权） |

## 前端功能

- **Login / Register** — JWT 身份认证
- **Hosts** — 基础设施主机的 CRUD 管理，含 SSH/Docker 详情
- **Vault** — 加密凭据存储，口令保护解锁
- **Security Scan** — CIS 合规检查，按严重程度查看结果，一键修复
- **Health Dashboard** — 实时模块健康监控，状态徽章展示
- **Agent Chat** — AI 对话式基础设施运维
- **Terminal** — 浏览器内 SSH 终端，通过 WebSocket 代理
- **Module Browser** — 查看和切换已加载模块

## 快速开始

### Docker（推荐）

```bash
# 克隆并配置
git clone https://github.com/shengqiangdd/ops-pilot.git
cd ops-pilot
cp .env.example .env
# 编辑 .env 填入你的 LLM_API_KEY 和密钥

# 启动服务（默认使用远程 LLM，无需 Ollama）
docker compose up -d

# 如需使用本地 Ollama：
#   export LLM_PROVIDER=ollama
#   export LLM_BASE_URL=http://ollama:11434/v1
#   docker compose --profile ollama up -d

# 访问 http://localhost:3001
```

### 拉取 GHCR 镜像（Tag 版本）

```bash
docker pull ghcr.io/shengqiangdd/ops-pilot:v0.1.0
docker run -d \
  -p 3001:3001 \
  -v ops_pilot_data:/app/data \
  -e JWT_SECRET=$(openssl rand -hex 32) \
  -e LLM_API_KEY=sk-... \
  ghcr.io/shengqiangdd/ops-pilot:v0.1.0
```

### 本地开发

#### 前置依赖

- Rust 1.82+
- Node.js 20+
- SQLite（通过 `libsqlite3-sys` 捆绑）

#### 后端

```bash
cd backend
cargo build --workspace
cargo test --workspace
cargo run -p ops-pilot-gateway
```

#### 前端

```bash
cd frontend
npm install
npm run dev     # Vite 开发服务器，地址 http://localhost:5173
npm run build   # 生产构建到 dist/
```

## 配置项

| 变量 | 默认值 | 说明 |
|----------|---------|------|
| `LISTEN_ADDR` | `0.0.0.0:3001` | 服务监听地址 |
| `DATABASE_URL` | `sqlite:ops-pilot.db` | 数据库连接字符串 |
| `JWT_SECRET` | （必填） | JWT 签名密钥 |
| `LLM_PROVIDER` | `openai` | LLM 提供商：`openai` / `ollama` / `deepseek` / `mimo` |
| `LLM_BASE_URL` | 取决于提供商 | LLM API 基础地址 |
| `LLM_API_KEY` | — | LLM API 密钥 |
| `LLM_MODEL` | `gpt-4o` | 对话模型名称 |
| `STATIC_DIR` | `./static` | 前端静态文件目录 |
| `OPSPILOT_MASTER_KEY` | （推荐） | 主机凭据加密主密钥 |

## 安全特性

- 密码使用 **Argon2id** 哈希
- JWT 令牌 24 小时过期
- 所有受保护路由需 `Authorization: Bearer <token>`
- 主机 API 强制用户隔离
- **Vault** 每个用户独立的口令派生 AES-256 加密
- 主机凭据使用 AES-256-GCM 加密
- SSH 支持密码和公钥认证
- SSH 主机密钥验证（`known_hosts`）防止中间人攻击
- **速率限制**：登录端点 5 次/分钟/IP
- **审计日志**：SSH 连接/断开事件自动记录
- **告警引擎**：非工作时间批量操作、高失败率、首次连接检测
- **CIS 合规扫描**：内置 24 条基准规则

## API 文档

完整的 API 文档使用 OpenAPI 3.0 规范编写，可使用 Swagger UI 查看：

```bash
# 直接在浏览器打开
open docs/swagger.html

# 或启动本地服务查看
cd docs && python3 -m http.server 8080
# 访问 http://localhost:8080/swagger.html
```

- **OpenAPI 规范**: [`docs/openapi.yaml`](docs/openapi.yaml)
- **Swagger UI**: [`docs/swagger.html`](docs/swagger.html)

### API 端点概览

| 模块 | 端点数 | 说明 |
|------|--------|------|
| Auth | 2 | 登录、注册 |
| Hosts | 5 | 主机管理 + 批量执行 |
| Users | 5 | 用户管理（RBAC） |
| Vault | 4 | 凭据加密存储 |
| Modules | 5 | 模块管理 |
| Health | 2 | 健康检查 |
| Security | 2 | 安全扫描 |
| FIM | 2 | 文件完整性监控 |
| Baseline | 2 | 安全基线 |
| Topology | 2 | 网络拓扑 |
| Monitor | 2 | 性能监控 |
| Escalation | 2 | 告警升级 |
| Audit | 3 | 审计日志 + 导出 |
| Alert Rules | 3 | 告警规则 CRUD |
| Alert History | 1 | 告警历史 |
| Notification Channels | 3 | 通知渠道管理 |
| CMDB | 8 | 配置管理数据库 |
| Agent | 5 | AI 对话 + NL 查询 + 诊断 |
| Knowledge | 2 | 知识库 |
| Runbook | 2 | 运维手册 |
| Terminal | 1 | WebSSH 终端 |
| **总计** | **62** | |

## 部署

详见 [DEPLOY.md](DEPLOY.md) —— Docker 部署、TLS 配置、安全检查清单。

## 许可证

Apache License 2.0 —— 详见 [LICENSE](LICENSE)。

---

> **English version:** This README is primarily in Chinese. All code examples, API paths, configuration keys, and tables remain in English for practical reference. For the original English version, refer to the [project history](https://github.com/shengqiangdd/ops-pilot/commits/main/README.md).
