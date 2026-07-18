# OpsPilot

> AI 驱动的基础设施运维助手 — 通过自然语言管理服务器、容器和监控。

## 架构概览

```
┌─────────────────────────────────────────────────────────┐
│                    ops-pilot-app                        │
│       React 18 + Vite 6 + Tailwind CSS 3 + ReactFlow  │
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
│  Host CRUD · SQLite · CommandExecutor · EventBus       │
└─────────────────────────────────────────────────────────┘
```

## 模块说明

| Crate | 说明 |
|-------|------|
| `ops-pilot-core` | 核心层：SSH 连接管理、Docker 容器操作、Host CRUD、用户认证、SQLite 数据库、事件总线 |
| `ops-pilot-sdk` | SDK 层：`OpsModule` trait 定义、模块加载器、事件系统、模块上下文、工具定义 |
| `ops-pilot-gateway` | 网关层：Axum HTTP API、Agent ReAct 循环、LLM 集成、WebSocket 终端、工具路由 |
| `ops-pilot-app` | 前端应用：React SPA，包含主机管理、AI 对话、工作流编辑器、用户认证 |

## 模块系统

每个运维模块实现 `OpsModule` trait，声明自己的工具（tool），由 Gateway 的 `ToolRegistry` 自动聚合并路由 AI 的工具调用：

```rust
#[async_trait]
pub trait OpsModule: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    fn dependencies(&self) -> Vec<&str>;
    fn tools(&self) -> Vec<ToolDefinition>;
    async fn execute(&self, ctx: &ModuleContext, tool: &str, params: Value) -> Result<Value>;
    async fn on_event(&self, ctx: &ModuleContext, event: &OpsEvent) -> Option<ModuleAction>;
    async fn health_check(&self, ctx: &ModuleContext) -> HealthStatus;
}
```

模块工具通过 JSON Schema 描述参数，Agent 在 ReAct 循环中自动发现并调用。

## 快速开始

### 前置条件

- Rust 1.85+（2024 Edition）
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
cd ops-pilot-app
npm install
npm run dev     # Vite 开发服务器 http://localhost:5173
npm run build   # 生产构建输出到 dist/
```

## API 端点

| Method | Path | 说明 |
|--------|------|------|
| POST | `/api/auth/register` | 用户注册 |
| POST | `/api/auth/login` | 登录获取 JWT |
| GET | `/api/hosts` | 列出所有主机 |
| POST | `/api/hosts` | 创建主机 |
| PUT | `/api/hosts/:id` | 更新主机 |
| DELETE | `/api/hosts/:id` | 删除主机 |
| GET | `/api/modules` | 列出已加载模块 |
| GET | `/api/modules/:name` | 模块详情 |
| POST | `/api/modules/:name/enable` | 启用模块 |
| POST | `/api/modules/:name/disable` | 禁用模块 |
| GET | `/api/modules/:name/health` | 模块健康检查 |
| POST | `/api/agent/session` | 创建 Agent 会话 |
| POST | `/api/agent/chat/:session_id` | 向 Agent 发送消息 |
| DELETE | `/api/agent/session/:session_id` | 关闭会话 |
| GET | `/ws/terminal/:host_id` | WebSocket SSH 终端 |

## 配置

通过环境变量配置网关：

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `DATABASE_URL` | `sqlite:ops_pilot.db` | SQLite 连接串 |
| `JWT_SECRET` | （必填） | JWT 签名密钥 |
| `LLM_PROVIDER` | `openai` | LLM 提供商：`openai` / `ollama` / `mimo` |
| `LLM_API_KEY` | （openai 必填） | LLM API 密钥 |
| `LLM_MODEL` | `gpt-4` | 聊天模型名称 |
| `HOST` | `0.0.0.0` | 监听地址 |
| `PORT` | `8080` | 监听端口 |
| `OPSPILOT_MASTER_KEY` | （推荐） | 主机凭证加密主密钥（Base64 编码的 32 字节密钥） |

## 安全

- 密码使用 **Argon2id** 哈希
- JWT Token 有效期 24 小时
- 所有受保护路由需要 `Authorization: Bearer <token>`
- SSH 连接支持密码和公钥认证
- SSH 主机密钥验证（`known_hosts`）— 防止中间人攻击
- 主机凭证 AES-256-GCM 加密存储（需设置 `OPSPILOT_MASTER_KEY` 环境变量）
- `russh` 升级至 0.62.x，修复 OOM DoS 和用户名状态绕过漏洞

## License

Apache License 2.0 — 详见 [LICENSE](LICENSE)。
