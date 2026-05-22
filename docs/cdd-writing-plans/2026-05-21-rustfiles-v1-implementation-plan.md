# RustFiles V1 Implementation Plan

> **For agentic workers:** Use checkbox syntax (`- [ ]`) for task tracking. Execute tasks in order unless explicitly marked parallel-safe.

**Goal:** 按细粒度版本切片实现 RustFiles V1，使它能在 Windows 10 / 11 上完成本地文件管理器的浏览、导航、搜索、排序、缩略图、基础设置、多标签和稳定文件操作闭环。

**Architecture:** 采用 React + Tauri + Rust Core 分层架构。React 只表达用户意图和展示状态，Tauri 只承担桌面壳、窗口和 command/event 桥接，Rust Core 是唯一真实文件系统副作用入口，并通过任务队列、路径安全、UI 优先调度和结构化事件保障可验证性。

**Tech Stack:** Rust stable、Tauri 2、React + TypeScript + Vite、TanStack Virtual、Zustand、Vitest、Playwright、Cargo test、Windows API / Rust 系统集成 crate、应用数据目录中的 settings 与 thumbnail cache。

---

## 背景与输入

- PRD：`docs/cdd-brainstorming/prds/2026-05-21-rustfiles-v1-prd.md`
- CDD 规格文档：`docs/cdd-specification/specs/2026-05-21-rustfiles-v1-cdd.md`
- 架构设计文档：`docs/cdd-architecture-design/architectures/2026-05-21-rustfiles-v1-architecture.md`
- 前端 UI 设计文档：`docs/cdd-frontend-design/ui-designs/2026-05-21-rustfiles-v1-frontend-ui.md`
- 当前代码库约束：仓库当前只有文档、`README.md`、`LICENSE`、`.gitignore` 和 `AGENTS.md`；尚无 Rust、React、Tauri、测试或构建脚手架。
- 风险等级：`R4`。永久删除、覆盖、移动、剪切、粘贴、拖拽移动、冲突处理和回收站集成必须人工审核。

## 目标与非目标

### 目标

- 从零建立可开发、可测试、可打包的 Tauri + React + Rust 工程。
- V0.1 到 V1.0 每个版本只推进同一类能力，并在每个版本末尾做真实运行层面的 e2e 验证。
- 在任何真实写操作前完成路径安全、测试根目录 guard、任务状态机、错误模型和 Tauri command 白名单。
- 实现 UI 优先调度：后台扫描、缩略图、搜索和文件操作可以降速，但前台滚动、选择、路径跳转、标签切换和菜单交互必须保持流畅。
- 实现 Apple-like Liquid Workbench UI：统一玻璃材质、可读性、降级策略、Windows 原生窗口行为和 forced-colors 支持。
- 发布阶段至少能打包成一个可直接启动的桌面二进制入口；调试阶段允许前后端分离运行。

### 非目标

- 不实现云盘同步、FTP/SFTP、插件系统、全文索引、AI 整理、压缩包浏览、内置终端、内置预览器、文件标签、Git 状态。
- 不实现标签分组、会话同步、跨窗口拖拽标签或复杂工作区恢复。
- 不把视频缩略图、PDF 缩略图作为 V1 阻断项。
- 不把与 Windows 资源管理器的跨应用拖入/拖出作为 V1 必达项；V1 默认只保证应用内拖拽语义。

## 文件与模块结构

### 根目录与工程脚手架

- `package.json`：前端、Tauri CLI、测试和 e2e 脚本入口。
- `index.html`：Vite 入口。
- `vite.config.ts`：React/Vite/Vitest 配置。
- `tsconfig.json`、`tsconfig.node.json`：TypeScript 配置。
- `playwright.config.ts`：Web UI 和 Tauri dev e2e 配置。
- `src-tauri/Cargo.toml`：Rust crate、features、测试依赖。
- `src-tauri/tauri.conf.json`：Tauri 应用配置、窗口、构建和 capability 引用。
- `src-tauri/capabilities/default.json`：Tauri command 白名单和插件权限。
- `scripts/create-fixtures.ps1`：生成隔离测试夹具，包括 small、large-10k、media、conflict、deep-tree、permission cases。
- `scripts/run-e2e.ps1`：启动 dev 进程并运行 Playwright。
- `.github/workflows/ci.yml`：Windows CI，运行 Rust、TS、单元测试、构建和可运行 smoke。

### Rust Core

- `src-tauri/src/main.rs`：Tauri 入口，调用 `rustfiles::run()`。
- `src-tauri/src/lib.rs`：Tauri builder、插件、command 注册、全局 state 注入。
- `src-tauri/src/commands.rs`：Tauri command adapter，只做入参/出参映射。
- `src-tauri/src/core/mod.rs`：Rust Core 聚合入口。
- `src-tauri/src/core/types.rs`：`FileEntry`、`DirectoryPage`、`TabState`、`FileTask`、`Settings` 等共享类型。
- `src-tauri/src/core/error.rs`：结构化错误码、用户可读消息、可重试标记和刷新建议。
- `src-tauri/src/core/path_safety.rs`：Windows 路径规范化、reparse point 分类、测试根 guard、命名校验。
- `src-tauri/src/core/fs.rs`：只读目录枚举、排序、过滤、分页、快照版本。
- `src-tauri/src/core/tasks.rs`：写操作任务队列、状态机、取消、部分完成、冲突暂停。
- `src-tauri/src/core/clipboard.rs`：应用内复制/剪切 pending operation。
- `src-tauri/src/core/drag.rs`：应用内 drag operation 和默认复制/移动语义。
- `src-tauri/src/core/scheduler.rs`：UI 优先调度信号、优先级、事件背压。
- `src-tauri/src/core/search.rs`：当前目录搜索、递归搜索、取消、批量结果。
- `src-tauri/src/core/thumbnails.rs`：图片缩略图、缓存 key、可视区域优先、失败退避。
- `src-tauri/src/core/settings.rs`：带 `schema_version` 的原子设置读写。
- `src-tauri/src/core/system.rs`：回收站、默认应用、终端、属性、常用目录、磁盘分区。
- `src-tauri/src/core/observability.rs`：日志、性能采样、任务审计。
- `src-tauri/tests/*.rs`：Rust 模块级和集成测试。

### React UI

- `src/main.tsx`：React 入口。
- `src/App.tsx`：应用根组件。
- `src/styles/tokens.css`：语义色、玻璃、间距、圆角、z-index、forced-colors token。
- `src/styles/global.css`：全局布局、字体、scrollbar、reduced motion/transparency。
- `src/api/tauri.ts`：typed command/event client。
- `src/stores/tabs.ts`：`TabState`、历史栈、视图、排序、搜索和滚动位置。
- `src/stores/tasks.ts`：任务状态、冲突、任务面板。
- `src/stores/settings.ts`：设置加载、更新确认、失败回滚。
- `src/stores/selection.ts`：单选、Ctrl 多选、Shift 范围、多选集合和 inline rename。
- `src/components/surfaces/GlassSurface.tsx`：统一材质组件。
- `src/components/shell/AppShell.tsx`：窗口背景、布局、window chrome 协调。
- `src/components/window/WindowChrome.tsx`：自定义标题栏、按钮热区、拖拽区域。
- `src/components/tabs/TabBar.tsx`：基础标签视觉框架和 V0.8 完整行为。
- `src/components/sidebar/Sidebar.tsx`：快速访问、常用目录、磁盘。
- `src/components/navigation/NavigationBar.tsx`、`Breadcrumb.tsx`、`PathInput.tsx`：路径导航。
- `src/components/toolbar/Toolbar.tsx`：视图、排序、过滤、搜索入口。
- `src/components/files/FileBrowser.tsx`、`FileGrid.tsx`、`FileList.tsx`、`DetailsTable.tsx`：虚拟列表和三种视图。
- `src/components/files/InlineRename.tsx`、`SelectionLayer.tsx`：选择与重命名。
- `src/components/menus/ContextMenu.tsx`：右键菜单。
- `src/components/dialogs/ConflictDialog.tsx`、`DeleteConfirmDialog.tsx`、`PropertiesDialog.tsx`：弹窗。
- `src/components/tasks/TaskPanel.tsx`：任务摘要和展开详情。
- `src/components/settings/SettingsView.tsx`：设置页。
- `src/test/*.test.ts(x)`：前端状态和组件测试。
- `e2e/*.spec.ts`：浏览器 / 桌面用户路径验证。

