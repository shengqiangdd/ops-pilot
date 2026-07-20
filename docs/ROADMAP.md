# 项目路线图

本文档概述 OpsPilot 的四个发展阶段，从 MVP 到生态成熟。

---

## 时间线总览

```
阶段 1         阶段 2         阶段 3         阶段 4
MVP            模块化          企业级          生态建设
第 1-2 月      第 3-4 月      第 5-6 月       第 7 月+
─────────────────────────────────────────────────────────────►

核心引擎       mod-rca         mod-security    模块市场
SSH + Docker   mod-finops      mod-topo        K8s Operator
Web UI         AI 集成         mod-chatops     多租户
基础 AI 对话    事件系统         RBAC            企业 SSO
```

---

## 阶段 1：MVP（第 1-2 月）

**目标：** 实现可用的单机基础设施运维平台，支持主机管理、SSH 命令执行、Docker 容器控制，以及通过 Web 界面的基础 AI 辅助运维。

### 交付物

| # | 交付物 | 说明 | 验收标准 |
|---|--------|------|---------|
| 1.1 | 核心引擎 | Rust 后端，axum HTTP 服务器，SQLite 数据库层，连接池 | 100 并发 SSH 会话无内存泄漏 |
| 1.2 | SSH 模块 | 基于 russh 的持久 SSH 连接、命令执行、终端 PTY | 99.9% 命令执行成功率 |
| 1.3 | Docker 模块 | 容器列表、启动/停止/重启、基础统计 | 兼容 Docker Engine 24+ |
| 1.4 | 主机管理 | 主机 CRUD、健康检查、状态跟踪 | 单实例支持 50+ 主机 |
| 1.5 | Web UI 框架 | React SPA，路由、认证、侧边栏、仪表盘布局 | Lighthouse 评分 > 90 |
| 1.6 | 终端 UI | xterm.js WebSocket SSH 终端 | 支持 256 色、自适应缩放、复制粘贴 |
| 1.7 | 基础 AI 对话 | Ollama/OpenAI 集成聊天界面，SSH 工具调用 | 简单查询响应 < 3s |
| 1.8 | 认证系统 | JWT 认证、用户注册、密码哈希 | bcrypt cost factor 12 |
| 1.9 | 审计追踪 | 只追加的操作日志，带查询 API | 记录所有 SSH 命令和 API 调用 |
| 1.10 | Docker Compose | 一键部署，包含所有依赖 | `docker compose up` 在 Linux/macOS 可运行 |

### 技术里程碑

- **第 1-2 周：** 核心引擎脚手架、数据库 schema、SSH 连接池
- **第 3-4 周：** Docker 集成、主机管理 API
- **第 5-6 周：** Web UI 框架、认证、终端 WebSocket
- **第 7-8 周：** AI 对话集成、审计追踪、Docker Compose、文档

### 风险因素

| 风险 | 影响 | 应对方案 |
|------|------|---------|
| russh 兼容性问题 | 高 | 评估 SSH2 crate 作为备选 |
| WebSocket 高负载稳定性 | 中 | 实现连接限制和心跳机制 |
| AI 响应延迟 | 中 | 支持流式响应，默认使用本地 Ollama |

---

## 阶段 2：模块化（第 3-4 月）

**目标：** 将核心功能抽取为可插拔模块系统，交付首批领域模块（RCA 和 FinOps）。

### 交付物

| # | 交付物 | 说明 | 验收标准 |
|---|--------|------|---------|
| 2.1 | Module SDK | trait 定义、生命周期管理、依赖注入、配置 | 所有现有功能以模块方式运行 |
| 2.2 | mod-core | 将主机/SSH/Docker 抽取为独立模块 | 与阶段 1 零回归 |
| 2.3 | mod-rca v0.1 | 基于告警和日志的自动根因分析 | 正确识别 70%+ 常见问题 |
| 2.4 | mod-rca + AI | LLM 驱动分析，含日志关联和修复建议 | 全链路分析 < 30s |
| 2.5 | mod-finops v0.1 | 云成本数据采集（AWS），异常检测 | 支持 AWS Cost Explorer API |
| 2.6 | 事件系统 | tokio broadcast 通道、类型化事件、模块发布订阅 | 事件传播延迟 < 1ms |
| 2.7 | 模块管理器 | 通过 API 和 UI 启用/停用/重载模块 | 热重载无需重启核心 |
| 2.8 | 模块 UI | 模块浏览器、配置编辑器、健康仪表盘 | 实时展示模块状态 |
| 2.9 | AI 工具注册表 | 模块向 AI Gateway 注册工具，支持 function calling | LLM 可自动调用模块工具 |
| 2.10 | 增强 AI | 多轮对话、上下文窗口管理、工具执行 | 支持 20+ 轮对话 |

### 技术里程碑

- **第 9-10 周：** Module SDK 设计、trait 定义、加载器实现
- **第 11-12 周：** mod-core 抽取、事件总线、模块管理器
- **第 13-14 周：** mod-rca 开发、LLM 集成分析
- **第 15-16 周：** mod-finops 开发、AI 工具注册表、模块 UI

### 成功标准

- 阶段 1 全部功能在 mod-core 下无差异运行
- 第三方开发者使用 SDK 文档可在 < 2 小时内创建模块
- mod-rca 在 15 秒内从日志中识别"磁盘已满"
- mod-finops 在异常发生 1 小时内检测到 2x 成本飙升

---

## 阶段 3：企业级（第 5-6 月）

**目标：** 增加安全扫描、网络拓扑可视化、ChatOps 集成和基于角色的访问控制。

### 交付物

