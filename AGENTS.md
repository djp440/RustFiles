# RustFiles 项目上下文

## 项目定位

RustFiles 是一款面向 Windows 10 / 11 用户的高性能、精致、现代化本地文件管理器，目标是成为 Windows 资源管理器的日常替代品。

V1 主方向是“漂亮顺手的日常替代品”，但性能与稳定是不可妥协的硬底线。

## 当前阶段

项目已进入 CDD 实施阶段。PRD、CDD 规格、架构设计、前端 UI 设计和实现计划均已完成；工程脚手架已落地，当前正在按实现计划逐任务推进。

当前 CDD 流程顺序：

1. `cdd-brainstorming`
2. `cdd-specification`
3. `cdd-architecture-design`
4. `cdd-frontend-design`
5. `cdd-writing-plans`
6. 实现与验证

进入编码实施后，以已批准的实现计划为直接输入；除非用户明确要求修改上游文档，不再回退重写 PRD、CDD 规格、架构设计或前端 UI 设计。

当前实施状态：

- 已完成 `Task 0.1`：最小 Tauri 2 + React + TypeScript + Vite 工程脚手架、基础测试脚本、最小桌面 smoke 启动脚本。
- 已完成 `Task 0.2` 的共享契约子包：Rust Core 共享类型、错误模型、观测模块占位和 `types_contract` 测试。
- 已完成 `Task 0.2` 的测试夹具子包：`scripts/create-fixtures.ps1`、`fixture_generation` 测试，以及隔离夹具根目录拒绝保护。
- 已完成 `Task 0.3`：Tauri command 白名单、危险 command 测试模式 guard 和空 command adapter。
- 已完成 `Task 1.1`：Windows 路径安全模块 `path_safety`，覆盖保留名、非法字符、长路径、reparse point、UNC、subst/网络映射盘和测试根逃逸保护。
- 已完成 `Task 1.2`：只读目录枚举、侧边栏 roots、驱动器基础信息，以及对应 Rust 集成测试。
- 已完成 `Task 1.3`：主界面浏览骨架、导航状态、路径输入、面包屑、侧边栏入口和 Playwright navigation e2e。
- 已完成 `Task 1.4`：验证链路修稳、Vitest 不再误收 Playwright spec、浏览器预览模式降级提示与文案编码修复，以及 `tauri:dev:test` 残留进程误判修复。
- 已完成 `Task 2.1`：Rust 排序/过滤、最小 settings 持久化与 commands、前端 settings store、Toolbar 接线，以及 `view-sort-filter` 组件测试和 Playwright 验收。
- 已完成 `Task 2.2`：三种文件视图（icon/list/details）切换、统一 Liquid Glass token 材质系统（`tokens.css` / `global.css`）、`GlassSurface` 材质组件、`FileGrid` / `FileList` / `DetailsTable` 展示组件，以及 `view-mode` 组件测试和 Playwright glass-readability e2e。
- 已完成 `Task 3.1`：Rust 后端大目录分页快照（`snapshot.rs`、`DirectoryPage` 增加 `offset`/`limit` 字段、`list_directory` 支持可选分页参数）、前端 `@tanstack/react-virtual` 虚拟列表（FileGrid 132px / FileList 36px / DetailsTable 44px 固定行高）、`large_directory` Rust 测试和 `virtual-list` 组件测试，以及 Playwright performance e2e。
- 已完成 `Task 3.2`：Rust `scheduler` 调度优先级与性能采样基础、`report_viewport_state` / `report_interaction_state` command、前端 viewport / interaction reporting hooks、browser-preview 调试容器 `window.__RUSTFILES_SCHEDULER_DEBUG__`、`interaction-reporting` 组件测试，以及 Playwright `ui-priority` browser-preview 验证。
- 已完成 `Task 4.1`：Rust 搜索契约与最小可取消任务骨架（`search.rs`、`tasks.rs`、`start_search` / `cancel_task`）、前端搜索 store 与 Toolbar 接线、当前目录实时过滤与递归搜索取消、搜索结果“打开所在位置”恢复链路，以及 Playwright `search` browser-preview 验证。
- 当前可用验证命令：`npm run check:all`、`npm run tauri:dev:test`、`npm run e2e -- e2e/navigation.spec.ts`、`npm run e2e -- e2e/view-sort-filter.spec.ts`、`npm run e2e -- e2e/glass-readability.spec.ts`、`npm run e2e -- e2e/performance.spec.ts`、`npm run e2e -- e2e/ui-priority.spec.ts`、`npm run e2e -- e2e/search.spec.ts`、`npm run test -- src/test/interaction-reporting.test.ts`、`npm run test -- src/test/search-store.test.ts`、`cargo test --manifest-path src-tauri/Cargo.toml --test command_whitelist`、`cargo test --manifest-path src-tauri/Cargo.toml --test path_safety`、`cargo test --manifest-path src-tauri/Cargo.toml --test fs_listing`、`cargo test --manifest-path src-tauri/Cargo.toml --test sort_filter`、`cargo test --manifest-path src-tauri/Cargo.toml --test settings`、`cargo test --manifest-path src-tauri/Cargo.toml --test sidebar_roots`、`cargo test --manifest-path src-tauri/Cargo.toml --test large_directory`、`cargo test --manifest-path src-tauri/Cargo.toml --test scheduler`、`cargo test --manifest-path src-tauri/Cargo.toml --test search`。
- `scripts/run-tauri-dev-test.ps1` 已修复为自动结束 `tauri dev` 进程树，并只把本次 `tauri dev` 的后代 `node.exe` / `npx.cmd` 视为残留进程，避免误杀当前 Codex 会话自己的 Node 进程。
- 下一实施任务：进入 `Task 5.1`，实现任务状态机和任务面板。