## 阶段级 e2e 验证总览

| 阶段 | 覆盖范围 | 验证类型 | 启动命令 | 验证命令或操作 | 预期结果 |
| --- | --- | --- | --- | --- | --- |
| V0.0 | 工程骨架 | CLI + build | `npm install` | `npm run check:all` | Rust/TS 测试与构建脚本可运行 |
| V0.1 | 浏览骨架 | 全栈路径 | `npm run tauri:dev:test` | `npm run e2e -- e2e/navigation.spec.ts` | 启动窗口，进入夹具目录，侧边栏/路径/面包屑可导航 |
| V0.2 | 视图排序过滤 | 前端浏览器操作 + Rust 请求 | `npm run tauri:dev:test` | `npm run e2e -- e2e/view-sort-filter.spec.ts` | 三种视图、排序、过滤、隐藏文件/扩展名切换正确 |
| V0.3 | 性能底座 | 全栈 + 性能采样 | `npm run tauri:dev:test` | `npm run e2e -- e2e/performance.spec.ts` | 10k 目录首屏可交互，滚动期间 UI 不被后台任务拖垮 |
| V0.4 | 搜索 | 全栈路径 | `npm run tauri:dev:test` | `npm run e2e -- e2e/search.spec.ts` | 当前目录搜索、递归开关、取消、打开所在位置可用 |
| V0.5 | 文件操作基础 | Rust 真实 FS + 桌面操作 | `npm run tauri:dev:test` | `npm run e2e -- e2e/basic-file-ops.spec.ts` | 新建、重命名、回收站、永久删除确认、打开、属性均走 Rust |
| V0.6 | 复制移动闭环 | Rust 真实 FS + 桌面操作 | `npm run tauri:dev:test` | `npm run e2e -- e2e/copy-move.spec.ts` | 复制、剪切、粘贴、拖拽移动/复制、任务进度和失败恢复可验证 |
| V0.7 | 冲突与多选 | 全栈路径 | `npm run tauri:dev:test` | `npm run e2e -- e2e/conflict-selection.spec.ts` | 替换、跳过、保留两者、应用于全部、多选、右键菜单正确 |
| V0.8 | 多标签 | 全栈路径 | `npm run tauri:dev:test` | `npm run e2e -- e2e/tabs.spec.ts` | 标签独立路径/历史/视图/排序/搜索，后台任务不污染前台 |
| V0.9 | 缩略图图标 | 全栈 + cache 断言 | `npm run tauri:dev:test` | `npm run e2e -- e2e/thumbnails.spec.ts` | 图片缩略图异步加载、缓存命中、滚动延迟和图标 fallback 可用 |
| V1.0 | 商业首发打磨 | 桌面 + package smoke | `npm run tauri:build` | `npm run smoke:bundle` | 生成可启动二进制，设置/主题/错误/崩溃日志/自动更新入口可验收 |

## 通用实施规则

- 每个版本开始前运行 `git status --short`，确认没有未理解的脏改动。
- 每个版本结束前运行 `npm run check:all`，并运行该版本对应 e2e。
- 所有破坏性文件操作测试只允许使用 `scripts/create-fixtures.ps1` 生成的测试根目录。
- 测试模式必须设置 `RUSTFILES_TEST_MODE=1` 和 `RUSTFILES_TEST_ROOT=<absolute fixture root>`。
- 任何危险 command 缺少测试根 guard、确认令牌或人工审核记录时，不允许合入。
- 单元测试、lint、typecheck 和 build 是必要条件；阶段完成必须包含真实运行层面的 e2e 证据。

## Task 0.1: 建立工程脚手架与基础命令（已完成）

**Files:**
- Create: `package.json`
- Create: `index.html`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `tsconfig.node.json`
- Create: `playwright.config.ts`
- Create: `src/main.tsx`
- Create: `src/App.tsx`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Test: `src/test/app-smoke.test.tsx`

- [x] **Step 1: 写最小失败测试**

```tsx
// src/test/app-smoke.test.tsx
import { render, screen } from "@testing-library/react";
import App from "../App";

it("renders the RustFiles shell", () => {
  render(<App />);
  expect(screen.getByRole("application", { name: "RustFiles" })).toBeInTheDocument();
});
```

- [x] **Step 2: 运行测试确认失败**

Run: `npm install && npm run test -- src/test/app-smoke.test.tsx`

Expected: 测试失败，提示 `Unable to find role="application"` 或依赖脚本尚未存在。

- [x] **Step 3: 写最小实现**

创建 React/Vite/Tauri 基础文件。`App.tsx` 暂时只渲染带 `role="application"` 和 `aria-label="RustFiles"` 的 App Shell 占位。`src-tauri/src/lib.rs` 注册空 Tauri builder，暂不注册危险 command。`package.json` 至少包含：

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc -b && vite build",
    "test": "vitest run",
    "typecheck": "tsc -b --noEmit",
    "tauri:dev": "tauri dev",
    "tauri:dev:test": "powershell -ExecutionPolicy Bypass -File scripts/run-tauri-dev-test.ps1",
    "tauri:build": "tauri build",
    "e2e": "playwright test",
    "check:all": "npm run typecheck && npm run test && cargo test --manifest-path src-tauri/Cargo.toml && npm run build"
  }
}
```

- [x] **Step 4: 运行测试确认通过**

Run: `npm run check:all`

Expected: TypeScript、Vitest、Cargo test、Vite build 全部退出码为 0。

- [x] **Step 5: 运行最小桌面验证**

Run: `npm run tauri:dev:test`

Expected: Tauri dev 窗口可启动，窗口中可见 RustFiles shell，占位 UI 无控制台错误。

- [x] **Step 6: Commit**

```bash
git add package.json index.html vite.config.ts tsconfig.json tsconfig.node.json playwright.config.ts src src-tauri
git commit -m "chore: scaffold RustFiles Tauri app"
```

## Task 0.2: 建立共享类型、错误模型与测试夹具工具（已完成）

**Files:**
- Create: `src-tauri/src/core/mod.rs`
- Create: `src-tauri/src/core/types.rs`
- Create: `src-tauri/src/core/error.rs`
- Create: `src-tauri/src/core/observability.rs`
- Create: `scripts/create-fixtures.ps1`
- Create: `scripts/run-tauri-dev-test.ps1`
- Test: `src-tauri/tests/types_contract.rs`
- Test: `src-tauri/tests/fixture_generation.rs`

- [x] **Step 1: 写失败测试**

在 `types_contract.rs` 断言 `TaskStatus` 包含 `queued`、`validating`、`running`、`waiting_for_conflict_decision`、`cancelling`、`cancelled`、`completed`、`failed`、`partially_completed`，并能序列化为前端契约字符串。

- [x] **Step 2: 运行测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml types_contract`

Expected: 编译失败或缺少 `TaskStatus` 类型。

- [x] **Step 3: 写最小实现**

实现 `AppError`、`ErrorCode`、`FileEntry`、`DirectoryPage`、`FileTask`、`TaskStatus`、`Settings`、`ViewMode`、`SortKey`、`FilterKind`、`ConflictDecision`。所有类型派生 `Serialize`、`Deserialize`、`Clone`、`Debug`；`Settings` 必须包含 `schema_version`。

