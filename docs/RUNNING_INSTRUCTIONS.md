# Wealthy MCP 适配器 - 启动和使用指南

## 项目结构概述

我们已经完成了 MCP (Model Context Protocol) 适配器的实现，包含以下关键组件：

1. **Rust 后端实现** (`src-tauri/src/mcp/`)
   - MCP 服务端点
   - 请求/响应处理
   - 与现有命令系统的集成

2. **示例客户端** (`example-mcp-client.js`, `populate-with-mcp.js`)
   - JavaScript 客户端示例
   - 数据填充脚本

3. **配置和文档** (`mcp-config.json`, `MCP_ADAPTER.md`, `MCP_QUICK_START.md`)
   - MCP 服务配置
   - 详细使用文档

## 启动步骤

### 1. 安装依赖

```bash
# 确保您已安装 Rust 和 Tauri CLI
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install tauri-cli

# 安装 Node.js 依赖
npm install
# 或者如果使用 pnpm
pnpm install
```

### 2. 启动应用（开发模式）

```bash
# 启动前端和 Tauri 应用
npm run tauri dev
# 或者
pnpm tauri dev
```

### 3. 验证 MCP 服务

当 Wealthy 应用启动后，MCP 服务会自动在 `http://localhost:3030/mcp` 启动。

验证服务是否运行：
```bash
# 检查服务状态
node check-mcp-status.js
```

或者使用 curl 直接测试：
```bash
curl -X POST http://localhost:3030/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "list_books",
    "params": {},
    "id": 1
  }'
```

## 使用 MCP 适配器填充示例数据

### 1. 确保 Wealthy 应用正在运行

在启动应用后，您应该能看到控制台输出 "MCP Server starting on port 3030"。

### 2. 运行数据填充脚本

```bash
node populate-with-mcp.js
```

这个脚本会：
- 通过 MCP 适配器创建 "2025年4-5月账本"
- 创建支付宝、微信、招商信用卡三个账户
- 创建餐饮、旅游两个分类
- 添加四条交易记录

### 3. 手动测试 MCP 功能

您也可以手动使用示例客户端：

```bash
node example-mcp-client.js
```

## MCP 适配器支持的方法

### 账本管理
- `list_books()` - 列出所有账本
- `create_book(name)` - 创建新账本

### 条目管理
- `list_entries(book_id)` - 列出指定账本的条目
- `create_entry(...)` - 创建新条目（账户、资产等）

### 记录管理
- `list_records(book_id)` - 列出指定账本的交易记录
- `create_record(...)` - 创建新交易记录

### 分类管理
- `list_categories(domain, level)` - 列出分类
- `create_category(...)` - 创建分类

### 更多方法...
完整的支持方法列表请参见 `MCP_ADAPTER.md`

## 故障排除

如果 MCP 服务无法访问：

1. **检查 Wealthy 应用是否正在运行**：
   - 应用启动时应输出 "MCP Server starting on port 3030"
   - 检查控制台是否有错误信息

2. **检查端口占用**：
   ```bash
   lsof -i :3030
   ```

3. **检查防火墙设置**：
   - 确保本地端口 3030 没有被阻止

4. **查看应用日志**：
   - Tauri 应用的日志将显示 MCP 服务的启动和运行状态

## 验证结果

成功运行数据填充脚本后，您可以在 Wealthy 应用中查看：

1. 新创建的 "2025年4-5月账本"
2. 账本下的支付宝、微信、招商信用卡账户
3. 餐饮、旅游分类
4. 四条交易记录（韩国旅游、研究生补助、股市投入、生活费）

## 安全注意事项

- MCP 服务仅在本地运行（localhost:3030），不应暴露给公网
- 只有在同一设备上运行的受信任应用才能访问此服务
- 所有数据操作都经过相同的验证和授权流程

## 集成到 AI 工作流

MCP 适配器允许 AI 模型直接访问财务数据以提供分析、报告和管理建议。