| # | 交付物 | 说明 | 验收标准 |
|---|--------|------|---------|
| 3.1 | mod-security v0.1 | SSH 密钥审计、CVE 扫描、CIS 合规检查 | 50 台主机扫描 < 5 分钟 |
| 3.2 | mod-topo v0.1 | 基于 SSH 的网络拓扑发现、依赖映射 | 自动发现 90%+ 网络链路 |
| 3.3 | mod-topo UI | React Flow 交互式拓扑可视化 | 缩放/搜索/过滤/导出 |
| 3.4 | mod-chatops v0.1 | Slack/Discord Webhook 集成、事件通知 | 30 秒内送达告警 |
| 3.5 | mod-chatops 指令 | 通过聊天平台的自然语言命令 | `!ops restart nginx` 在 Slack 生效 |
| 3.6 | RBAC | 基于角色的访问控制、权限组、资源范围 | Admin/Operator/Viewer 三种角色 |
| 3.7 | 凭据保险箱 | 加密凭据存储、轮换提醒、审计 | AES-256-GCM 静态加密 |
| 3.8 | Webhook 系统 | 可配置事件 Webhook，带重试机制 | 99.9% 送达率 |
| 3.9 | API Key | 长期 API Key，作用域权限 | 支持只读和管理 Key |
| 3.10 | 邮件通知 | 通过 SMTP 发送告警，支持摘要模式和模板 | 兼容主流 SMTP 提供商 |

### 技术里程碑

- **第 17-18 周：** mod-security 和 mod-chatops 脚手架
- **第 19-20 周：** mod-topo 发现引擎和可视化
- **第 21-22 周：** RBAC 系统、凭据保险箱、Webhook 引擎
- **第 23-24 周：** 集成测试、性能调优、文档

### 成功标准

- mod-topo 可自动映射三层 Web 应用拓扑
- mod-chatops 在触发后 30 秒内发送 Slack 告警
- RBAC 阻止非管理员执行破坏性命令
- 安全扫描发现最近 30 天内的已知 CVE

---

## 阶段 4：生态建设（第 7 月+）

**目标：** 构建社区贡献生态，扩展到多租户部署，集成 Kubernetes。

### 交付物

| # | 交付物 | 说明 | 验收标准 |
|---|--------|------|---------|
| 4.1 | 模块市场 | 社区模块注册、搜索、安装、评分 | 10+ 个社区模块上架 |
| 4.2 | K8s Operator | OpsPilot 的 Kubernetes Operator | 在 EKS/GKE/AKS 可运行 |
| 4.3 | 多租户 | 组织隔离、共享模块、跨组织可见性 | 单实例支持 50+ 组织 |
| 4.4 | 企业 SSO | SAML/OIDC 集成 | 兼容 Okta/Azure AD/Auth0 |
| 4.5 | Prometheus Exporter | 暴露 OpsPilot 指标 | 标准 `/metrics` 端点 |
| 4.6 | Terraform Provider | 通过 Terraform 管理 OpsPilot 资源 | 主机、模块、告警即代码 |
| 4.7 | CLI v2 | 全功能命令行，交互模式，shell 自动补全 | 功能与 Web UI 一致 |
| 4.8 | 移动端适配 | 移动/平板优化界面 | iOS/Android 浏览器可用 |
| 4.9 | 成本优化引擎 | 自动化建议，一键应用 | 节省 20%+ 云成本 |
| 4.10 | 事件时间线 | 关联事件的可视化时间线 | 完整展示 RCA → 修复 → 解决 |

### 成功标准

- 公开发布 3 个月内 100+ GitHub Stars
- 5+ 社区贡献模块
- K8s Operator 在任意主流云上 5 分钟内完成部署
- 多租户部署在 10 个组织下管理 1000+ 主机

---

## 阶段 4 之后

未来可能的方向：

- **AIOps 管道：** 基于基础设施数据训练的自定义异常检测模型
- **成本优化引擎：** 自动竞价实例管理、预留实例建议
- **合规仪表盘：** 实时展示 CIS/SOC2/HIPAA/PCI-DSS 合规状态
- **GitOps 集成：** 通过 Git PR 部署基础设施变更
- **服务网格可观测性：** Istio/Linkerd 集成
- **混沌工程：** 受控故障注入，自动回滚
- **自定义仪表盘：** 拖拽式仪表盘构建器
- **API Gateway：** 对外暴露 OpsPilot 工具为托管 API

---

## 版本策略

| 版本 | 阶段 | 破坏性变更 | 支持 |
|------|------|-----------|------|
| 0.1.x | 阶段 1（MVP） | 无 | 社区 |
| 0.2.x | 阶段 2（模块化） | Module API v1 | 社区 |
| 0.3.x | 阶段 3（企业级） | RBAC schema 变更 | 社区 + 企业 |
| 1.0.0 | 阶段 4（生态） | 稳定 API 契约 | 长期支持 |

**语义化版本策略：**

- **MAJOR**（1.0.0, 2.0.0）：Module SDK、REST API 或数据库 schema 的破坏性变更。
- **MINOR**（0.1.0 → 0.2.0）：新功能、模块或 API 端点。向后兼容。
- **PATCH**（0.1.0 → 0.1.1）：Bug 修复、性能改进、文档更新。

---

## 参与路线图

欢迎社区对优先级提出建议。如需提议变更：

1. 创建 GitHub Issue，标记 `roadmap` 标签
2. 描述功能或改进
3. 附上使用场景和预期影响
4. 核心团队根据社区反馈和市场需求进行评估

路线图每月由核心团队审查，根据社区反馈和市场需求更新。

---

> **English version:** This roadmap is primarily in Chinese. All code references, API endpoints, and technical terms remain in English. To read the original English version, check the [commit history](https://github.com/shengqiangdd/ops-pilot/commits/main/docs/ROADMAP.md).