- [x] **Step 4: 添加夹具生成脚本**

`scripts/create-fixtures.ps1` 接收 `-Root <path>`，生成：

- `small-dir`
- `large-10k-dir`
- `media-dir`
- `conflict-source`
- `conflict-target`
- `deep-tree`
- `permission-cases`

脚本必须拒绝空路径、用户 profile 根、桌面、下载、文档、图片、视频、音乐目录。

- [x] **Step 5: 运行验证**

Run: `powershell -ExecutionPolicy Bypass -File scripts/create-fixtures.ps1 -Root .\.tmp\fixtures`

Expected: 生成所有夹具目录；`large-10k-dir` 文件数为 10000；脚本对真实用户常用目录返回非 0。

- [x] **Step 6: Commit**

```bash
git add src-tauri/src/core scripts src-tauri/tests
git commit -m "chore: add core contracts and fixtures"
```

## Task 0.3: 锁定 Tauri command 白名单和测试模式 guard 外壳（已完成）

**Files:**
- Modify: `src-tauri/capabilities/default.json`
- Modify: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/commands.rs`
- Create: `src-tauri/src/core/runtime.rs`
- Test: `src-tauri/tests/command_whitelist.rs`

- [x] **Step 1: 写失败测试**

测试读取 `src-tauri/capabilities/default.json`，断言只暴露架构文档列出的 command 名称；危险 command 名称必须在 `dangerous_commands` 测试集合中。

- [x] **Step 2: 运行测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml command_whitelist`

Expected: 缺少 capability 或命令集合不匹配。

- [x] **Step 3: 写最小实现**

创建 command adapter 空实现，所有 command 返回 `not_implemented` 结构化错误。危险 command adapter 必须检查测试模式和确认字段，但此阶段不执行真实文件操作。

- [x] **Step 4: 运行验证**

Run: `cargo test --manifest-path src-tauri/Cargo.toml command_whitelist`

Expected: 白名单匹配；危险 command 缺少确认字段时返回 `confirmation_required`。

- [x] **Step 5: 运行 e2e smoke**

Run: `npm run tauri:dev:test`

Expected: 应用仍能启动，前端没有直接文件系统写权限相关配置。

- [x] **Step 6: Commit**

```bash
git add src-tauri/capabilities/default.json src-tauri/src/lib.rs src-tauri/src/commands.rs src-tauri/src/core/runtime.rs src-tauri/tests
git commit -m "chore: lock Tauri command boundary"
```

## Task 1.1: 实现路径安全模块（已完成）

**Files:**
- Create: `src-tauri/src/core/path_safety.rs`
- Test: `src-tauri/tests/path_safety.rs`

- [x] **Step 1: 写失败测试**

覆盖 Windows 保留名、非法字符、尾随空格/点、长路径、大小写冲突、symlink、junction、reparse point、UNC、subst/network 分类和测试根逃逸。

- [x] **Step 2: 运行测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml path_safety`

Expected: 编译失败或安全决策函数缺失。

- [x] **Step 3: 写最小实现**

实现 `normalize_path`、`classify_path`、`validate_child_name`、`guard_destructive_path`、`guard_test_root_after_reparse_resolution`。破坏性递归操作默认不得跟随 reparse point 到目标树。

- [x] **Step 4: 运行测试确认通过**

Run: `cargo test --manifest-path src-tauri/Cargo.toml path_safety`

Expected: 所有路径安全用例通过；当前系统无法创建某类 reparse point 时，测试必须显式 `ignored` 并记录原因，不得假装通过。

- [x] **Step 5: 运行测试根逃逸 e2e**

Run: `powershell -ExecutionPolicy Bypass -File scripts/create-fixtures.ps1 -Root .\.tmp\fixtures; cargo test --manifest-path src-tauri/Cargo.toml test_root_escape_is_rejected -- --nocapture`

Expected: 指向测试根外部的 symlink/junction destructive operation 返回 `test_root_escape`。

- [x] **Step 6: Commit**

```bash
git add src-tauri/src/core/path_safety.rs src-tauri/tests/path_safety.rs
git commit -m "feat: add Windows path safety guards"
```

## Task 1.2: 实现只读目录枚举和侧边栏 roots（已完成）

**Files:**
- Create: `src-tauri/src/core/fs.rs`
- Create: `src-tauri/src/core/system.rs`
- Modify: `src-tauri/src/commands.rs`
- Test: `src-tauri/tests/fs_listing.rs`
- Test: `src-tauri/tests/sidebar_roots.rs`

- [x] **Step 1: 写失败测试**

测试 `list_directory` 对 `small-dir` 返回 `DirectoryPage`，包含路径、条目、总数、排序、过滤、隐藏文件设置、`snapshot_version`。测试 `get_sidebar_roots` 返回桌面、下载、文档、图片、视频、音乐、此电脑和磁盘分区。

- [x] **Step 2: 运行测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml fs_listing sidebar_roots`

Expected: command 或模块未实现。

- [x] **Step 3: 写最小实现**

`fs` 只读枚举目录，不执行写操作。`system` 读取常用目录和磁盘分区，失败时返回结构化错误但不阻塞应用启动。

- [x] **Step 4: 运行测试确认通过**

Run: `cargo test --manifest-path src-tauri/Cargo.toml fs_listing sidebar_roots`

Expected: `small-dir` 枚举成功；无权限路径返回 `permission_denied`；不存在路径返回 `path_not_found`。

- [x] **Step 5: 运行 command 级验证**

Run: `cargo test --manifest-path src-tauri/Cargo.toml list_directory_command_returns_page -- --nocapture`

Expected: command adapter 返回 JSON 可反序列化为 `DirectoryPage`。

- [x] **Step 6: Commit**

```bash
git add src-tauri/src/core/fs.rs src-tauri/src/core/system.rs src-tauri/src/commands.rs src-tauri/tests
git commit -m "feat: list local directories"
```

## Task 1.3: 实现主界面浏览骨架（已完成）

**Files:**
- Create: `src/api/tauri.ts`
- Create: `src/stores/tabs.ts`
- Create: `src/components/shell/AppShell.tsx`
- Create: `src/components/window/WindowChrome.tsx`
- Create: `src/components/sidebar/Sidebar.tsx`
- Create: `src/components/navigation/NavigationBar.tsx`
- Create: `src/components/navigation/Breadcrumb.tsx`
- Create: `src/components/files/FileBrowser.tsx`
- Modify: `src/App.tsx`
- Test: `src/test/navigation-state.test.ts`
- Test: `e2e/navigation.spec.ts`

- [x] **Step 1: 写失败测试**

`navigation-state.test.ts` 验证进入目录、返回、前进、路径输入和面包屑点击会更新当前 tab 的 path/history。`navigation.spec.ts` 打开应用后点击侧边栏夹具目录、双击子文件夹、点击返回。

- [x] **Step 2: 运行测试确认失败**

Run: `npm run test -- src/test/navigation-state.test.ts && npm run e2e -- e2e/navigation.spec.ts`

Expected: 前端组件或 e2e 目标缺失。

- [x] **Step 3: 写最小实现**

实现 App Shell、Sidebar、NavigationBar、Breadcrumb、FileBrowser。默认单标签，但 `tabs.ts` 必须使用 `TabState` 结构，为 V0.8 保留状态框架。

- [x] **Step 4: 运行测试确认通过**

Run: `npm run test -- src/test/navigation-state.test.ts`

Expected: tab 导航状态测试通过。

- [x] **Step 5: 运行 V0.1 e2e**

Run: `npm run tauri:dev:test`，另一个终端运行 `npm run e2e -- e2e/navigation.spec.ts`

Expected: 用户可从默认目录进入夹具目录，路径栏、面包屑、返回/前进和文件列表同步；无布局跳动。

- [x] **Step 6: Commit**

