---
name: Pull Request
about: 提交代码变更
title: ''
labels: ''
assignees: ''
---

## 描述

请清晰描述此 PR 的改动内容和动机。

## 关联 Issue

Fixes #（填写关联的 Issue 编号）

## 变更类型

- [ ] Bug fix（非破坏性修复）
- [ ] New feature（非破坏性新增功能）
- [ ] Breaking change（可能导致现有功能异常的变更）
- [ ] Documentation（文档更新）
- [ ] Test（新增或修改测试）
- [ ] Refactor（重构，不涉及功能变化）
- [ ] Chore（构建/CI/工具链）

## 测试情况

- [ ] 后端测试通过：`cargo test --workspace`
- [ ] 前端测试通过：`npx vitest run`
- [ ] 类型检查通过：`npx tsc --noEmit`
- [ ] Lint 检查通过：`cargo clippy --workspace -- -D warnings` && `npx eslint src --max-warnings 0`

## 检查清单

- [ ] 我的代码遵循本项目的代码风格
- [ ] 我已自审查自己的代码
- [ ] 我为新功能添加了必要的测试
- [ ] 我更新了相关文档（README、API 文档等）
- [ ] 我的提交信息符合 [Conventional Commits](https://www.conventionalcommits.org/) 规范

## 截图（可选）

如有 UI 变更，请附上前后的截图对比。

## 补充说明

任何 Reviewer 需要知道的额外信息。
