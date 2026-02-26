# CLAUDE.md

此文件为 Claude Code (claude.ai/code) 在处理此仓库代码时提供指导。

## 项目概述

Wealthy 是一个使用 Tauri 2、Vue 3、TypeScript 和 Rust 构建的个人财务管理应用程序。它使用 SQLite 进行数据存储，采用资产优先的财务跟踪方法，支持多账本、详细分类和历史快照。

## 架构

### 前端 (Vue 3 + TypeScript)
- **框架**: 使用 Composition API 的 Vue 3
- **路由**: 使用 hash history 的 Vue Router
- **状态管理**: Pinia 存储
- **UI 组件**: Shadcn-vue 与 Tailwind CSS
- **图标**: Lucide Vue Next 图标
- **结构**:
  - `src/pages/`: 按功能组织的主要应用页面
  - `src/stores/`: 用于状态管理的 Pinia 存储
  - `src/api/`: 与 Tauri 后端通信的 API 层
  - `src/layouts/`: 应用布局
  - `src/components/`: 可重用的 UI 组件

### 后端 (Rust + SQLite)
- **框架**: Tauri 2
- **数据库**: 通过 rusqlite 的 SQLite
- **命令**: 将后端功能暴露给前端的 Tauri 命令
- **结构**:
  - `src-tauri/src/lib.rs`: 定义所有暴露命令的主要 Tauri 模块
  - `src-tauri/src/commands/`: 不同领域的独立模块
  - `src-tauri/src/db.rs`: 数据库初始化和模式管理
  - `src-tauri/src/error.rs`: 错误处理定义

## 核心数据模型

### 领域概念
- **账本 (Book)**: 包含资产、负债和交易的财务账本容器
- **条目 (Entry)**: 具有估值和分类的资产/负债项
- **事件 (Event)**: 相关交易记录的可选分组
- **记录 (Record)**: 个别交易记录（收入、支出、转账）
- **分类 (Category)**: 两级分层分类系统
- **标签 (Tag)**: 按领域划分的自由标签系统
- **快照 (Snapshot)**: 历史资产价值保存
- **快照任务 (SnapshotTask)**: 周期性快照调度

### 关键 API 端点
- 账本: `list_books`, `create_book`, `update_book`, `archive_book`, `delete_book`
- 条目: `list_entries`, `get_entry`, `create_entry`, `update_entry`, `adjust_entry_value`
- 记录: `list_records`, `get_record`, `create_record`, `update_record`, `delete_record`
- 事件: `list_events`, `get_event`, `create_event`, `update_event`, `delete_event`
- 分类: `list_categories`, `create_category`, `update_category`, `delete_category`
- 快照: `list_snapshots`, `create_snapshot`, `diff_snapshots`

## 开发命令

### 常见操作
- `pnpm dev` - 启动开发服务器
- `pnpm build` - 构建生产包
- `pnpm preview` - 本地预览生产构建
- `pnpm tauri dev` - 启动带热重载的 Tauri 开发环境
- `pnpm tauri build` - 构建 Tauri 应用程序以供分发

### 数据库模式更新
- 模式更改应在 `src-tauri/src/db.rs` 中进行
- 数据库在首次运行时自动初始化
- 应谨慎添加新迁移以保持向后兼容性

## 关键特性

### 多账本支持
- 用户可以创建多个财务账本
- 每个账本独立管理自己的资产、负债和交易
- 统计可以通过视图跨多个账本聚合

### 资产优先方法
- 主要关注跟踪资产和负债的价值变化
- 资产可以标记为账户以进行交易链接
- 支持手动价值调整及历史记录跟踪

### 灵活分类
- 两级分层分类系统
- 领域隔离的分类（资产/负债/收入/支出）
- 支持颜色编码的标签系统

### 快照系统
- 手动和自动定期快照
- 价值变化随时间跟踪
- 应用启动时延迟快照执行

### 交易管理
- 收入/支出/转账交易类型
- 相关交易的可选事件分组
- 基于账户的现金流跟踪

## 文件位置

### 关键页面
- `src/pages/records/RecordsPage.vue` - 交易历史视图
- `src/pages/books/BookDetailPage.vue` - 资产/负债详情视图
- `src/pages/views/ViewStatsPage.vue` - 统计概览
- `src/pages/settings/CategoriesPage.vue` - 分类管理
- `src/pages/settings/TagsPage.vue` - 标签管理

### 关键存储
- `src/stores/books.ts` - 账本管理存储
- `src/stores/categories.ts` - 分类管理存储
- `src/stores/entries.ts` - 条目管理存储

### 关键后端模块
- `src-tauri/src/commands/books.rs` - 账本管理命令
- `src-tauri/src/commands/entries.rs` - 条目管理命令
- `src-tauri/src/commands/records.rs` - 交易管理命令
- `src-tauri/src/commands/snapshots.rs` - 快照管理命令