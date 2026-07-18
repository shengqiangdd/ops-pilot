# OpsPilot — Architecture Deep Dive

本文档详细描述 OpsPilot 各组件的设计决策、数据流和扩展点。

## 1. 分层架构

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 4: ops-pilot-app (React SPA)                         │
│   职责：UI 渲染、用户交互、状态管理、API 调用               │
│   依赖：ops-pilot-gateway REST/WebSocket API               │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: ops-pilot-gateway (Axum)                          │
│   职责：HTTP 路由、认证中间件、Agent 编排、工具路由          │
│   依赖：ops-pilot-sdk、ops-pilot-core                      │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: ops-pilot-sdk (Library)                           │
│   职责：模块抽象、事件系统、工具定义、上下文传递             │
│   依赖：无内部 crate 依赖（纯 trait + 数据类型）            │
├─────────────────────────────────────────────────────────────┤
│ Layer 1: ops-pilot-core (Library)                          │
│   职责：基础设施实现（SSH、Docker、DB、Auth、EventBus）     │
│   依赖：ops-pilot-sdk                                      │
└─────────────────────────────────────────────────────────────┘
```

**依赖方向**：`gateway → sdk ← core`（SDK 是最底层的共享抽象层）

## 2. Agent ReAct Loop

```
User Message
    │
    ▼
┌─────────────────────┐
│  Append to messages  │
│  + Truncate if need  │
└─────────┬───────────┘
          ▼
┌─────────────────────┐
│  LLM.complete()     │◄──────────────┐
│  with tools          │               │
└─────────┬───────────┘               │
          ▼                           │
    ┌─────────┐                       │
    │ Has     │── No ──► Return text   │
    │ tool    │                       │
    │ calls?  │                       │
    └────┬────┘                       │
         │ Yes                        │
         ▼                            │
┌─────────────────────┐               │
│  For each tool_call: │               │
│  registry.invoke()   │               │
│  → module.execute()  │               │
│  append tool result  │               │
└─────────┬───────────┘               │
          ▼                           │
    ┌─────────┐                       │
    │ Turn    │── >= max ─► Force text │
    │ count   │            response   │
    └────┬────┘                       │
         │ < max                      │
         └────────────────────────────┘
```

**关键参数**：
- `max_turns`: 最大循环次数（默认 10）
- `max_tokens`: 上下文窗口大小（默认 8000，约 2000 词）
- `CHARS_PER_TOKEN`: 估算系数（4 字符/Token）

## 3. 模块系统

### 3.1 生命周期

```
ModuleLoader::load_module()
    │
    ├── 1. 检查名称唯一性（避免重复加载）
    ├── 2. 验证依赖模块已加载（拓扑排序保证）
    ├── 3. 插入 HashMap<String, Arc<dyn OpsModule>>
    │
    ▼
ModuleManager（Gateway 层）
    │
    ├── enable(name)  → 标记 enabled
    ├── disable(name) → 标记 disabled
    ├── health_check() → 调用模块的 health_check()
    │
    ▼
ToolRegistry
    │
    ├── get_tools_for_llm() → 收集所有 enabled 模块的工具定义
    ├── invoke_tool() → 路由到正确的模块执行
    │
    ▼
Agent ReAct Loop 自动发现和调用工具
```

### 3.2 工具路由

`ToolRegistry` 维护两个视图：
1. **LLM 视图**：`Vec<Value>` — OpenAI function calling 格式
2. **路由索引**：`HashMap<String, Arc<dyn OpsModule>>` — 工具名 → 模块快速查找

## 4. 数据库 Schema

```sql
-- 用户认证
CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 主机管理
CREATE TABLE hosts (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    address TEXT NOT NULL,
    port INTEGER NOT NULL DEFAULT 22,
    username TEXT NOT NULL,
    auth_method TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'unknown',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 审计日志
CREATE TABLE audit_log (
    id TEXT PRIMARY KEY NOT NULL,
    "user" TEXT NOT NULL,
    action TEXT NOT NULL,
    resource TEXT NOT NULL,
    outcome TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

## 5. 事件系统

`EventBus` 基于 `tokio::broadcast`，实现多生产者多消费者模式：

- **发布**：`EventBus::publish(event)` → 广播到所有活跃订阅者
- **订阅**：`EventBus::subscribe()` → 返回独立接收器
- **背压**：慢订阅者收到 `RecvError::Lagged` 而非阻塞发布者

事件类型覆盖：
- 基础设施事件：`ConnectionAdded`、`ConnectionRemoved`、`CommandExecuted`
- Docker 事件：`DockerEvent`
- 监控事件：`HealthCheck`、`MetricUpdated`
- 审计事件：`AuditLog`
- 模块事件：`ModuleAction`

## 6. SSH 连接管理

```
SshConnectionPool (DashMap<String, Arc<SshConnection>>)
    │
    ├── get(host_id) → 检查连接活性，移除死连接
    ├── connect(host_id, config) → 建立新连接并加入池
    ├── disconnect(host_id) → 断开并移除
    └── disconnect_all() → 清空池

SshConnection
    ├── config: SshConfig
    ├── handle: Handle<ClientHandler>  // russh Handle，Arc-based
    ├── connected: Arc<AtomicBool>
    │
    ├── connect() → 建立连接 + 认证（密码/公钥）
    ├── exec(command) → 打开通道，执行命令，返回输出
    ├── disconnect() → 断开连接
    └── reconnect() → ⚠️ 当前有 bug，需要修复
```

## 7. LLM 集成

```rust
#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(&self, messages: &[Message]) -> Result<String, LlmError>;
    async fn complete_with_tools(&self, messages: &[Message], tools: &[Value])
        -> Result<CompletionResponse, LlmError>;
    async fn complete_stream(&self, messages: &[Message])
        -> Result<Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>>, LlmError>;
}
```

支持的提供商（通过配置切换）：
- OpenAI（GPT-4 等）
- Ollama（本地模型）
- MiMo（自定义接口）

## 8. 前端状态管理

```
Zustand Store (useAppStore)
    │
    ├── auth: { user, token, login(), logout() }
    ├── hosts: { hosts, loading, error, CRUD actions }
    ├── agent: { sessions, messages }
    └── ui: { sidebar, theme }

React Query (useQuery/useMutation)
    │
    ├── 自动缓存 API 响应
    ├── 401 自动触发登出
    ├── 乐观更新
    └── 失败重试
```

## 9. WebSocket 终端

```
Browser WebSocket
    │
    ▼
Axum WebSocket Handler
    │
    ├── 解析 host_id
    ├── 从 SshConnectionPool 获取连接
    ├── PTY 请求 (80x24)
    ├── Shell 请求
    │
    ├── ssh→ws: tokio::spawn 转发 channel output → WebSocket
    └── ws→ssh: 循环读取 WebSocket → channel stdin
```

## 10. 扩展指南

### 添加新模块

1. 在 `ops-pilot-sdk` 中 `#[derive(Serialize, Deserialize)]` 你的数据类型
2. 实现 `OpsModule` trait：
   - `tools()` 返回 `Vec<ToolDefinition>`（JSON Schema 参数）
   - `execute()` 处理工具调用
   - `on_event()` 可选的事件响应
3. 在 Gateway 中通过 `ModuleLoader::load_module()` 注册
4. Agent 自动发现你的工具

### 添加新 API 端点

1. 在 `ops-pilot-gateway/src/routes/` 下创建或修改路由模块
2. 定义 handler 函数（使用 `axum::extract`）
3. 在 `mod.rs` 中导出路由组合函数
4. 在 `server.rs` 中挂载到主 Router
