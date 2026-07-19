# OpsPilot

> AI 驱动的基础设施运维助手 — 通过自然语言管理服务器、容器和监控。

## 架构概览

```
┌─────────────────────────────────────────────────────────┐
│                    frontend/                            │
│       React 19 + Vite 6 + Tailwind CSS 3 + ReactFlow  │
│       Zustand 状态管理 · React Query · React Router 7  │
└──────────────────────────┬──────────────────────────────┘
                           │ REST API + WebSocket
┌──────────────────────────▼──────────────────────────────┐
│                   ops-pilot-gateway                     │
│   Axum 0.8 HTTP 服务 · JWT 认证中间件                   │
│   ┌────────────┐ ┌──────────┐ ┌────────────────────┐   │
│   │ Agent ReAct│ │ LLM Chat │ │ Terminal WS→SSH    │   │
│   │   Loop     │ │ Service  │ │ Proxy              │   │
│   └─────┬──────┘ └────┬─────┘ └────────┬───────────┘   │
│         └─────────────┼────────────────┘                │
│         ┌─────────────▼────────────────┐                │
│         │      ToolRegistry            │                │
│         │  路由 tool call → module      │                │
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

## 模块说明

| Crate | 说明 |
|-------|------|
| `ops-pilot-core` | 核心层：SSH 连接管理、Docker 容器操作、Host CRUD、用户认证、Vault 密钥管理、审计日志、告警引擎、SQLite 数据库、事件总线 |
| `ops-pilot-sdk` | SDK 层：`OpsModule` trait 定义、模块加载器、事件系统、模块上下文、工具定义 |
| `ops-pilot-gateway` | 网关层：Axum HTTP API、Agent ReAct 循环、LLM 集成、WebSocket 终端、工具路由、Vault 路由 |
| `frontend/` | 前端应用：React SPA，包含主机管理、AI 对话、Vault 管理、模块浏览 |

## 快速开始

### 前置条件

- Rust 1.82+
- Node.js 20+
- SQLite（通过 `libsqlite3-sys` 捆绑编译）

### 后端

```bash
# 构建所有 crate
cargo build --workspace

# 运行测试
cargo test --workspace

# 启动网关服务
cargo run -p ops-pilot-gateway
```

### 前端

```bash
cd frontend
npm install
npm run dev     # Vite 开发服务器 http://localhost:5173
npm run build   # 生产构建输出到 dist/
```

## API 端点

### 认证

| Method | Path | 说明 |
|--------|------|------|
| POST | `/api/auth/register` | 用户注册 |
| POST | `/api/auth/login` | 登录获取 JWT |

### 主机管理

| Method | Path | 说明 |
|--------|------|------|
| GET | `/api/hosts` | 列出当前用户的主机 |
| POST | `/api/hosts` | 创建主机 |
| GET | `/api/hosts/:id` | 获取主机详情 |
| PUT | `/api/hosts/:id` | 更新主机 |
| DELETE | `/api/hosts/:id` | 删除主机 |

### Vault（凭据加密）

| Method | Path | 说明 |
|--------|------|------|
| GET | `/api/vault/status` | 查看 vault 状态 |
| POST | `/api/vault/set-passphrase` | 设置 vault passphrase |
| POST | `/api/vault/unlock` | 解锁 vault |
| POST | `/api/vault/lock` | 锁定 vault |

### 模块

| Method | Path | 说明 |
|--------|------|------|
| GET | `/api/modules` | 列出已加载模块 |
| GET | `/api/modules/:name` | 模块详情 |
| POST | `/api/modules/:name/enable` | 启用模块 |
| POST | `/api/modules/:name/disable` | 禁用模块 |
| GET | `/api/modules/:name/health` | 模块健康检查 |

### Agent

| Method | Path | 说明 |
|--------|------|------|
| POST | `/api/agent/session` | 创建 Agent 会话 |
| POST | `/api/agent/chat/:session_id` | 向 Agent 发送消息 |
| DELETE | `/api/agent/session/:session_id` | 关闭会话 |

### 终端

| Method | Path | 说明 |
|--------|------|------|
| GET | `/api/terminal/:host_id` | WebSocket SSH 终端（`?token=` 认证） |

## 配置

通过环境变量配置网关：

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `DATABASE_URL` | `sqlite:ops-pilot.db` | SQLite 连接串 |
| `JWT_SECRET` | （必填） | JWT 签名密钥 |
| `OPSPILOT_MASTER_KEY` | （推荐） | 主机凭证加密主密钥（Base64 编码的 32 字节密钥） |
| `LLM_PROVIDER` | `openai` | LLM 提供商：`openai` / `ollama` / `deepseek` |
| `LLM_API_KEY` | — | LLM API 密钥 |
| `LLM_MODEL` | `gpt-4o` | 聊天模型名称 |
| `LISTEN_ADDR` | `0.0.0.0:3001` | 监听地址 |

## 安全特性

- 密码使用 **Argon2id** 哈希
- JWT Token 有效期 24 小时
- 所有受保护路由需要 `Authorization: Bearer <token>`
- Hosts API 强制用户隔离（每用户只看到自己的主机）
- **Vault** 独立加密密钥：每个用户有自己的 passphrase 派生 AES-256 密钥
- 主机凭证 AES-256-GCM 加密存储
- SSH 连接支持密码和公钥认证
- SSH 主机密钥验证（`known_hosts`）— 防止中间人攻击
- **Rate limiting**：登录端点 5 次/分钟/IP
- **审计日志**：SSH 连接/断开事件自动记录
- **告警引擎**：夜间批量操作、高失败率、首次连接检测

## 部署

详见 [DEPLOY.md](DEPLOY.md) — 包含 Docker 部署、TLS 配置、安全检查清单。

## License

Apache License 2.0 — 详见 [LICENSE](LICENSE)。