```bash
git add src e2e
git commit -m "feat: add browsing shell"
```

## Task 1.4: 修稳验证链路与非 Tauri 降级浏览体验（已完成）

**Files:**
- Modify: `package.json`
- Modify: `vite.config.ts`
- Modify: `src/components/shell/AppShell.tsx`
- Modify: `src/components/sidebar/Sidebar.tsx`
- Modify: `src/components/files/FileBrowser.tsx`
- Test: `src/test/runtime-fallback.test.tsx`
- Test: `e2e/navigation.spec.ts`

- [x] **Step 1: 写失败测试与回归断言**

新增 `runtime-fallback.test.tsx`，覆盖纯浏览器环境下的 App Shell 降级行为：`This PC` 这类非文件系统路径不会触发误导性的“目录加载失败”语义；Drives 区域在没有 Tauri runtime 时显示明确的桌面运行时提示，而不是把“没有驱动器”当成真实系统状态；加载文案必须是可读的正常字符串，不能出现编码污染。同步补充 `navigation.spec.ts`，让现有导航 e2e 在浏览器预览模式下也断言关键降级提示存在。

- [x] **Step 2: 运行回归，确认当前实现真实失败**

Run: `npm run check:all`

Expected: `vitest run` 误收 `e2e/navigation.spec.ts`，出现 “Playwright Test did not expect test() to be called here” 并导致聚合检查失败。

Run: `npm run test -- src/test/runtime-fallback.test.tsx`

Expected: 断言失败，暴露当前浏览器预览模式仍显示 `No drives available` 或存在 `Loading directory...` 文案编码异常。

- [x] **Step 3: 写最小实现**

在 `vite.config.ts` 中显式收窄 Vitest 的测试收集范围，只运行 `src/test/` 下的单元/组件测试，并排除 `e2e/**`，保证 `npm run check:all` 不再把 Playwright spec 当成 Vitest suite。`AppShell.tsx` 负责向 Sidebar/FileBrowser 传递当前是否运行在 Tauri runtime 的明确信号；`Sidebar.tsx` 在浏览器预览模式下显示“驱动器仅在桌面运行时加载”之类的明确提示；`FileBrowser.tsx` 修正编码污染的 loading 文案，并为 `This PC` 这类占位路径显示清晰的预览态空状态说明，而不是看起来像真实文件系统枚举结果。

- [x] **Step 4: 运行验证，确认检查链路恢复**

Run: `npm run check:all`

Expected: TypeScript、Vitest、Cargo test、Vite build 全部退出码为 0，且 Vitest 不再执行 `e2e/navigation.spec.ts`。

- [x] **Step 5: 运行阶段 e2e 与人工预览验收**

Run: `npm run tauri:dev:test`

Expected: Tauri smoke 继续通过，新增提示文案不会破坏桌面端启动。

Run: `npm run dev -- --host 127.0.0.1 --port 1420`，另一个终端运行 `npm run e2e -- e2e/navigation.spec.ts`

Expected: 浏览器导航 e2e 通过；纯浏览器预览模式下能看到明确的降级提示，Drives 区域不再误导性显示真实系统“无驱动器”，页面文案不存在 mojibake。

- [x] **Step 6: Commit**

```bash
git add package.json vite.config.ts src/components/shell/AppShell.tsx src/components/sidebar/Sidebar.tsx src/components/files/FileBrowser.tsx src/test/runtime-fallback.test.tsx e2e/navigation.spec.ts
git commit -m "fix: stabilize validation pipeline and browser fallback shell"
```

## Task 2.1: 实现排序、过滤、隐藏文件和扩展名显示设置（已完成）

**Files:**
- Modify: `src-tauri/src/core/fs.rs`
- Create: `src-tauri/src/core/settings.rs`
- Modify: `src-tauri/src/commands.rs`
- Create: `src/stores/settings.ts`
- Create: `src/components/toolbar/Toolbar.tsx`
- Test: `src-tauri/tests/sort_filter.rs`
- Test: `src-tauri/tests/settings.rs`
- Test: `src/test/settings-store.test.ts`
- Test: `e2e/view-sort-filter.spec.ts`

- [x] **Step 1: 写失败测试**

Rust 测试固定文件大小和 mtime 后，验证名称、修改时间、大小、类型排序；过滤文件、文件夹、图片、视频、文档；隐藏文件开关。前端测试验证设置必须等 Rust 确认后才显示持久化成功。

- [x] **Step 2: 运行测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml sort_filter settings && npm run test -- src/test/settings-store.test.ts`

Expected: 排序、过滤或 settings command 缺失。

- [x] **Step 3: 写最小实现**

实现 `Settings` 原子写入、`schema_version`、轻量设置即时保存、失败回滚。`fs` 根据 `SortKey`、`FilterKind` 和 settings 应用排序过滤。

- [x] **Step 4: 运行测试确认通过**

Run: `npm run check:all`

Expected: Rust/TS 测试、类型检查、构建通过。

- [x] **Step 5: 运行 V0.2 e2e**

Run: `npm run e2e -- e2e/view-sort-filter.spec.ts`

Expected: 三种排序和五类过滤可见生效；隐藏文件/扩展名设置保存后刷新仍生效。

- [x] **Step 6: Commit**

```bash
git add src-tauri/src/core/fs.rs src-tauri/src/core/settings.rs src-tauri/src/commands.rs src src-tauri/tests e2e
git commit -m "feat: add sorting filtering and settings persistence"
```

## Task 2.2: 实现三种文件视图和统一 Liquid Glass token

**Files:**
- Create: `src/styles/tokens.css`
- Create: `src/styles/global.css`
- Create: `src/components/surfaces/GlassSurface.tsx`
- Create: `src/components/files/FileGrid.tsx`
- Create: `src/components/files/FileList.tsx`
- Create: `src/components/files/DetailsTable.tsx`
- Modify: `src/components/files/FileBrowser.tsx`
- Test: `src/test/view-mode.test.tsx`
- Test: `e2e/glass-readability.spec.ts`

- [ ] **Step 1: 写失败测试**

组件测试验证切换图标、列表、详情视图后保留选择和路径；e2e 截图检查浅色、深色、降低透明度下文件名可见，行高和网格 cell 不变化。

- [ ] **Step 2: 运行测试确认失败**

Run: `npm run test -- src/test/view-mode.test.tsx`

Expected: 视图组件缺失。

- [ ] **Step 3: 写最小实现**

定义 `bg.window`、`surface.chrome`、`surface.content`、`surface.floating`、`surface.solidSafety`、`text.*`、`border.*`、`glass.*`、`z-index` token。实现 `GlassSurface` 的 `chrome`、`content`、`floating`、`solid-safety` 变体。

- [ ] **Step 4: 运行测试确认通过**

Run: `npm run test -- src/test/view-mode.test.tsx && npm run typecheck`

Expected: 视图状态和类型检查通过。

- [ ] **Step 5: 运行视觉 e2e**

Run: `npm run e2e -- e2e/glass-readability.spec.ts`

Expected: 文件列表文本可读，forced-colors / reduced-transparency fallback 生效，视图切换不引发布局跳动。

- [ ] **Step 6: Commit**

```bash
git add src/styles src/components/surfaces src/components/files src/test e2e
git commit -m "feat: add file views and glass design tokens"
```

## Task 3.1: 引入虚拟列表与大目录分页快照

**Files:**
- Modify: `src-tauri/src/core/fs.rs`
- Create: `src-tauri/src/core/snapshot.rs`
- Modify: `src/components/files/FileGrid.tsx`
- Modify: `src/components/files/FileList.tsx`
- Modify: `src/components/files/DetailsTable.tsx`
- Test: `src-tauri/tests/large_directory.rs`
- Test: `src/test/virtual-list.test.tsx`
- Test: `e2e/performance.spec.ts`

- [ ] **Step 1: 写失败测试**

Rust 测试对 `large-10k-dir` 请求首批窗口，断言不一次性阻塞返回全部重元数据。前端测试断言 DOM 中只渲染可视范围附近的 item。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml large_directory && npm run test -- src/test/virtual-list.test.tsx`

