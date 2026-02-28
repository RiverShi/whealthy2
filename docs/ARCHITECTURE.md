# 架构说明

Wealthy 应用架构详解。

## 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                        Frontend (Vue 3)                     │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐        │
│  │  Pages  │  │ Stores  │  │   API   │  │Components│        │
│  └────┬────┘  └────┬────┘  └────┬────┘  └─────────┘        │
│       │            │            │                          │
│       └────────────┴────────────┘                          │
│                    │                                       │
│              Tauri API (@tauri-apps/api)                   │
└────────────────────┼────────────────────────────────────────┘
                     │
┌────────────────────┼────────────────────────────────────────┐
│                    ▼                                        │
│                   Backend (Rust/Tauri)                      │
│  ┌─────────────────┬─────────────────┬─────────────────┐   │
│  │  Tauri Commands │  MCP Server     │  Database       │   │
│  │  (lib.rs)       │  (Port 3030)    │  (SQLite)       │   │
│  └─────────────────┴─────────────────┴─────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## 前端架构

### 状态管理 (Pinia)

```
stores/
├── books.ts       # 账本状态，管理 activeBookId
├── entries.ts     # 条目状态，提供 assets/liabilities 计算属性
└── categories.ts  # 分类和标签状态
```

Store 职责：
- 封装 API 调用
- 维护本地状态缓存
- 提供计算属性（如 `totalAssets`, `netWorth`）
- 处理错误和加载状态

### API 层

所有 API 模块遵循相同模式：

```typescript
// 类型定义
interface Xxx { ... }
interface CreateXxxParams { ... }

// API 对象
export const xxxApi = {
  list: () => invoke<Xxx[]>("list_xxx"),
  create: (params) => invoke<Xxx>("create_xxx", { params }),
  // ...
};
```

### 页面结构

```
pages/
├── home/           # 首页概览
├── records/        # 账目/流水（混合 Feed）
├── books/          # 账本管理 + 资产详情
├── views/          # 统计图表
├── settings/       # 分类、标签设置
└── more/           # 更多功能入口
```

### 布局系统

`AppLayout.vue` 提供：
- 底部固定导航栏（4 个主 tab）
- 全局 FAB（浮动按钮）- 仅在账目页和资产页显示
- 无账本时的引导页面

## 后端架构

### 模块划分

```
src-tauri/src/
├── lib.rs              # 主入口，定义所有 Tauri 命令
├── db.rs               # 数据库初始化和迁移
├── error.rs            # 错误类型 AppError/AppResult
├── commands/           # 业务逻辑
│   ├── books.rs        # 账本 CRUD
│   ├── entries.rs      # 条目 + 调整历史
│   ├── records.rs      # 流水 + 事件 + Feed
│   ├── categories.rs   # 分类 + 标签
│   ├── snapshots.rs    # 快照 + 统计 + 任务
│   └── export_import.rs # 导入导出
└── mcp/
    ├── mod.rs          # MCP 协议处理
    └── server.rs       # HTTP 服务器
```

### 错误处理

统一使用 `AppResult<T>` 和 `AppError`：

```rust
pub enum AppError {
    Db(rusqlite::Error),
    NotFound(String),
    InvalidInput(String),
    Json(serde_json::Error),
}
```

所有错误自动转换为字符串返回给前端。

### 数据库访问

共享连接模式：

```rust
pub struct DbState(pub Arc<Mutex<Connection>>);

// Tauri 命令和 MCP 服务共用同一连接
app.manage(DbState(Arc::clone(&db_arc)));
```

### MCP 服务

启动时自动在后台运行：

```rust
// lib.rs
spawn(async move {
    mcp::server::start_mcp_server(3030, db_arc).await
});
```

- 端点: `POST http://127.0.0.1:3030/mcp`
- 协议: JSON-RPC 2.0
- 功能: 与 Tauri 命令完全相同的 API

## 关键设计决策

### 混合 Feed

账目页使用 `list_feed` 替代简单的流水列表：

- 事件显示聚合金额（总收入/总支出）
- 独立流水直接显示
- 统一按日期或金额排序

### 账户系统

条目标记 `is_account=true` 后可参与资金流转：

- 支出: from_account → 外部
- 收入: 外部 → to_account
- 转账: from_account → to_account

### 价值调整追踪

手动调整条目价值时自动记录：

```rust
// 更新当前值
UPDATE entries SET value = ? WHERE id = ?
// 记录历史
INSERT INTO entry_adjustments (...)
```

### 导入导出策略

智能复用已有数据：

1. 分类按 name+domain+level(+parent) 匹配
2. 标签按 name+domain 匹配
3. 条目、事件、流水创建新记录
4. 维护 ID 映射表确保关联正确

### 快照系统

支持手动和自动快照：

- 快照保存完整条目状态（JSON 序列化）
- 支持快照对比查看变化
- 自动任务按 daily/weekly/monthly 执行

## 数据流

```
用户操作
   ↓
Vue 组件
   ↓
Pinia Store (状态更新)
   ↓
API 层 (invoke)
   ↓
Tauri Command
   ↓
Rust 业务逻辑
   ↓
SQLite (rusqlite)
   ↓
返回结果 → Store 更新 → UI 响应
```

## 安全考虑

- MCP 服务绑定 localhost only
- 数据库使用应用私有目录
- 无远程网络请求
