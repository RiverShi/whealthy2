# Wealthy MCP (Model Context Protocol) 适配器

Wealthy 是一款个人财务管理应用程序，支持通过 MCP (Model Context Protocol) 协议访问其财务数据。本适配器允许 AI 模型或其他外部系统安全地查询和操作财务信息。

## 概述

Wealthy MCP 适配器提供了一个标准化的接口，允许外部系统访问以下功能：
- 账本管理 (Books)
- 账户条目管理 (Entries)
- 交易记录管理 (Records)
- 分类和标签管理 (Categories & Tags)
- 事件和快照管理 (Events & Snapshots)

## 服务端点

- **地址**: `http://localhost:3030/mcp`
- **协议**: JSON-RPC 2.0 over HTTP POST
- **内容类型**: `application/json`

## 支持的方法

### 账本操作
- `list_books()` - 列出所有账本
- `create_book(name: string)` - 创建新账本
- `update_book(id: string, name: string)` - 更新账本
- `delete_book(id: string)` - 删除账本

### 条目操作
- `list_entries(book_id: string, filter?: EntryFilter)` - 列出条目
- `create_entry(params: CreateEntryParams)` - 创建条目
- `update_entry(id: string, params: CreateEntryParams)` - 更新条目
- `delete_entry(id: string)` - 删除条目
- `adjust_entry_value(id: string, new_value: number, reason?: string)` - 调整条目值
- `list_entry_adjustments(entry_id: string)` - 列出调整记录

### 记录操作
- `list_records(book_id: string, filter?: RecordFilter)` - 列出记录
- `create_record(params: CreateRecordParams)` - 创建记录
- `update_record(id: string, params: UpdateRecordParams)` - 更新记录
- `delete_record(id: string)` - 删除记录
- `get_record(id: string)` - 获取单个记录
- `list_feed(book_id: string, sort?: FeedSort)` - 获取混合信息流

### 事件操作
- `list_events(book_id: string)` - 列出事件
- `create_event(params: CreateEventParams)` - 创建事件
- `update_event(id: string, params: UpdateEventParams)` - 更新事件
- `delete_event(id: string)` - 删除事件
- `get_event(id: string)` - 获取事件详情

### 分类和标签操作
- `list_categories(domain: string, level?: number)` - 列出分类
- `create_category(domain: string, level: number, name: string, parent_id?: string, icon?: string)` - 创建分类
- `update_category(id: string, name?: string, icon?: string, parent_id?: string)` - 更新分类
- `delete_category(id: string)` - 删除分类
- `list_tags(domain?: string)` - 列出标签
- `create_tag(domain: string, name: string, color?: string)` - 创建标签
- `update_tag(id: string, name?: string, color?: string)` - 更新标签
- `delete_tag(id: string)` - 删除标签

### 统计和快照操作
- `get_book_stats(book_id: string, from: string, to: string)` - 获取账本统计
- `list_snapshots(book_id: string, from?: string, to?: string)` - 列出快照
- `create_snapshot(book_id: string)` - 创建快照
- `get_snapshot(id: string)` - 获取快照
- `diff_snapshots(from_id: string, to_id: string)` - 比较快照
- `check_and_run_snapshot_tasks()` - 运行快照任务

## 数据模型

### Book（账本）
表示一个财务账本，如"2025年家庭账本"。

### Entry（条目）
表示一个财务实体，可以是资产（如银行账户）、负债（如信用卡）等。

### Record（记录）
表示一笔交易记录，如一次消费、收入或转账。

### Category（分类）
对交易进行分类，如"餐饮"、"交通"等。

### Event（事件）
将多个相关的交易记录组织在一起，如"春游活动"。

## 安全考虑

- MCP 服务目前没有认证机制，请仅在受信任的环境中使用
- 不要在公共网络上暴露 MCP 端点
- 定期备份财务数据以防意外损失

## 故障排除

如果遇到连接问题，请检查：
1. Wealthy 应用是否正在运行
2. 端口 3030 是否被其他应用占用
3. 防火墙设置是否阻止本地连接

## 开发信息

- 服务在应用启动时自动启动
- 使用 Rust + Tauri + Axum 实现
- 数据存储在本地 SQLite 数据库中