Expected: 分页快照或虚拟列表未实现。

- [ ] **Step 3: 写最小实现**

`DirectoryPage` 返回分页窗口、总数和 `snapshot_version`。前端使用 TanStack Virtual，固定详情行高、列表行高和图标网格 cell 尺寸。

- [ ] **Step 4: 运行测试确认通过**

Run: `npm run check:all`

Expected: 10k 夹具测试通过，虚拟列表测试通过。

- [ ] **Step 5: 运行 V0.3 大目录 e2e**

Run: `npm run e2e -- e2e/performance.spec.ts`

Expected: 10k 目录首屏可滚动可选择；连续滚动期间 DOM 节点数量保持有限；无明显白屏。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/core/fs.rs src-tauri/src/core/snapshot.rs src/components/files src-tauri/tests src/test e2e
git commit -m "feat: virtualize large directories"
```

## Task 3.2: 实现 UI 优先调度信号与性能采样

**Files:**
- Create: `src-tauri/src/core/scheduler.rs`
- Modify: `src-tauri/src/core/observability.rs`
- Modify: `src-tauri/src/commands.rs`
- Create: `src/hooks/useViewportReporting.ts`
- Create: `src/hooks/useInteractionReporting.ts`
- Test: `src-tauri/tests/scheduler.rs`
- Test: `src/test/interaction-reporting.test.ts`
- Test: `e2e/ui-priority.spec.ts`

- [ ] **Step 1: 写失败测试**

Rust 测试给出 `active_tab_id`、`visible_range`、`last_input_at`、`is_scrolling`、`interaction_epoch` 后，断言优先级顺序为前台目录枚举、可视缩略图、前台搜索、文件进度、后台刷新、非可视缩略图、后台递归搜索。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml scheduler && npm run test -- src/test/interaction-reporting.test.ts`

Expected: scheduler 或前端上报 hook 缺失。

- [ ] **Step 3: 写最小实现**

实现 `report_viewport_state`、`report_interaction_state` command。前端滚动、选择、路径跳转、标签切换时上报；Rust 对 `thumbnail_ready`、`search_result_batch` 做批量事件背压。

- [ ] **Step 4: 运行测试确认通过**

Run: `npm run check:all`

Expected: scheduler 和上报测试通过。

- [ ] **Step 5: 运行 UI 优先 e2e**

Run: `npm run e2e -- e2e/ui-priority.spec.ts`

Expected: 模拟后台扫描/缩略图压力时，前台滚动、选择、右键、路径跳转仍能响应；性能采样日志出现 `interaction_latency_ms` 和 `frame_budget_degraded`。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/core/scheduler.rs src-tauri/src/core/observability.rs src-tauri/src/commands.rs src/hooks src-tauri/tests src/test e2e
git commit -m "feat: prioritize visible UI work"
```

## Task 4.1: 实现当前目录搜索和递归搜索任务

**Files:**
- Create: `src-tauri/src/core/search.rs`
- Modify: `src-tauri/src/core/tasks.rs`
- Modify: `src-tauri/src/commands.rs`
- Create: `src/stores/search.ts`
- Modify: `src/components/toolbar/Toolbar.tsx`
- Test: `src-tauri/tests/search.rs`
- Test: `src/test/search-store.test.ts`
- Test: `e2e/search.spec.ts`

- [ ] **Step 1: 写失败测试**

Rust 测试覆盖非递归搜索、递归搜索、取消、无权限目录跳过并返回错误摘要。前端测试覆盖输入防抖、实时结果、清除搜索、打开所在位置。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml search && npm run test -- src/test/search-store.test.ts`

Expected: search module 和 store 缺失。

- [ ] **Step 3: 写最小实现**

非递归搜索优先使用当前 `DirectoryPage` 快照；递归搜索进入可取消后台任务，服从 scheduler，结果通过 `search_result_batch` 批量发送。

- [ ] **Step 4: 运行测试确认通过**

Run: `npm run check:all`

Expected: 搜索单元测试和类型检查通过。

- [ ] **Step 5: 运行 V0.4 e2e**

Run: `npm run e2e -- e2e/search.spec.ts`

Expected: 搜索文件名实时过滤；递归搜索可取消；打开所在位置导航到正确目录；目标消失时显示“项目已不存在或已移动”并刷新。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/core/search.rs src-tauri/src/core/tasks.rs src-tauri/src/commands.rs src/stores/search.ts src/components/toolbar src-tauri/tests src/test e2e
git commit -m "feat: add current directory search"
```

## Task 5.1: 实现任务状态机和任务面板

**Files:**
- Create: `src-tauri/src/core/tasks.rs`
- Create: `src/stores/tasks.ts`
- Create: `src/components/tasks/TaskPanel.tsx`
- Test: `src-tauri/tests/task_state_machine.rs`
- Test: `src/test/task-panel.test.tsx`

- [ ] **Step 1: 写失败测试**

测试所有合法状态迁移和非法迁移拒绝；前端任务面板测试覆盖摘要态、展开态、冲突等待、取消中、失败和部分完成。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml task_state_machine && npm run test -- src/test/task-panel.test.tsx`

Expected: 状态机或任务面板缺失。

- [ ] **Step 3: 写最小实现**

Rust 只允许任务状态由 tasks 模块推进。终态必须包含结果摘要；`partially_completed` 必须列出已完成、未完成和未知项。

- [ ] **Step 4: 运行测试确认通过**

Run: `npm run check:all`

Expected: 状态机和任务面板测试通过。

- [ ] **Step 5: 运行任务面板 e2e**

Run: `npm run e2e -- e2e/task-panel.spec.ts`

Expected: 模拟任务事件时，摘要态不长期挤压文件列表，展开态可查看详情和取消。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/core/tasks.rs src/stores/tasks.ts src/components/tasks src-tauri/tests src/test e2e
git commit -m "feat: add file task state machine"
```

## Task 5.2: 实现新建文件夹、重命名、删除和系统打开

**Files:**
- Modify: `src-tauri/src/core/tasks.rs`
- Modify: `src-tauri/src/core/system.rs`
- Modify: `src-tauri/src/commands.rs`
- Create: `src/components/files/InlineRename.tsx`
- Create: `src/components/dialogs/DeleteConfirmDialog.tsx`
- Create: `src/components/dialogs/PropertiesDialog.tsx`
- Test: `src-tauri/tests/basic_file_ops.rs`
- Test: `src-tauri/tests/system_integration.rs`
- Test: `src/test/inline-rename.test.tsx`
- Test: `e2e/basic-file-ops.spec.ts`

- [ ] **Step 1: 写失败测试**

Rust 测试在夹具根中验证新建文件夹、重命名、删除到回收站、永久删除确认令牌、默认应用打开、终端打开当前目录和属性错误码。前端测试覆盖 F2、Enter、Escape、非法字符、保留名、重名错误。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml basic_file_ops system_integration && npm run test -- src/test/inline-rename.test.tsx`

Expected: 文件操作 command 或 inline rename 缺失。

- [ ] **Step 3: 写最小实现**

所有真实写操作进入 `tasks`。删除到回收站失败不得降级为永久删除。永久删除必须由前端显示二次确认，并由 Rust 校验确认令牌。

- [ ] **Step 4: 运行测试确认通过**

Run: `npm run check:all`

Expected: 文件系统夹具中的最终状态符合断言；错误码结构化。

- [ ] **Step 5: 运行 V0.5 e2e**

Run: `npm run e2e -- e2e/basic-file-ops.spec.ts`

