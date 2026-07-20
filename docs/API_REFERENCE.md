# API 参考 API Reference

OpsPilot v1 完整 REST API 参考文档。保留英文原文作为对照参考。

---

## Table of Contents

- [Base URL 基础地址](#base-url)
- [Authentication 认证](#authentication)
- [Endpoints 端点](#endpoints)
  - [Auth 认证](#auth)
  - [Hosts 主机](#hosts)
  - [SSH](#ssh)
  - [WebSocket](#websocket)
  - [Docker](#docker)
  - [Modules 模块](#modules)
  - [AI](#ai)
  - [Audit & Alerts 审计与告警](#audit--alerts)
  - [Dashboard 仪表盘](#dashboard)
- [Error Codes 错误码](#error-codes)
- [Rate Limiting 速率限制](#rate-limiting)

---

## Base URL 基础地址

```
http://localhost:3000/api/v1
```

All endpoints are prefixed with `/api/v1`. HTTPS is recommended in production.

---

## Authentication 认证

Most endpoints require a valid JWT token. Include it in the `Authorization` header:

```
Authorization: Bearer <token>
```

Obtain a token via `POST /auth/login`.

---

## Endpoints 端点

### Auth 认证

认证相关端点。支持用户注册、登录获取 JWT 令牌，所有受保护端点需在请求头携带 `Authorization: Bearer <token>`。

认证相关端点。注册新用户、登录获取 JWT 令牌。

#### `POST /auth/login`

Authenticate and receive a JWT token.

**Request:**

```json
{
  "username": "admin",
  "password": "ops-pilot"
}
```

**Response (200):**

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 900,
  "user": {
    "id": "user_001",
    "username": "admin",
    "role": "admin",
    "created_at": "2026-01-15T10:00:00Z"
  }
}
```

**Error (401):**

```json
{
  "error": {
    "code": "AUTH_INVALID_CREDENTIALS",
    "message": "Invalid username or password"
  }
}
```

---

#### `POST /auth/refresh`

Refresh an expired access token.

**Request:**

```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

**Response (200):**

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 900
}
```

---

### Hosts 主机

基础设施主机管理。支持 CRUD 操作、SSH 连接信息配置、在线状态追踪和健康检查。

管理基础设施主机，支持 SSH 连接信息和健康状态追踪。

#### `GET /hosts`

List all hosts.

**Headers:** `Authorization: Bearer <token>`

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | 1 | Page number |
| `per_page` | integer | 20 | Items per page (max 100) |
| `status` | string | — | Filter by status: `online`, `offline`, `unknown` |
| `search` | string | — | Search by name or IP |

**Response (200):**

```json
{
  "data": [
    {
      "id": "host_abc123",
      "name": "prod-web-01",
      "ip": "192.168.1.100",
      "port": 22,
      "os": "Ubuntu 22.04",
      "status": "online",
      "connected": true,
      "ssh_user": "deploy",
      "last_health_check": "2026-07-15T10:30:00Z",
      "tags": ["production", "web"],
      "created_at": "2026-01-15T10:00:00Z"
    },
    {
      "id": "host_def456",
      "name": "prod-db-01",
      "ip": "192.168.1.200",
      "port": 22,
      "os": "Ubuntu 22.04",
      "status": "online",
      "connected": false,
      "ssh_user": "deploy",
      "last_health_check": "2026-07-15T10:29:55Z",
      "tags": ["production", "database"],
      "created_at": "2026-01-15T10:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 2,
    "total_pages": 1
  }
}
```

---

#### `POST /hosts`

Add a new host.

**Request:**

```json
{
  "name": "prod-web-02",
  "ip": "192.168.1.101",
  "port": 22,
  "ssh_user": "deploy",
  "ssh_key_path": "/home/deploy/.ssh/id_ed25519",
  "tags": ["production", "web"],
  "metadata": {
    "environment": "production",
    "team": "platform"
  }
}
```

**Response (201):**

```json
{
  "id": "host_ghi789",
  "name": "prod-web-02",
  "ip": "192.168.1.101",
  "port": 22,
  "status": "unknown",
  "connected": false,
  "created_at": "2026-07-15T10:35:00Z"
}
```

---

#### `GET /hosts/:id`

Get host details.

**Response (200):**

```json
{
  "id": "host_abc123",
  "name": "prod-web-01",
  "ip": "192.168.1.100",
  "port": 22,
  "os": "Ubuntu 22.04",
  "kernel": "5.15.0-76-generic",
  "status": "online",
  "connected": true,
  "ssh_user": "deploy",
  "last_health_check": "2026-07-15T10:30:00Z",
  "tags": ["production", "web"],
  "metadata": {
    "environment": "production",
    "team": "platform"
  },
  "health": {
    "cpu_usage": 45.2,
    "memory_usage": 68.7,
    "disk_usage": 52.1,
    "uptime_seconds": 2592000,
    "load_average": [1.2, 0.8, 0.5]
  },
  "created_at": "2026-01-15T10:00:00Z",
  "updated_at": "2026-07-15T10:30:00Z"
}
```

---

#### `DELETE /hosts/:id`

Remove a host.

**Response (200):**

```json
{
  "message": "Host prod-web-01 deleted",
  "id": "host_abc123"
}
```

---

#### `POST /hosts/:id/connect`

Establish an SSH connection to the host.

**Request:**

```json
{
  "timeout_seconds": 10
}
```

**Response (200):**

```json
{
  "host_id": "host_abc123",
  "connected": true,
  "connection_id": "conn_xyz789",
  "server_info": {
    "ssh_version": "OpenSSH_8.9",
    "os": "Ubuntu 22.04.3 LTS",
    "uptime": "30 days, 2 hours"
  }
}
```

---

#### `POST /hosts/:id/disconnect`

Close the SSH connection to the host.

**Response (200):**

```json
{
  "host_id": "host_abc123",
  "connected": false,
  "message": "Disconnected"
}
```

---

#### `GET /hosts/:id/health`

Get detailed health metrics for a host.

**Response (200):**

```json
{
  "host_id": "host_abc123",
  "timestamp": "2026-07-15T10:30:00Z",
  "status": "healthy",
  "metrics": {
    "cpu": {
      "usage_pct": 45.2,
      "cores": 4,
      "model": "Intel Xeon E5-2680 v4"
    },
    "memory": {
      "total_gb": 16.0,
      "used_gb": 10.99,
      "usage_pct": 68.7,
      "swap_used_mb": 0
    },
    "disk": [
      {
        "device": "/dev/sda1",
        "mount": "/",
        "total_gb": 100.0,
        "used_gb": 52.1,
        "usage_pct": 52.1
      }
    ],
    "network": {
      "interfaces": [
        {
          "name": "eth0",
          "rx_bytes": 1073741824,
          "tx_bytes": 536870912
        }
      ]
    },
    "load_average": [1.2, 0.8, 0.5],
    "uptime_seconds": 2592000
  }
}
```

---

### SSH

SSH 连接与命令执行。支持密码和公钥认证，提供远程命令执行和 PTY 终端会话。

#### `POST /ssh/exec`

Execute a command on a host via SSH.

**Request:**

```json
{
  "host_id": "host_abc123",
  "command": "df -h /",
  "timeout_seconds": 30
}
```

**Response (200):**

```json
{
  "host_id": "host_abc123",
  "command": "df -h /",
  "stdout": "Filesystem      Size  Used Avail Use% Mounted on\n/dev/sda1       100G   52G   48G  52% /",
  "stderr": "",
  "exit_code": 0,
  "duration_ms": 245
}
```

**Error (403):**

```json
{
  "error": {
    "code": "SSH_COMMAND_BLOCKED",
    "message": "Command 'rm -rf /' is blocked by security policy",
    "command": "rm -rf /",
    "reason": "destructive_command"
  }
}
```

---

#### `POST /ssh/exec/batch`

Execute multiple commands sequentially on a host.

**Request:**

```json
{
  "host_id": "host_abc123",
  "commands": [
    "df -h /",
    "free -m",
    "uptime"
  ],
  "timeout_seconds": 60
}
```

**Response (200):**

```json
{
  "host_id": "host_abc123",
  "results": [
    {
      "command": "df -h /",
      "stdout": "Filesystem      Size  Used Avail Use% Mounted on\n/dev/sda1       100G   52G   48G  52% /",
      "stderr": "",
      "exit_code": 0,
      "duration_ms": 245
    },
    {
      "command": "free -m",
      "stdout": "              total        used        free      shared  buff/cache   available\nMem:          16384       10992        2048         256        3344        4880\nSwap:          2048           0        2048",
      "stderr": "",
      "exit_code": 0,
      "duration_ms": 180
    },
    {
      "command": "uptime",
      "stdout": " 10:30:00 up 30 days,  2:00,  1 user,  load average: 1.20, 0.80, 0.50",
      "stderr": "",
      "exit_code": 0,
      "duration_ms": 95
    }
  ],
  "total_duration_ms": 520
}
```

---

### WebSocket

WebSocket 实时通信端点。主要用于浏览器端 SSH 终端代理，通过 WebSocket 将用户输入转发到 SSH 通道。

#### `WS /ws/terminal`

Interactive terminal session via WebSocket.

**Connection:**

```
ws://localhost:3000/ws/terminal?token=<jwt>&host_id=host_abc123&cols=120&rows=40
```

**Message Protocol (JSON):**

| Direction | Type | Payload | Description |
|-----------|------|---------|-------------|
| Client → Server | `input` | `{"type":"input","data":"ls -la\n"}` | Terminal input |
| Client → Server | `resize` | `{"type":"resize","cols":120,"rows":40}` | Resize terminal |
| Server → Client | `output` | `{"type":"output","data":"..."}` | Terminal output |
| Server → Client | `exit` | `{"type":"exit","code":0}` | Command finished |

---

#### `WS /ws/logs/:connectionId`

Real-time log stream from an SSH session.

**Connection:**

```
ws://localhost:3000/ws/logs/conn_xyz789?token=<jwt>&follow=true
```

**Message Protocol:**

| Direction | Type | Payload | Description |
|-----------|------|---------|-------------|
| Server → Client | `log` | `{"type":"log","timestamp":"...","line":"..."}` | Log line |
| Server → Client | `eof` | `{"type":"eof"}` | End of file |

---

### Docker

Docker 容器管理。通过 Docker Engine API 实现容器列表、启动/停止、日志查看和资源统计。

#### `GET /docker/containers`

List Docker containers on all connected hosts.

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `host_id` | string | — | Filter by host |
| `all` | boolean | false | Include stopped containers |
| `status` | string | — | Filter: `running`, `stopped`, `paused` |

**Response (200):**

```json
{
  "data": [
    {
      "id": "abc123def456",
      "name": "nginx",
      "image": "nginx:1.25-alpine",
      "status": "running",
      "state": "running",
      "host_id": "host_abc123",
      "host_name": "prod-web-01",
      "created": "2026-07-01T10:00:00Z",
      "ports": [
        {
          "host": "0.0.0.0",
          "host_port": 80,
          "container_port": 80,
          "protocol": "tcp"
        }
      ],
      "stats": {
        "cpu_pct": 2.3,
        "memory_mb": 64,
        "network_rx_bytes": 1048576,
        "network_tx_bytes": 524288
      }
    }
  ],
  "total": 1
}
```

---

#### `POST /docker/:action`

Perform a Docker action on a container.

**Actions:** `start`, `stop`, `restart`, `remove`, `pause`, `unpause`

**Request:**

```json
{
  "container_id": "abc123def456",
  "host_id": "host_abc123",
  "force": false,
  "timeout_seconds": 30
}
```

**Response (200):**

```json
{
  "action": "restart",
  "container_id": "abc123def456",
  "host_id": "host_abc123",
  "success": true,
  "duration_ms": 1200
}
```

---

### Modules 模块

可插拔模块管理。支持模块的加载、启用/停用、健康检查和配置热更新。

#### `GET /modules`

List all installed modules.

**Response (200):**

```json
{
  "data": [
    {
      "name": "mod-core",
      "version": "0.1.0",
      "description": "Core host management, SSH, and Docker operations",
      "enabled": true,
      "status": "healthy",
      "category": "core",
      "tools_count": 12,
      "dependencies": [],
      "health": {
        "status": "healthy",
        "message": "OK",
        "last_check": "2026-07-15T10:30:00Z"
      },
      "config": {}
    },
    {
      "name": "mod-rca",
      "version": "0.1.0",
      "description": "AI-powered Root Cause Analysis",
      "enabled": true,
      "status": "healthy",
      "category": "analysis",
      "tools_count": 3,
      "dependencies": ["mod-core"],
      "health": {
        "status": "healthy",
        "message": "LLM connection OK",
        "last_check": "2026-07-15T10:30:00Z"
      },
      "config": {
        "max_log_lines": 10000,
        "correlation_window": 30
      }
    },
    {
      "name": "mod-finops",
      "version": "0.1.0",
      "description": "Cloud cost analysis and optimization",
      "enabled": false,
      "status": "disabled",
      "category": "finops",
      "tools_count": 3,
      "dependencies": ["mod-core"],
      "health": null,
      "config": {}
    }
  ]
}
```

---

#### `POST /modules/:name/enable`

Enable a module.

**Response (200):**

```json
{
  "name": "mod-finops",
  "enabled": true,
  "status": "initializing",
  "message": "Module mod-finops is initializing..."
}
```

---

#### `POST /modules/:name/disable`

Disable a module.

**Response (200):**

```json
{
  "name": "mod-finops",
  "enabled": false,
  "status": "stopped",
  "message": "Module mod-finops has been stopped"
}
```

---

#### `GET /modules/:name/config`

Get module configuration.

**Response (200):**

```json
{
  "name": "mod-rca",
  "config": {
    "max_log_lines": 10000,
    "correlation_window": 30,
    "auto_fix_threshold": 0.8,
    "llm_provider": "ollama",
    "llm_model": "qwen2.5:32b"
  },
  "schema": {
    "type": "object",
    "properties": {
      "max_log_lines": {
        "type": "integer",
        "default": 10000
      }
    }
  }
}
```

---

#### `PUT /modules/:name/config`

Update module configuration.

**Request:**

```json
{
  "config": {
    "max_log_lines": 20000,
    "correlation_window": 60
  }
}
```

**Response (200):**

```json
{
  "name": "mod-rca",
  "config": {
    "max_log_lines": 20000,
    "correlation_window": 60,
    "auto_fix_threshold": 0.8,
    "llm_provider": "ollama",
    "llm_model": "qwen2.5:32b"
  },
  "message": "Configuration updated. Module will reload."
}
```

---

### AI

AI Agent 对话接口。支持创建会话、多轮对话和工具调用（function calling），通过 ToolRegistry 路由到对应模块。

#### `POST /ai/chat`

Send a chat message to the AI assistant.

**Request:**

```json
{
  "message": "What's the CPU usage on prod-web-01?",
  "conversation_id": "conv_abc123",
  "provider": "ollama",
  "model": "qwen2.5:32b"
}
```

**Response (200, streaming):**

```json
{
  "conversation_id": "conv_abc123",
  "response": "Based on the current metrics, prod-web-01 has a CPU usage of 45.2% with 4 cores. The load average is 1.2, 0.8, 0.5, which indicates normal operation. No action needed.",
  "tool_calls": [
    {
      "tool": "host.health",
      "params": {"host_id": "host_abc123"},
      "result": {
        "cpu_usage": 45.2,
        "load_average": [1.2, 0.8, 0.5]
      }
    }
  ],
  "model": "qwen2.5:32b",
  "tokens_used": 256,
  "duration_ms": 1800
}
```

---

#### `POST /ai/execute`

Execute a natural language command as an AI agent. Unlike chat, this performs actions and returns results.

**Request:**

```json
{
  "instruction": "Restart nginx on prod-web-01",
  "auto_confirm": false,
  "provider": "ollama"
}
```

**Response (200):**

```json
{
  "status": "pending_confirmation",
  "plan": {
    "steps": [
      {
        "step": 1,
        "tool": "ssh.exec",
        "params": {
          "host_id": "host_abc123",
          "command": "systemctl restart nginx"
        },
        "description": "Restart nginx service",
        "risk_level": "medium"
      }
    ]
  },
  "requires_approval": true,
  "approval_token": "appr_xyz789",
  "message": "I'll restart nginx on prod-web-01. Please confirm this action."
}
```

**With `auto_confirm: true`:**

```json
{
  "status": "completed",
  "plan": {
    "steps": [
      {
        "step": 1,
        "tool": "ssh.exec",
        "params": {
          "host_id": "host_abc123",
          "command": "systemctl restart nginx"
        },
        "description": "Restart nginx service",
        "result": {
          "exit_code": 0,
          "stdout": "",
          "stderr": ""
        },
        "duration_ms": 1200
      }
    ]
  },
  "summary": "Nginx has been restarted on prod-web-01. The service is now running.",
  "total_duration_ms": 1200
}
```

---

### Audit & Alerts 审计与告警

操作审计和告警管理。记录所有 SSH 命令和 API 调用，支持基于规则的告警触发和通知。

#### `GET /audit-logs`

Query the audit trail.

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | 1 | Page number |
| `per_page` | integer | 50 | Items per page |
| `user_id` | string | — | Filter by user |
| `module` | string | — | Filter by module |
| `action` | string | — | Filter by action (partial match) |
| `from` | string | — | ISO 8601 start time |
| `to` | string | — | ISO 8601 end time |
| `risk_level` | string | — | Filter: `low`, `medium`, `high`, `critical` |

**Response (200):**

```json
{
  "data": [
    {
      "id": "evt_abc123",
      "timestamp": "2026-07-15T10:30:00Z",
      "user_id": "user_001",
      "username": "admin",
      "module": "mod-core",
      "action": "ssh.exec",
      "target": "host:prod-web-01",
      "details": {
        "command": "systemctl restart nginx",
        "exit_code": 0,
        "duration_ms": 1200
      },
      "risk_level": "medium",
      "ai_generated": false
    },
    {
      "id": "evt_def456",
      "timestamp": "2026-07-15T10:25:00Z",
      "user_id": null,
      "username": "ai-agent",
      "module": "mod-rca",
      "action": "rca.analyze",
      "target": "alert:alert_xyz",
      "details": {
        "root_cause": "Disk space exhaustion due to log rotation failure",
        "confidence": 0.92,
        "suggested_fix": "Run logrotate -f /etc/logrotate.conf"
      },
      "risk_level": "low",
      "ai_generated": true
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 2,
    "total_pages": 1
  }
}
```

---

#### `GET /alerts`

List active and recent alerts.

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `status` | string | `active` | Filter: `active`, `resolved`, `all` |
| `severity` | string | — | Filter: `info`, `warning`, `critical` |
| `host_id` | string | — | Filter by host |

**Response (200):**

```json
{
  "data": [
    {
      "id": "alert_xyz123",
      "host_id": "host_abc123",
      "host_name": "prod-web-01",
      "metric": "disk_usage",
      "severity": "warning",
      "status": "active",
      "message": "Disk usage on /dev/sda1 is at 92% (threshold: 90%)",
      "value": 92.0,
      "threshold": 90.0,
      "triggered_at": "2026-07-15T09:00:00Z",
      "acknowledged_by": null,
      "rca_suggestion": {
        "confidence": 0.88,
        "root_cause": "Log files accumulating in /var/log",
        "suggested_fix": "Rotate logs and clean up old archives"
      }
    }
  ],
  "summary": {
    "active": 1,
    "resolved_today": 3,
    "critical": 0,
    "warning": 1,
    "info": 0
  }
}
```

---

#### `POST /alerts/:id/acknowledge`

Acknowledge an alert.

**Request:**

```json
{
  "note": "Looking into this now"
}
```

**Response (200):**

```json
{
  "id": "alert_xyz123",
  "status": "acknowledged",
  "acknowledged_by": "admin",
  "acknowledged_at": "2026-07-15T10:35:00Z",
  "note": "Looking into this now"
}
```

---

#### `POST /alerts/:id/resolve`

Resolve an alert.

**Request:**

```json
{
  "resolution": "Cleaned up /var/log archives. Disk usage now at 65%."
}
```

**Response (200):**

```json
{
  "id": "alert_xyz123",
  "status": "resolved",
  "resolved_by": "admin",
  "resolved_at": "2026-07-15T10:40:00Z",
  "resolution": "Cleaned up /var/log archives. Disk usage now at 65%."
}
```

---

### Dashboard 仪表盘

聚合仪表盘数据。提供主机健康状态、模块运行状况、系统资源使用情况的统一视图。

#### `GET /dashboard/overview`

Get dashboard overview data.

**Response (200):**

```json
{
  "hosts": {
    "total": 12,
    "online": 10,
    "offline": 2,
    "unknown": 0
  },
  "alerts": {
    "active": 1,
    "critical": 0,
    "warning": 1
  },
  "modules": {
    "total": 4,
    "enabled": 3,
    "healthy": 3,
    "degraded": 0,
    "unhealthy": 0
  },
  "costs": {
    "current_month": 12450.67,
    "previous_month": 11200.32,
    "change_pct": 11.16,
    "forecast": 13800.00
  },
  "recent_activity": [
    {
      "timestamp": "2026-07-15T10:30:00Z",
      "action": "ssh.exec",
      "user": "admin",
      "target": "prod-web-01",
      "summary": "Restarted nginx"
    }
  ],
  "system": {
    "version": "0.1.0",
    "uptime_seconds": 864000,
    "database_size_mb": 12.5,
    "active_connections": 3
  }
}
```

---

## Error Codes 错误码

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `AUTH_INVALID_CREDENTIALS` | 401 | Wrong username or password |
| `AUTH_TOKEN_EXPIRED` | 401 | JWT token has expired |
| `AUTH_TOKEN_INVALID` | 401 | Malformed or invalid JWT token |
| `AUTH_INSUFFICIENT_PERMISSIONS` | 403 | User lacks required permission |
| `HOST_NOT_FOUND` | 404 | Host ID does not exist |
| `HOST_ALREADY_EXISTS` | 409 | Host with this IP/name already exists |
| `HOST_UNREACHABLE` | 502 | Cannot connect to host via SSH |
| `SSH_COMMAND_BLOCKED` | 403 | Command blocked by security policy |
| `SSH_TIMEOUT` | 504 | SSH command execution timed out |
| `SSH_CONNECTION_FAILED` | 502 | Failed to establish SSH connection |
| `DOCKER_UNAVAILABLE` | 502 | Docker daemon not reachable |
| `DOCKER_CONTAINER_NOT_FOUND` | 404 | Container ID does not exist |
| `MODULE_NOT_FOUND` | 404 | Module name does not exist |
| `MODULE_ALREADY_ENABLED` | 409 | Module is already enabled |
| `MODULE_LOAD_FAILED` | 500 | Module failed to load |
| `MODULE_DEPENDENCY_MISSING` | 400 | Required module not enabled |
| `AI_PROVIDER_UNAVAILABLE` | 502 | LLM provider not reachable |
| `AI_REQUEST_TIMEOUT` | 504 | LLM request timed out |
| `AI_INVALID_RESPONSE` | 500 | LLM returned malformed response |
| `VALIDATION_ERROR` | 400 | Request body failed validation |
| `NOT_FOUND` | 404 | Resource not found |
| `INTERNAL_ERROR` | 500 | Unexpected internal error |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |

**Standard Error Response Format:**

```json
{
  "error": {
    "code": "HOST_NOT_FOUND",
    "message": "Host with ID 'host_xyz' not found",
    "details": {
      "host_id": "host_xyz"
    }
  }
}
```

---

## Rate Limiting 速率限制

| Endpoint Group | Limit | Window |
|---------------|-------|--------|
| Auth (`/auth/*`) | 10 req | 1 minute |
| SSH (`/ssh/*`) | 60 req | 1 minute |
| AI (`/ai/*`) | 20 req | 1 minute |
| All others | 120 req | 1 minute |

Rate limit headers are included in responses:

```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 58
X-RateLimit-Reset: 1689412200
```

When rate limited (429):

```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded. Retry after 30 seconds.",
    "retry_after": 30
  }
}
```