## 长期产品约束

- 目标系统：Windows 10 / 11。
- 技术假设：Rust 核心 / React UI / Tauri 桌面壳。
- V1 聚焦本地文件管理，不做云盘同步、远程协议、插件系统、全文索引或 AI 文件整理。
- V1 必须支持多标签页，但只做基础标签页能力，不做复杂工作区系统。
- 性能调度遵循 UI 优先原则：优先保障当前可见界面的输入响应、滚动、动画、视图切换和路径导航。
- 后台扫描、缩略图、搜索、复制移动等任务可以降速，但不能拖慢前台体感。
- React 前端只表达用户意图和展示状态，真实文件系统写入、删除、移动、冲突执行和系统集成必须由 Rust 后端统一完成。
- 调试阶段允许前端与后端分开运行；发布阶段必须至少能打包为一个可直接启动的桌面二进制入口。
- Windows 路径安全必须覆盖 symlink、junction、reparse point、UNC、网络盘、subst、长路径、保留名、非法字符和大小写冲突。
- Tauri 只暴露显式白名单 command，禁止前端通过文件系统插件或等价能力直接写真实文件。
- UI 采用 Apple-like Liquid Workbench 方向：液态玻璃/毛玻璃是统一材质系统，必须兼顾可读性、性能和全界面一致性。

## 文档位置

- PRD：`docs/cdd-brainstorming/prds/`
- CDD 规格文档：`docs/cdd-specification/specs/`
- 架构设计文档：`docs/cdd-architecture-design/architectures/`
- 前端 UI 设计文档：`docs/cdd-frontend-design/ui-designs/`
- 实现计划文档：`docs/cdd-writing-plans/`

## 当前工程结构

- 前端入口：`src/`
- Tauri / Rust 入口：`src-tauri/`
- 脚本目录：`scripts/`
- 前端测试：`src/test/`
- 端到端测试预留目录：`e2e/`

## 编码规范

- 默认使用满足需求的最小安全改动。
- 优先保持清晰数据流、稳定接口、低耦合和可预测行为。
- 中文项目文档和注释默认使用简体中文。
- 注释解释设计原因、边界条件和非直观行为，不解释显而易见的代码。

## 常见坑点