Expected: 用户可新建、行内重命名、删除到回收站、取消永久删除、确认永久删除、打开文件、打开终端和查看属性；所有破坏性操作只作用于测试根。

- [ ] **Step 6: 人工审核**

检查永久删除、回收站失败、重命名覆盖风险和测试根 guard 证据。审核通过后继续。

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/core/tasks.rs src-tauri/src/core/system.rs src-tauri/src/commands.rs src/components/files/InlineRename.tsx src/components/dialogs src-tauri/tests src/test e2e
git commit -m "feat: add basic file operations"
```

## Task 6.1: 实现复制、剪切、粘贴 pending operation

**Files:**
- Create: `src-tauri/src/core/clipboard.rs`
- Modify: `src-tauri/src/core/tasks.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src/stores/selection.ts`
- Test: `src-tauri/tests/clipboard_ops.rs`
- Test: `src/test/clipboard-ui.test.ts`
- Test: `e2e/copy-paste.spec.ts`

- [ ] **Step 1: 写失败测试**

Rust 测试验证 `create_clipboard_operation` 保存源路径、类型、来源标签、创建时间和路径安全分类；`paste_clipboard_operation` 只接收 operation id 和目标目录；源消失时返回结构化错误。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml clipboard_ops && npm run test -- src/test/clipboard-ui.test.ts`

Expected: clipboard 模块或剪切视觉态缺失。

- [ ] **Step 3: 写最小实现**

React 可以显示 cut-pending；Rust 持有真正可执行的 pending operation。粘贴时 Rust 重新校验源、目标、reparse point 和测试根。

- [ ] **Step 4: 运行测试确认通过**

Run: `npm run check:all`

Expected: clipboard operation 测试通过。

- [ ] **Step 5: 运行 e2e**

Run: `npm run e2e -- e2e/copy-paste.spec.ts`

Expected: Ctrl+C/Ctrl+X/Ctrl+V 可复制和移动夹具文件；剪切项显示降低不透明度；源消失时显示可恢复错误。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/core/clipboard.rs src-tauri/src/core/tasks.rs src-tauri/src/commands.rs src/stores src-tauri/tests src/test e2e
git commit -m "feat: add app clipboard operations"
```

## Task 6.2: 实现拖拽移动/复制与进度反馈

**Files:**
- Create: `src-tauri/src/core/drag.rs`
- Modify: `src-tauri/src/core/tasks.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src/components/files/FileBrowser.tsx`
- Modify: `src/components/tasks/TaskPanel.tsx`
- Test: `src-tauri/tests/drag_ops.rs`
- Test: `src-tauri/tests/copy_move_progress.rs`
- Test: `e2e/copy-move.spec.ts`

- [ ] **Step 1: 写失败测试**

Rust 测试验证同卷拖拽默认移动、跨卷拖拽默认复制、显式修饰键切换语义、跨卷移动复制成功后再删除源。进度测试验证任务事件包含总数、已完成、速度和当前阶段。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml drag_ops copy_move_progress`

Expected: drag module 或进度事件缺失。

- [ ] **Step 3: 写最小实现**

React 拖拽开始创建 drag operation id；drop 时提交目标目录和语义。Rust 执行前重新校验源/目标/卷信息/reparse point/冲突状态。

- [ ] **Step 4: 运行测试确认通过**

Run: `npm run check:all`

Expected: 复制移动和拖拽测试通过。

- [ ] **Step 5: 运行 V0.6 e2e**

Run: `npm run e2e -- e2e/copy-move.spec.ts`

Expected: 用户可拖拽移动/复制文件夹和文件；任务面板显示进度；取消后进入 `cancelled` 或 `partially_completed`，不出现源文件丢失。

- [ ] **Step 6: 人工审核**

审核跨卷移动、取消、部分完成和拖拽语义。审核通过后继续。

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/core/drag.rs src-tauri/src/core/tasks.rs src-tauri/src/commands.rs src/components src-tauri/tests e2e
git commit -m "feat: add copy move and drag operations"
```

## Task 7.1: 实现冲突处理流程

**Files:**
- Modify: `src-tauri/src/core/tasks.rs`
- Modify: `src-tauri/src/commands.rs`
- Create: `src/components/dialogs/ConflictDialog.tsx`
- Test: `src-tauri/tests/conflict_resolution.rs`
- Test: `src/test/conflict-dialog.test.tsx`
- Test: `e2e/conflict-selection.spec.ts`

- [ ] **Step 1: 写失败测试**

Rust 测试覆盖替换、跳过、保留两者、应用于全部、批量同类型冲突、取消冲突处理。前端测试验证危险操作不默认聚焦，应用于全部是独立 checkbox/toggle，保留两者显示新名称预览。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml conflict_resolution && npm run test -- src/test/conflict-dialog.test.tsx`

Expected: 冲突状态或弹窗缺失。

- [ ] **Step 3: 写最小实现**

冲突发生时任务进入 `waiting_for_conflict_decision`。关闭弹窗不等价于继续危险操作；取消进入 `cancelling`，最终为 `cancelled` 或 `partially_completed`。

- [ ] **Step 4: 运行测试确认通过**

Run: `npm run check:all`

Expected: 冲突四路径和批量流程测试通过。

- [ ] **Step 5: 运行 V0.7 冲突 e2e**

Run: `npm run e2e -- e2e/conflict-selection.spec.ts`

Expected: 替换、跳过、保留两者、应用于全部均改变夹具文件状态；取消冲突后任务状态可见且无默认覆盖。

- [ ] **Step 6: 人工审核**

审核覆盖、保留两者命名、应用于全部范围和部分完成摘要。审核通过后继续。

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/core/tasks.rs src-tauri/src/commands.rs src/components/dialogs/ConflictDialog.tsx src-tauri/tests src/test e2e
git commit -m "feat: add conflict resolution flow"
```

## Task 7.2: 实现多选、右键菜单和快捷键

**Files:**
- Create: `src/stores/selection.ts`
- Create: `src/components/menus/ContextMenu.tsx`
- Create: `src/hooks/useFileShortcuts.ts`
- Modify: `src/components/files/FileBrowser.tsx`
- Test: `src/test/selection.test.ts`
- Test: `src/test/context-menu.test.tsx`
- Test: `e2e/selection-shortcuts.spec.ts`

- [ ] **Step 1: 写失败测试**

测试单击、Ctrl 单击、Shift 范围、Ctrl+A、空白区清除、右键未选中项先选中、右键已选中项保留多选。快捷键覆盖 F2、Delete、Shift+Delete、Ctrl+C/X/V、Ctrl+L、Alt+Left/Right、Ctrl+T/W、F5、Escape。

- [ ] **Step 2: 运行测试确认失败**

Run: `npm run test -- src/test/selection.test.ts src/test/context-menu.test.tsx`

Expected: selection store 或 context menu 缺失。

- [ ] **Step 3: 写最小实现**

实现选择模型、右键菜单顺序、危险分组、键盘导航和快捷键路由。inline rename、地址栏和搜索输入内，文字编辑快捷键优先于全局快捷键。

- [ ] **Step 4: 运行测试确认通过**

Run: `npm run check:all`

Expected: 选择、菜单和快捷键测试通过。

- [ ] **Step 5: 运行 e2e**

Run: `npm run e2e -- e2e/selection-shortcuts.spec.ts`

Expected: 多选不串项；右键多选保留集合；快捷键触发同一 Rust 任务路径；危险快捷键仍显示确认。

- [ ] **Step 6: Commit**

```bash
git add src/stores/selection.ts src/components/menus src/hooks/useFileShortcuts.ts src/components/files src/test e2e
git commit -m "feat: add selection menus and shortcuts"
```

## Task 8.1: 实现完整多标签页

**Files:**
- Modify: `src/stores/tabs.ts`
- Modify: `src/components/tabs/TabBar.tsx`
- Modify: `src/components/files/FileBrowser.tsx`
- Modify: `src/components/menus/ContextMenu.tsx`
- Test: `src/test/tabs.test.ts`
- Test: `e2e/tabs.spec.ts`

- [ ] **Step 1: 写失败测试**

测试新建、关闭、切换标签；每个标签独立维护路径、历史、视图、排序、过滤、搜索和滚动位置；关闭最后一个标签时创建默认标签。

- [ ] **Step 2: 运行测试确认失败**

Run: `npm run test -- src/test/tabs.test.ts`

Expected: 完整 tabs 行为缺失。

- [ ] **Step 3: 写最小实现**

实现 Ctrl+T、Ctrl+W、点击新建/关闭、右键文件夹在新标签打开。后台标签可以保留旧快照，但收到 `directory_changed` 后标记需要刷新。

- [ ] **Step 4: 运行测试确认通过**

Run: `npm run check:all`

Expected: tab store 和 UI 测试通过。

- [ ] **Step 5: 运行 V0.8 e2e**

Run: `npm run e2e -- e2e/tabs.spec.ts`

Expected: 多标签状态隔离；后台标签任务失败不污染当前标签；切回后台标签时刷新提示正确。

- [ ] **Step 6: Commit**

```bash
git add src/stores/tabs.ts src/components/tabs src/components/files src/components/menus src/test e2e
git commit -m "feat: add multi-tab browsing"
```

## Task 8.2: 实现目录变化事件和快照一致性

**Files:**
- Modify: `src-tauri/src/core/fs.rs`
- Modify: `src-tauri/src/core/tasks.rs`
- Modify: `src-tauri/src/core/snapshot.rs`
- Modify: `src/api/tauri.ts`
- Modify: `src/stores/tabs.ts`
- Test: `src-tauri/tests/snapshot_consistency.rs`
- Test: `src/test/directory-changed.test.ts`
- Test: `e2e/snapshot-refresh.spec.ts`

- [ ] **Step 1: 写失败测试**

Rust 测试断言文件操作完成后发送 `directory_changed`，包含 `path`、`old_snapshot_version`、`reason`、`suggested_refresh_scope`。前端测试断言旧事件可丢弃，相关标签标记刷新。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml snapshot_consistency && npm run test -- src/test/directory-changed.test.ts`

