# RustFiles 项目上下文

## 项目定位

RustFiles 是一款面向 Windows 10 / 11 用户的高性能、精致、现代化本地文件管理器，目标是成为 Windows 资源管理器的日常替代品。

V1 主方向是“漂亮顺手的日常替代品”，但性能与稳定是不可妥协的硬底线。

## 当前阶段

项目处于 CDD 文档流水线末端。PRD、CDD 规格、架构设计、前端 UI 设计和实现计划均已完成；尚未搭建实现代码或技术脚手架。

当前 CDD 流程顺序：

1. `cdd-brainstorming`
2. `cdd-specification`
3. `cdd-architecture-design`
4. `cdd-frontend-design`
5. `cdd-writing-plans`
6. 实现与验证

进入编码实施前，以已批准的实现计划为直接输入；除非用户明确要求修改上游文档，不再回退重写 PRD、CDD 规格、架构设计或前端 UI 设计。

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

## 编码规范

- 默认使用满足需求的最小安全改动。
- 优先保持清晰数据流、稳定接口、低耦合和可预测行为。
- 中文项目文档和注释默认使用简体中文。
- 注释解释设计原因、边界条件和非直观行为，不解释显而易见的代码。

## 验证原则

- 前端页面以浏览器类人工验收为最终目标。
- 后端能力以实际请求或真实文件系统场景验证为目标。
- 桌面/终端程序必须实际运行最小可用验证。
- 无法验证时必须明确说明风险。
