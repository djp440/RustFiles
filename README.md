# RustFiles

高性能、精致、现代化的 Windows 本地文件管理器，目标成为 Windows 资源管理器的日常替代品。

## 技术栈

- **桌面壳**：Tauri 2
- **核心**：Rust
- **前端**：React + TypeScript + Vite
- **测试**：Vitest（单元/组件）、Playwright（e2e）、Rust 集成测试

## 快速开始

```bash
npm install
npm run tauri:dev:test
```

## 验证命令

```bash
npm run check:all                          # 类型检查 + 前端测试 + Cargo 测试 + 构建
npm run e2e -- e2e/navigation.spec.ts       # 导航 e2e
npm run e2e -- e2e/view-sort-filter.spec.ts # 排序/过滤 e2e
npm run e2e -- e2e/glass-readability.spec.ts # 视图与 Glass 材质 e2e
npm run e2e -- e2e/performance.spec.ts      # 大目录虚拟列表 e2e
npm run e2e -- e2e/ui-priority.spec.ts      # UI 优先调度 browser-preview e2e
npm run e2e -- e2e/search.spec.ts           # 搜索、取消与打开所在位置 browser-preview e2e
npm run test -- src/test/interaction-reporting.test.ts # 调度上报前端测试
npm run test -- src/test/search-store.test.ts          # 搜索 store 前端测试
```

## 项目文档

- PRD：`docs/cdd-brainstorming/prds/`
- CDD 规格：`docs/cdd-specification/specs/`
- 架构设计：`docs/cdd-architecture-design/architectures/`
- 前端 UI 设计：`docs/cdd-frontend-design/ui-designs/`
- 实现计划：`docs/cdd-writing-plans/`