Expected: 快照一致性事件缺失。

- [ ] **Step 3: 写最小实现**

排序、过滤、隐藏文件设置变化生成新快照；文件操作刷新源目录、目标目录和父目录；搜索结果目标不存在时显示可恢复错误并刷新。

- [ ] **Step 4: 运行测试确认通过**

Run: `npm run check:all`

Expected: 快照一致性测试通过。

- [ ] **Step 5: 运行 e2e**

Run: `npm run e2e -- e2e/snapshot-refresh.spec.ts`

Expected: 两个标签打开同一路径时，一个标签执行重命名/删除，另一个标签收到刷新标记并可刷新到最新内容。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/core src/api src/stores src-tauri/tests src/test e2e
git commit -m "feat: refresh directory snapshots"
```

## Task 9.1: 实现文件类型图标和图片缩略图缓存

**Files:**
- Create: `src-tauri/src/core/thumbnails.rs`
- Modify: `src-tauri/src/core/fs.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src/components/files/FileGrid.tsx`
- Modify: `src/components/files/FileList.tsx`
- Modify: `src/components/files/DetailsTable.tsx`
- Test: `src-tauri/tests/thumbnails.rs`
- Test: `src/test/thumbnail-ui.test.tsx`
- Test: `e2e/thumbnails.spec.ts`

- [ ] **Step 1: 写失败测试**

Rust 测试验证图片缩略图生成、缓存命中、损坏图片 fallback、缓存 key 包含真实路径、mtime、size 和规格、测试缓存目录可覆盖。前端测试验证缩略图占位尺寸稳定。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml thumbnails && npm run test -- src/test/thumbnail-ui.test.tsx`

Expected: thumbnails module 或 UI 状态缺失。

- [ ] **Step 3: 写最小实现**

实现 `request_thumbnails`、`cancel_thumbnail_requests`，可视区域优先，滚动时延迟非可视任务。缩略图失败显示文件类型图标，不阻塞列表。

- [ ] **Step 4: 运行测试确认通过**

Run: `npm run check:all`

Expected: 缩略图生成、缓存、fallback 测试通过。

- [ ] **Step 5: 运行 V0.9 e2e**

Run: `npm run e2e -- e2e/thumbnails.spec.ts`

Expected: 图片缩略图异步出现；第二次进入目录命中缓存；快速滚动时非可视缩略图延迟且布局不跳动。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/core/thumbnails.rs src-tauri/src/core/fs.rs src-tauri/src/commands.rs src/components/files src-tauri/tests src/test e2e
git commit -m "feat: add thumbnails and file icons"
```

## Task 9.2: 完成玻璃性能降级和可访问性验收

**Files:**
- Modify: `src/styles/tokens.css`
- Modify: `src/styles/global.css`
- Modify: `src/components/surfaces/GlassSurface.tsx`
- Modify: `src/hooks/useInteractionReporting.ts`
- Test: `src/test/glass-fallback.test.tsx`
- Test: `e2e/accessibility-glass.spec.ts`

- [ ] **Step 1: 写失败测试**

测试滚动中降低 `content` blur、停止滚动 150-300ms 后恢复、连续帧超预算后降低非关键 blur、reduced transparency 永远不使用 blur、forced colors 退回 solid surface。

- [ ] **Step 2: 运行测试确认失败**

Run: `npm run test -- src/test/glass-fallback.test.tsx`

Expected: fallback 逻辑缺失。

- [ ] **Step 3: 写最小实现**

通过 CSS class 和性能采样状态控制材质强度，只改变 blur、透明度和阴影，不改变布局尺寸。

- [ ] **Step 4: 运行测试确认通过**

Run: `npm run check:all`

Expected: 玻璃降级和可访问性单元测试通过。

- [ ] **Step 5: 运行可访问性 e2e**

Run: `npm run e2e -- e2e/accessibility-glass.spec.ts`

Expected: keyboard-only 可完成浏览、搜索、右键和弹窗关闭；focus ring 可见；forced-colors、reduced motion、reduced transparency 均生效。

- [ ] **Step 6: Commit**

```bash
git add src/styles src/components/surfaces src/hooks src/test e2e
git commit -m "feat: add glass fallback accessibility"
```

## Task 10.1: 完成设置页、主题和错误体验

**Files:**
- Create: `src/components/settings/SettingsView.tsx`
- Modify: `src/stores/settings.ts`
- Modify: `src/components/dialogs/PropertiesDialog.tsx`
- Create: `src/components/feedback/Toast.tsx`
- Create: `src/components/feedback/ErrorState.tsx`
- Test: `src/test/settings-view.test.tsx`
- Test: `src/test/error-copy.test.tsx`
- Test: `e2e/settings-errors.spec.ts`

- [ ] **Step 1: 写失败测试**

测试默认启动路径、默认视图、隐藏文件、扩展名、主题、排序偏好、缩略图、动效、降低透明度设置；错误文案必须包含发生了什么、可能原因、下一步。

- [ ] **Step 2: 运行测试确认失败**

Run: `npm run test -- src/test/settings-view.test.tsx src/test/error-copy.test.tsx`

Expected: 设置页或错误组件缺失。

- [ ] **Step 3: 写最小实现**

轻量设置即时保存并等待 Rust 确认；路径类设置显式确认；保存失败回滚到最后确认值并显示重试入口。

- [ ] **Step 4: 运行测试确认通过**

Run: `npm run check:all`

Expected: 设置和错误体验测试通过。

- [ ] **Step 5: 运行 V1.0 设置 e2e**

Run: `npm run e2e -- e2e/settings-errors.spec.ts`

Expected: 设置持久化；主题切换生效；缓存不可写、路径不存在、权限不足等错误显示结构化文案。

- [ ] **Step 6: Commit**

```bash
git add src/components/settings src/components/feedback src/stores/settings.ts src/components/dialogs src/test e2e
git commit -m "feat: polish settings and errors"
```

## Task 10.2: 完成 Windows 窗口行为、打包、崩溃日志和自动更新入口

**Files:**
- Modify: `src/components/window/WindowChrome.tsx`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/src/core/observability.rs`
- Modify: `.github/workflows/ci.yml`
- Create: `scripts/smoke-bundle.ps1`
- Test: `e2e/window-behavior.spec.ts`
- Test: `src-tauri/tests/crash_log.rs`

