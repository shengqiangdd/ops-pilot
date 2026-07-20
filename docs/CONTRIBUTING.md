# 参与贡献 OpsPilot

感谢你对 OpsPilot 的关注！本文档帮助你快速了解开发流程、编码规范和 PR 提交流程。

---

## 目录

- [快速开始](#快速开始)
- [开发环境配置](#开发环境配置)
- [项目结构](#项目结构)
- [模块开发](#模块开发)
- [代码风格](#代码风格)
- [测试](#测试)
- [Pull Request 流程](#pull-request-流程)
- [Issue 指南](#issue-指南)
- [行为准则](#行为准则)

---

## 快速开始

### 前置依赖

| 工具 | 版本 | 用途 |
|------|------|------|
| **Rust** | 1.75+ | 后端语言 |
| **Node.js** | 20+ | 前端构建 |
| **Docker** | 24+ | 容器测试 |
| **Git** | 2.30+ | 版本控制 |

### 快速安装

```bash
# 1. Fork 并克隆
git clone https://github.com/YOUR_USERNAME/ops-pilot.git
cd ops-pilot

# 2. 安装 Rust 工具链
rustup default stable
rustup component add clippy rustfmt

# 3. 安装前端依赖
cd frontend && npm install && cd ..

# 4. 复制环境配置
cp .env.example .env

# 5. 启动开发服务（可选：本地 LLM）
docker compose up -d ollama

# 6. 构建并运行
cargo build
cargo run

# 7. 运行测试
cargo test
cd frontend && npm test && cd ..
```

### Makefile 快捷命令

```bash
make dev          # 完整开发环境
make build        # 构建所有 crate
make test         # 运行全部测试
make lint         # clippy + eslint
make fmt          # 格式化代码
make run          # 启动服务
make docker-up    # Docker Compose 启动
make docker-down  # Docker Compose 停止
```

---

## 开发环境配置

### Rust 工具链

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable
rustup component add clippy rustfmt rust-analyzer
```

### 前端工具链

```bash
nvm install 20 && nvm use 20
cd frontend && npm install
npm run dev  # Vite 开发服务器，独立终端
```

### 数据库

OpsPilot 默认使用 SQLite（零配置）。如需 PostgreSQL：

```bash
docker run -d --name ops-pilot-pg \
  -e POSTGRES_DB=ops_pilot \
  -e POSTGRES_USER=ops \
  -e POSTGRES_PASSWORD=dev \
  -p 5432:5432 \
  postgres:16

# 更新 .env
DATABASE_URL=postgres://ops:dev@localhost:5432/ops_pilot
```

### LLM 提供商（可选）

```bash
docker run -d --name ollama -p 11434:11434 -v ollama_data:/root/.ollama ollama/ollama:latest
docker exec ollama ollama pull qwen2.5:32b
```

### IDE 配置

**VS Code 推荐：**

```json
// .vscode/settings.json
{
  "rust-analyzer.cargo.features": ["all"],
  "rust-analyzer.check.command": "clippy",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[typescript][typescriptreact]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  }
}
```

**推荐插件：** rust-analyzer、Even Better TOML、ESLint、Prettier、Tailwind CSS IntelliSense

---

## 项目结构

```
ops-pilot/
├── Cargo.toml                 # Workspace 根
├── src/
│   ├── core/                  # 核心引擎 crate
│   ├── gateway/               # AI Gateway crate
│   ├── sdk/                   # Module SDK crate
│   └── modules/               # 可插拔模块
│       ├── mod-core/          # 主机/SSH/Docker 管理
│       ├── mod-rca/           # 根因分析
│       ├── mod-finops/        # 成本优化
│       ├── mod-security/      # 安全扫描
│       ├── mod-topo/          # 拓扑可视化
│       └── mod-chatops/       # 聊天平台集成
├── frontend/                  # React 前端
├── docs/                      # 文档
└── .github/                   # GitHub 配置
```

### Crate 依赖关系

```
ops-pilot (core)
├── ops-pilot-sdk     (trait 定义，不依赖 core)
├── ops-pilot-gateway (AI 集成，依赖 sdk)
├── mod-core          (依赖 sdk)
└── 其他模块           (依赖 sdk, mod-core)
```

**关键约束：** `sdk` crate 绝不能依赖 `core`，防止循环依赖，保证模块可独立开发。

---

## 模块开发

### 创建新模块

```bash
cargo new src/modules/mod-my-feature --lib
```

### 模块检查清单

提交新模块前：

- [ ] 实现 `OpsModule` trait，包含所有必需方法
- [ ] 包含 `module.toml` 清单文件
- [ ] 单元测试覆盖所有工具
- [ ] 至少一个端到端集成测试
- [ ] 所有公共项有文档注释
- [ ] 符合代码风格（clippy, rustfmt）
- [ ] 无硬编码配置值
- [ ] 优雅处理错误（生产路径无 unwrap/expect）
- [ ] 健康检查返回有意义的状态

---

## 代码风格

### Rust

默认使用 `rustfmt` + `clippy` 严格模式。

**规则：**

- 提交前运行 `cargo fmt`
- 运行 `cargo clippy -- -D warnings`，零警告
- 错误类型用 `thiserror`，应用错误用 `anyhow`
- 优先 `Result<T, E>` 而非 panic
- 公共项用 `///` 文档注释
- 优先 `#[derive]` 而非手动实现

### TypeScript / 前端

**规则：**

- 提交前运行 `npm run lint`
- 使用函数式组件 + hooks
- TypeScript strict 模式——禁止 `any`
- Props 接口从组件文件导出
- 测试文件与组件同目录：`Component.test.tsx`

### 提交消息

遵循 [Conventional Commits](https://www.conventionalcommits.org/)：

```
feat(mod-rca): add log correlation for multi-host incidents

Fixes #123

- Correlate logs across hosts within a 5-minute window
- Use AI to identify patterns in correlated logs
- Add unit tests for correlation engine
```

**类型：** `feat` / `fix` / `docs` / `style` / `refactor` / `test` / `chore` / `perf`

**作用域：** `core`, `sdk`, `gateway`, `mod-core`, `mod-rca`, `ui`, `api`, `docs`, `ci`

---

## 测试

### 后端测试

```bash
cargo test                      # 单元测试
cargo test -- --nocapture       # 带输出
cargo test test_ssh_connection  # 指定测试
cargo test --test integration   # 集成测试
```

**测试规范：** Arrange → Act → Assert，每个测试独立，使用 `MockModuleContext` 模拟依赖。

### 前端测试

```bash
npm test              # 全部测试
npm test -- --watch   # 监听模式
npm test -- --coverage  # 覆盖率
```

### 覆盖率目标

| 组件 | 最低覆盖率 |
|------|-----------|
| 核心引擎 Core Engine | 80% |
| Module SDK | 90% |
| 模块 Modules | 85% |
| AI Gateway | 75% |
| 前端组件 | 70% |
| 前端 Hooks/Utils | 85% |

---

## Pull Request 流程

### 提交前检查

1. **同步 main 分支：** `git fetch origin && git rebase origin/main`
2. **运行全部检查：** `cargo fmt --check && cargo clippy -- -D warnings && cargo test && cd frontend && npm run lint && npm test && cd ..`
3. **更新文档**——如果变更影响公共 API 或用户功能

### 分支命名

```
feature/mod-rca-log-correlation
fix/ssh-connection-timeout
docs/api-reference-update
chore/upgrade-axum-0.8
```

### 审查流程

1. **自动化检查**必须通过（CI: clippy, tests, build）
2. **至少一位维护者**审查
3. **无未解决的对话**
4. **Squash and merge** 保持历史清洁

---

## Issue 指南

### Bug 报告

使用 bug 报告模板，包含：环境信息、复现步骤、期望行为、实际行为、日志、截图。

### 功能请求

使用功能请求模板，包含：解决的问题、建议方案、备选方案、真实场景。

---

## 行为准则

我们遵循 [Contributor Covenant](https://www.contributor-covenant.org/version/2/1/code_of_conduct/) 行为准则。

**核心：** 尊重他人、建设性反馈、包容新人、保持专业。

违规行为可向项目维护者报告。

---

## 获取帮助

- **GitHub Discussions** —— 问答和讨论
- **GitHub Issues** —— Bug 报告和功能请求
- **Module SDK 文档** —— [docs/MODULE_SDK.md](MODULE_SDK.md)

感谢你为 OpsPilot 做贡献！🚀
