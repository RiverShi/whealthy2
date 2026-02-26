# Wealthy MCP 适配器 - 快速入门指南

## 什么是 MCP 适配器？

Wealthy MCP (Model Context Protocol) 适配器是一个接口，允许外部 AI 模型或其他系统通过标准的 JSON-RPC 协议访问您的财务数据。它让 AI 助手能够帮助您分析财务状况、生成报告或协助管理财务记录。

## 如何使用 MCP 适配器

### 1. 启动 Wealthy 应用
首先确保 Wealthy 应用程序正在运行，MCP 服务会自动在端口 3030 上启动。

### 2. 验证 MCP 服务
服务启动后，您可以向 `http://localhost:3030/mcp` 发送 JSON-RPC 请求。

示例请求：
```json
{
  "jsonrpc": "2.0",
  "method": "list_books",
  "params": {},
  "id": 1
}
```

### 3. 使用 JavaScript 客户端示例
我们提供了一个 JavaScript 客户端示例 (`example-mcp-client.js`) 来演示如何与 MCP 服务交互。

要运行示例：

1. 安装 Node.js 依赖：
   ```bash
   npm install axios
   ```

2. 运行示例客户端：
   ```bash
   node example-mcp-client.js
   ```

## 支持的操作

### 账本管理
- **列出账本**: `list_books()` - 返回所有财务账本的列表
- **创建账本**: `create_book(name: string)` - 创建新的财务账本

### 条目管理
- **列出条目**: `list_entries(book_id: string)` - 列出指定账本中的所有财务条目（如银行账户、信用卡等）
- **创建条目**: `create_entry(...)` - 创建新的财务条目

### 交易记录管理
- **列出记录**: `list_records(book_id: string)` - 列出指定账本中的所有交易记录
- **创建记录**: `create_record(...)` - 创建新的交易记录（收入、支出或转账）

### 分类和标签管理
- **列出分类**: `list_categories(domain: string, level?: number)` - 列出支出或收入分类
- **创建分类**: `create_category(...)` - 创建新的分类

## JSON-RPC 请求格式

所有请求都应采用标准的 JSON-RPC 2.0 格式：

```json
{
  "jsonrpc": "2.0",
  "method": "方法名",
  "params": {
    // 参数对象
  },
  "id": 任意唯一标识符
}
```

## 常见的 MCP 方法示例

### 获取所有账本
```json
{
  "jsonrpc": "2.0",
  "method": "list_books",
  "params": {},
  "id": 1
}
```

### 获取特定账本的交易记录
```json
{
  "jsonrpc": "2.0",
  "method": "list_records",
  "params": {
    "book_id": "some-book-id"
  },
  "id": 2
}
```

### 创建新的交易记录
```json
{
  "jsonrpc": "2.0",
  "method": "create_record",
  "params": {
    "book_id": "some-book-id",
    "type": "expense",
    "amount": 45.50,
    "happened_at": "2025-01-15T12:00:00.000Z",
    "from_account_id": "account-id",
    "category_id": "category-id",
    "note": "午餐费用"
  },
  "id": 3
}
```

## 安全注意事项

- **本地访问**: MCP 服务默认仅在 localhost 上运行，不建议在公共网络上暴露
- **权限**: 通过 MCP 进行的所有操作都具有与 Wealthy 应用相同的权限级别
- **数据保护**: 请谨慎分享您的财务数据，只向受信任的应用或 AI 模型授予访问权限

## 故障排除

- 如果无法连接到 MCP 服务，请确保 Wealthy 应用正在运行
- 检查端口 3030 是否被防火墙阻止
- 查看 Wealthy 应用日志中的错误消息

## 集成到 AI 工作流

MCP 适配器特别适用于：

1. **AI 财务助手**: 让 AI 模型访问您的财务数据以提供分析和建议
2. **自动化报告**: 自动生成月度或年度财务报告
3. **智能分类**: 让 AI 帮助自动分类新的交易记录
4. **预算分析**: 基于历史数据的预算规划和分析

## 贡献

如果您希望扩展 MCP 适配器的功能或修复问题，请查看源代码位于 `src-tauri/src/mcp/` 目录。