- [ ] **Step 1: 写失败测试**

e2e 覆盖标题栏拖拽、最小化、最大化/还原、关闭按钮热区、Snap Layout 可用性、高 DPI 缩放布局稳定。Rust 测试覆盖崩溃日志目录创建和路径脱敏。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml crash_log && npm run e2e -- e2e/window-behavior.spec.ts`

Expected: 崩溃日志或窗口行为未完善。

- [ ] **Step 3: 写最小实现**

自定义标题栏保留明确拖拽区域；窗口控制按钮热区符合 Windows 习惯；最大化后弱化外圆角/阴影；加入本地崩溃日志入口；Tauri 配置保留自动更新入口但不启用未配置的远程发布。

- [ ] **Step 4: 运行测试确认通过**

Run: `npm run check:all`

Expected: 单元测试、类型检查和构建通过。

- [ ] **Step 5: 运行打包 smoke**

Run: `npm run tauri:build; powershell -ExecutionPolicy Bypass -File scripts/smoke-bundle.ps1`

Expected: 生成可直接启动的 Windows 桌面二进制入口；启动后显示主窗口；不需要单独启动前端或后端服务。

- [ ] **Step 6: 运行 V1.0 总 e2e**

Run: `npm run e2e`

Expected: V0.1 到 V1.0 所有 e2e 通过；所有破坏性测试只作用于隔离测试根；性能证据、错误日志和截图归档到测试报告目录。

- [ ] **Step 7: 人工发布前审核**

人工确认永久删除、覆盖、剪切、移动、冲突处理、回收站、默认应用、终端打开、拖拽语义、UI 优先调度、Windows 窗口行为和“可日常替代资源管理器”主观体验。

- [ ] **Step 8: Commit**

```bash
git add src/components/window src-tauri/tauri.conf.json src-tauri/src/core/observability.rs .github/workflows/ci.yml scripts e2e src-tauri/tests
git commit -m "chore: prepare V1 desktop release"
```

## 推荐执行顺序

1. V0.0：Task 0.1 -> 0.2 -> 0.3，先建立工程、类型、错误、夹具和 command 边界。
2. V0.1：Task 1.1 -> 1.2 -> 1.3 -> 1.4，先路径安全，再只读浏览，再前端浏览骨架，最后修稳检查链路与浏览器降级体验后再进入文件列表交互。
3. V0.2：Task 2.1 -> 2.2，先数据行为，再视觉视图。
4. V0.3：Task 3.1 -> 3.2，先虚拟列表和分页，再 UI 优先调度。
5. V0.4：Task 4.1，搜索单独成版。
6. V0.5：Task 5.1 -> 5.2，先任务状态机，再基础文件操作。
7. V0.6：Task 6.1 -> 6.2，先 clipboard/paste，再 drag/copy/move 进度。
8. V0.7：Task 7.1 -> 7.2，先冲突，再多选/菜单/快捷键。
9. V0.8：Task 8.1 -> 8.2，先多标签，再快照一致性。
10. V0.9：Task 9.1 -> 9.2，先缩略图，再材质降级和可访问性补强。
11. V1.0：Task 10.1 -> 10.2，先设置/错误，再打包/崩溃/窗口行为/总验收。

## 可并行机会

- Task 2.2 的 UI token 和 Task 2.1 的 Rust 排序过滤可以并行，前提是共享 `ViewMode`、`SortKey`、`FilterKind` 类型先合入。
- Task 3.2 的前端交互上报 hook 和 Rust scheduler 测试可以并行，前提是 command 名称和请求结构先冻结。
- Task 7.2 的右键菜单视觉和选择 store 可以并行，前提是 selection contract 先冻结。
- Task 9.2 的 forced-colors/reduced-transparency CSS 可以和 Task 9.1 的 Rust thumbnail cache 并行，前提是缩略图 UI 状态枚举先冻结。
- 最终合并、危险文件操作、冲突处理、发布打包和人工审核不得交给并行结果直接落地，必须由主实施者统一复核。

## 高风险任务

- Task 1.1 路径安全：风险是 symlink/junction/reparse point 逃逸测试根或真实用户目录。缓解：destructive operation 在解析 reparse point 后再次 guard，失败默认拒绝。
- Task 5.2 基础文件操作：风险是永久删除绕过确认、回收站失败降级为永久删除、重命名覆盖。缓解：确认令牌、结构化错误、人工审核、真实夹具断言。
- Task 6.2 复制移动：风险是跨卷移动失败后源/目标状态不一致。缓解：复制成功后再删除源，失败进入 `partially_completed` 并列出状态。
- Task 7.1 冲突处理：风险是默认覆盖或应用于全部范围不清。缓解：无默认危险选择、明确同类型冲突范围、人工审核。
- Task 8.2 快照一致性：风险是多标签显示过期状态并继续操作。缓解：`snapshot_version`、`directory_changed` 和旧事件丢弃。
- Task 9.1 缩略图缓存：风险是缓存目录不可写、损坏图片阻塞、滚动掉帧。缓解：fallback 图标、失败退避、可视区域优先、滚动延迟。
- Task 10.2 打包和窗口行为：风险是自定义标题栏破坏 Windows Snap 或按钮热区。缓解：窗口 e2e、人工验证、多 DPI 验收。

## 提交节奏

- 每个 Task 一个 commit；高风险 Task 在 commit 前必须留下测试命令输出和人工审核结论。
- 一个版本内所有 Task 完成后增加版本验收 commit，消息格式为 `test: validate RustFiles V0.X <scope>`，提交 e2e 报告配置、截图基线或文档化验收记录。
- 不把多个版本能力塞进同一个 commit；例如 V0.4 搜索不得夹带缩略图实现，V0.8 多标签不得夹带文件操作语义修改。

## 暂不推进的内容

- FTP/SFTP、云盘同步、局域网共享、跨设备同步。
- Git 状态、插件系统、文件标签、全文索引、AI 文件整理。
- 压缩包浏览、高级批量重命名、内置预览器、内置终端。
- 标签分组、标签会话同步、跨窗口拖拽标签、复杂工作区恢复。
- 视频缩略图、PDF 缩略图。
- 与 Windows 资源管理器的跨应用拖入/拖出互操作。
- 自动更新服务端、官网落地页和远程崩溃上报服务；V1.0 只保留本地入口和配置边界。

## 自审结果

- 已覆盖 PRD 中的浏览、多标签、搜索、排序、过滤、缩略图、设置和文件操作范围。
- 已延续 CDD 的 R4 风险边界、测试根隔离、真实文件系统验证和人工审核要求。
- 已延续架构文档中的 Rust 后端唯一写入者、Tauri command 白名单、路径安全、任务状态机、UI 优先调度、快照一致性和发布单入口要求。
- 已延续前端设计文档中的 Apple-like Liquid Workbench、Window Chrome、快捷键、选择与 inline rename、玻璃降级、批量冲突和 forced-colors 要求。
- 每个任务都包含文件路径、测试命令、预期结果、e2e 验证和提交节奏。
- 单元测试、typecheck、lint 和 build 被列为必要检查，但每个阶段完成仍以真实运行层面的 e2e 为验收条件。
