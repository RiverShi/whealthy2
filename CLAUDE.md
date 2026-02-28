# CLAUDE.md

Wealthy 个人财务管理应用 - 开发速查

## 技术栈

- **前端**: Vue 3 + TypeScript + Tailwind CSS + Pinia
- **后端**: Tauri 2 + Rust + SQLite
- **构建**: Vite

## 包管理

- **前端**: pnpm
- **后端**: cargo


## 项目结构

```
src/                    # 前端
├── api/               # Tauri 命令调用
├── pages/             # 页面组件
├── stores/            # Pinia 状态管理
├── components/        # UI 组件
└── layouts/           # 布局

src-tauri/src/         # 后端
├── commands/          # 业务命令 (books, entries, records, ...)
├── mcp/               # MCP HTTP 服务
├── db.rs              # 数据库初始化
├── error.rs           # 错误类型
└── lib.rs             # 主入口
```

## 核心概念

| 概念 | 说明 |
|------|------|
| Book | 账本，财务数据的顶层容器 |
| Entry | 条目，资产或负债实体 |
| Record | 流水记录，收入/支出/转账 |
| Event | 事件，多条流水的聚合分组 |
| Category | 两级分类 (asset/liability/income/expense) |
| Snapshot | 净值快照，历史价值记录 |

## 常用命令

```bash
web 
pnpm dev              # 前端开发
pnpm tauri dev        # Tauri 开发
pnpm build            # 构建前端
pnpm tauri build      # 构建应用
pnpm tauri ios dev      # 构建 iOS 应用
pnpm tauri ios build    
pnpm tauri ios build --export-method app-store-connect ...
```

## 关键文件速查

| 功能 | 前端 | 后端 |
|------|------|------|
| 账本 | `stores/books.ts` | `commands/books.rs` |
| 条目 | `stores/entries.ts` | `commands/entries.rs` |
| 流水 | `api/records.ts` | `commands/records.rs` |
| 分类 | `stores/categories.ts` | `commands/categories.rs` |
| 快照 | `api/views.ts` | `commands/snapshots.rs` |
| 导入导出 | `api/export_import.ts` | `commands/export_import.rs` |

## 详细文档

- `docs/API_REFERENCE.md` - 完整 API 参考
- `docs/ARCHITECTURE.md` - 架构细节
- `docs/MCP_ADAPTER.md` - MCP 服务说明
