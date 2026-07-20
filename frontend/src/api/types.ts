/** 模块基本信息 */
export interface ModuleInfo {
  /** 模块名称，如 "mod-core::ssh" */
  name: string;
  /** 语义化版本号 */
  version: string;
  /** 模块功能描述 */
  description: string;
  /** 是否启用 */
  enabled: boolean;
}

/**
 * 健康检查状态联合类型
 *
 * - `Healthy` — 模块运行正常
 * - `Degraded` — 模块部分降级，附带原因说明
 * - `Unhealthy` — 模块不可用，附带原因说明
 */
export type HealthStatus =
  | { Healthy: null }
  | { Degraded: { reason: string } }
  | { Unhealthy: { reason: string } };

/** 模块健康信息，包含名称、状态和启用状态 */
export interface ModuleHealth {
  /** 模块名称 */
  name: string;
  /** 健康检查状态 */
  status: HealthStatus;
  /** 模块是否启用 */
  enabled: boolean;
}

/** 模块自定义配置（键值对） */
export interface ModuleConfig {
  [key: string]: unknown;
}

// ── Host types ──────────────────────────────────────────────────────────

/** 主机状态枚举 */
export type HostStatus = 'online' | 'offline' | 'unknown' | 'maintenance';

/** 主机配置信息 */
export interface Host {
  /** 主机唯一标识 */
  id: string;
  /** 主机名称 */
  name: string;
  /** 主机地址（IP 或域名） */
  address: string;
  /** SSH 端口号 */
  port: number;
  /** SSH 登录用户名 */
  username: string;
  /** 认证方式（如 "key" 或 "password"） */
  auth_method: string;
  /** 主机当前状态 */
  status: HostStatus;
  /** 创建时间（ISO 8601） */
  created_at: string;
  /** 最后更新时间（ISO 8601） */
  updated_at: string;
}

/** 创建主机的请求参数 */
export interface CreateHostInput {
  /** 主机名称 */
  name: string;
  /** 主机地址（IP 或域名） */
  address: string;
  /** SSH 端口号（默认 22） */
  port?: number;
  /** SSH 登录用户名 */
  username: string;
  /** 认证方式（"key" 或 "password"） */
  auth_method: string;
  /** 密码（当 auth_method 为 "password" 时必填） */
  password?: string;
  /** 私钥内容（当 auth_method 为 "key" 时必填） */
  private_key?: string;
}

// ── Agent types ─────────────────────────────────────────────────────────

/** Agent 会话标识 */
export interface AgentSession {
  /** 会话唯一标识 */
  session_id: string;
}

/** Agent 响应，包含助手回复内容和工具调用轮次 */
export interface AgentResponse {
  /** 会话 ID */
  session_id: string;
  /** 助手回复的文本内容 */
  content: string;
  /** 工具调用轮次列表 */
  turns: AgentTurn[];
  /** 是否因达到最大轮次而截断 */
  truncated: boolean;
}

/** Agent 单轮工具调用记录 */
export interface AgentTurn {
  /** 轮次序号（从 1 开始） */
  turn: number;
  /** 执行的动作描述，如 "call get_server_status" */
  action: string;
  /** 工具返回的结果 */
  result: string;
}