- Tauri v2 capability 构建校验对 permission 标识较严格；当前项目在 `src-tauri/capabilities/default.json` 里只保留 `core:default`，自定义 command 白名单由 Rust 侧 `tauri::generate_handler!` 强制收口，并用 `src-tauri/tests/command_whitelist.rs` 解析 `src-tauri/src/lib.rs` 做防漂移测试。
- `scripts/run-tauri-dev-test.ps1` 不能只结束父进程；必须结束整棵 `tauri dev` 进程树，并在收尾时检查没有残留 `rustfiles.exe`、`node.exe` 或 `npx.cmd`。
- 路径安全不能只校验输入路径字符串；测试模式下必须在 reparse point 解析后的真实目标路径上再次校验测试根，防止 symlink/junction 逃逸。
- `subst` 盘和网络映射盘不能混同于普通本地盘；`src-tauri/src/core/path_safety.rs` 需要先识别 `UNC`，再单独识别 `SubstDrive`，否则相关测试会出现“枚举值已声明但永远返回不到”的假通过。
- `src-tauri/tests/path_safety.rs` 里的 `test_root_escape_is_rejected` 依赖夹具和 Windows reparse point 能力，默认 `ignored`；执行顺序固定为先运行 `powershell -ExecutionPolicy Bypass -File scripts/create-fixtures.ps1 -Root .\.tmp\fixtures`，再运行 `cargo test --manifest-path src-tauri/Cargo.toml test_root_escape_is_rejected -- --ignored --nocapture`。
- `playwright.config.ts` 的 `baseURL` 固定为 `http://localhost:1420`；运行 `npm run e2e -- e2e/navigation.spec.ts` 前必须确保 Vite 已用 `npm run dev -- --host 127.0.0.1 --port 1420` 启动，否则会在 `page.goto('/')` 处出现 `ERR_CONNECTION_REFUSED`。
- `e2e/ui-priority.spec.ts` 同样依赖 `http://localhost:1420`；若手动启动 `npm run dev -- --host 127.0.0.1 --port 1420` 做 browser-preview 验证，结束后要检查并停止对应监听进程，避免残留占用端口。
- `e2e/search.spec.ts` 也依赖 `http://localhost:1420`；运行前要先启动 `npm run dev -- --host 127.0.0.1 --port 1420`，结束后停止对应监听进程。该用例会验证递归搜索取消、打开所在位置，以及“项目已不存在或已移动”告警恢复链路。
- Playwright navigation e2e 不能用全局 `getByRole('button', { name: 'This PC' })` 断言同名入口；`This PC` 同时存在于侧边栏和面包屑，必须用 `page.getByLabel('Sidebar')` 或 `page.getByLabel('Breadcrumb')` 收窄作用域，避免 strict mode 歧义。
- `e2e/view-sort-filter.spec.ts` 当前在纯浏览器预览模式下通过 `FileBrowser` 的 `View settings` 摘要断言排序、过滤、隐藏文件和扩展名开关的可见结果；如果后续重构这个摘要，需要同步更新对应 Playwright 验收。
- browser-preview 下的调度上报证据通过 `window.__RUSTFILES_SCHEDULER_DEBUG__` 暴露，供 Vitest 和 Playwright 断言；它只能代表浏览器预览模式证据，不能替代未来桌面全栈调度压力验证。
- 临时启动 Vite 做 e2e 后要确认 `127.0.0.1:1420` 没有残留监听；必要时用 `Get-NetTCPConnection -LocalPort 1420 -State Listen` 定位并停止对应 `vite --host 127.0.0.1 --port 1420` 进程。
- `cargo test` 命令后不要追加 `2>&1`。PowerShell 的 `2>&1` 会把 native command 的 stderr ErrorRecord 送入格式化管道，导致 shell 死锁卡死；直接运行 `cargo test ...` 即可。

## 验证原则

- 前端页面以浏览器类人工验收为最终目标。
- 后端能力以实际请求或真实文件系统场景验证为目标。
- 桌面/终端程序必须实际运行最小可用验证。
- 无法验证时必须明确说明风险。
