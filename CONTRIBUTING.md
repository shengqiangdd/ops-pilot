# Contributing to OpsPilot

感谢你考虑为 OpsPilot 贡献代码！以下指南帮助你快速上手。

## 报告 Bug

请使用 [Bug Report 模板](.github/ISSUE_TEMPLATE/bug_report.md) 提交 Issue，并确保包含：

- 清晰的标题和描述
- 复现步骤（最小复现示例最佳）
- 预期行为与实际行为
- 环境信息（OS、Rust 版本、Node 版本、浏览器等）
- 日志输出或错误截图

## 提交 Feature Request

请使用 [Feature Request 模板](.github/ISSUE_TEMPLATE/feature_request.md) 提交，并说明：

- 该功能解决什么场景问题
- 期望的 API 或交互方式
- 是否有可参考的实现

## 本地开发环境搭建

### 前置依赖

- **Rust** 1.82+（推荐通过 [rustup](https://rustup.rs/) 安装）
- **Node.js** 20+（推荐通过 [nvm](https://github.com/nvm-sh/nvm) 管理）
- **SQLite**（通过 `libsqlite3-sys` 捆绑编译，通常无需单独安装）
- **Cargo 工具链完整**：`cargo install cargo-make cargo-watch`（可选）

### 后端

```bash
cd backend
cp ../.env.example ../.env   # 编辑 .env 填入 LLM_API_KEY 等
cargo build --workspace       # 编译全部 crate
cargo test --workspace        # 运行所有测试
cargo run -p ops-pilot-gateway  # 启动网关
```

### 前端

```bash
cd frontend
npm install
npm run dev            # 开发服务器，默认 http://localhost:5173
npm run build          # 生产构建
npx vitest run         # 运行前端测试
npx tsc --noEmit       # 类型检查
```

## 代码风格

### Rust

- 使用 `cargo fmt` 格式化（项目根目录包含 `rustfmt.toml`）
- 使用 `cargo clippy --workspace -- -D warnings` 确保无警告
- 遵循 Rust API 准则（见 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)）
- 所有公有 API 必须添加文档注释（`///`）

### TypeScript / React

- 使用 ESLint + Prettier 保持代码风格一致：

```bash
cd frontend
npx eslint 'src/**/*.{ts,tsx}' --max-warnings 0
npx prettier --check 'src/**/*.{ts,tsx,css,json}'
npx prettier --write 'src/**/*.{ts,tsx,css,json}'   # 自动修复
```

- 使用 TypeScript 严格模式（`strict: true`）
- React 组件使用函数组件 + Hooks
- 状态管理优先使用 Zustand store

### 提交信息规范

我们使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>(<scope>): <description>

[optional body]
[optional footer]
```

类型（type）包括：

| 类型 | 说明 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `docs` | 文档修改 |
| `style` | 代码格式调整（不影响功能） |
| `refactor` | 重构（既非 feat 也非 fix） |
| `test` | 新增或修改测试 |
| `chore` | 构建/CI/工具链变更 |
| `perf` | 性能优化 |
| `ci` | CI 配置变更 |

示例：

```
feat(hosts): 添加批量命令执行功能
fix(vault): 修复空口令导致 panic 的问题
test(alert): 为告警规则 CRUD 添加单元测试
docs: 更新 README 中的 API 端点表格
```

## PR 流程

1. Fork 本仓库并创建你的特性分支：`git checkout -b feat/my-feature`
2. 在本地进行开发并确保所有测试通过
3. 运行 lint 检查：`cargo clippy --workspace && cd frontend && npx eslint src --max-warnings 0`
4. 确保你的提交信息符合 Conventional Commits 规范
5. 推送分支并创建 Pull Request
6. 在 PR 描述中清晰说明改动内容和影响范围
7. 等待 Code Review，根据反馈进行修改

### PR 检查清单

- [ ] 所有测试通过（`cargo test --workspace && npx vitest run`）
- [ ] 代码格式化（`cargo fmt` + `npx prettier --write`）
- [ ] 无新增 clippy / eslint 警告
- [ ] 新增代码包含适当的测试
- [ ] 文档已更新（README、API 文档等）
- [ ] 提交信息符合 Conventional Commits

## 项目结构

```
ops-pilot/
├── backend/
│   ├── gateway/        # HTTP 网关（Axum REST + WebSocket）
│   ├── core/           # 核心引擎（SSH、Docker、Vault、Auth 等）
│   ├── sdk/            # 模块 SDK（ModuleLoader、EventBus 等）
│   └── modules/        # 功能模块
├── frontend/           # React 19 + Vite 6 + TypeScript
├── docs/               # API 文档、架构图
└── .github/            # GitHub 模板、Actions
```

## 获取帮助

- 提交 Issue 或 Discussion
- 联系维护者：在 GitHub 上 @shengqiangdd

再次感谢你的贡献！🎉
