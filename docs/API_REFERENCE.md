# API 参考

完整的前后端 API 说明。

## 前端 API 层 (src/api/)

### books.ts

```typescript
interface Book {
  id: string;
  name: string;
  createdAt: string;
}

bookApi.list()                 // 获取所有账本
bookApi.create(name)           // 创建账本
bookApi.update(id, name)       // 更新账本
bookApi.remove(id)             // 删除账本
```

### entries.ts

```typescript
interface Entry {
  id: string;
  bookId: string;
  name: string;
  kind: 'asset' | 'liability';
  isAccount: boolean;
  valuationType: 'fixed' | 'manual';
  value: number;
  categoryL1Id: string | null;
  categoryL2Id: string | null;
  tagIds: string[];
  openedAt: string;
  closedAt: string | null;
}

entryApi.list(bookId, filter?)     // 列出条目
entryApi.get(id)                   // 获取单个条目
entryApi.create(params)            // 创建条目
entryApi.update(id, params)        // 更新条目
entryApi.remove(id)                // 删除条目
entryApi.adjustValue(id, value, reason?)  // 调整价值
entryApi.listAdjustments(entryId)  // 获取调整历史
```

### records.ts

```typescript
type RecordType = 'income' | 'expense' | 'transfer';

interface FlowRecord {
  id: string;
  bookId: string;
  eventId: string | null;
  name: string | null;
  type: RecordType;
  amount: number;
  categoryId: string | null;
  fromAccountId: string | null;  // 支出/转账来源
  toAccountId: string | null;    // 收入/转账去向
  tagIds: string[];
  note: string | null;
  happenedAt: string;
}

// 流水 API
recordApi.list(bookId, filter?)    // 列出流水
recordApi.get(id)                  // 获取单条流水
recordApi.create(params)           // 创建流水
recordApi.update(id, params)       // 更新流水
recordApi.delete(id)               // 删除流水

// 事件 API
eventApi.list(bookId)              // 列出事件
eventApi.get(id)                   // 获取事件详情（含旗下流水）
eventApi.create(params)            // 创建事件
eventApi.update(id, params)        // 更新事件
eventApi.delete(id)                // 删除事件

// 混合 Feed（事件 + 独立流水）
feedApi.list(bookId, sort?)        // sort: { sortBy: 'date'|'amount', sortOrder: 'asc'|'desc' }
```

### categories.ts

```typescript
type CategoryDomain = 'asset' | 'liability' | 'income' | 'expense';

interface Category {
  id: string;
  domain: CategoryDomain;
  level: 1 | 2;
  parentId: string | null;  // level=2 时必填
  name: string;
  icon: string | null;
}

categoryApi.list(domain, level?)   // 列出分类
categoryApi.create(domain, level, name, icon?, parentId?)  // 创建分类
categoryApi.update(id, params)     // 更新分类
categoryApi.remove(id)             // 删除分类

// 标签
tagApi.list(domain?)               // 列出标签
tagApi.create(domain, name, color?) // 创建标签
tagApi.update(id, params)          // 更新标签
tagApi.remove(id)                  // 删除标签
```

### views.ts

```typescript
interface BookStats {
  totalAssets: number;
  totalLiabilities: number;
  netWorth: number;
  income: number;
  expense: number;
  other: number;  // 不计收支类
  incomeByCategory: CategoryStat[];
  expenseByCategory: CategoryStat[];
}

interface Snapshot {
  id: string;
  bookId: string;
  source: 'auto' | 'manual';
  netWorth: number;
  totalAssets: number;
  totalLiabilities: number;
  data: SnapshotData;
  createdAt: string;
}

// 统计
statsApi.getBookStats(bookId, from, to)  // from/to: YYYY-MM-DD

// 快照
snapshotApi.list(bookId, from?, to?)
snapshotApi.get(id)
snapshotApi.create(bookId)
snapshotApi.diff(fromId, toId)           // 快照对比

// 快照任务
snapshotApi.listTasks(bookId?)
snapshotApi.createTask(bookId, frequency)  // frequency: 'daily'|'weekly'|'monthly'
snapshotApi.updateTask(id, params)
snapshotApi.deleteTask(id)
```

### export_import.ts

```typescript
exportBook(bookId): Promise<string>           // 返回 JSON 字符串
importBook(jsonData, newName?): Promise<Book> // 导入账本
shareOrDownloadJson(content, filename)        // 分享/下载
```

## 后端 Tauri 命令

所有命令定义在 `src-tauri/src/lib.rs`。

### 账本
- `list_books` → `Vec<Book>`
- `create_book(name)` → `Book`
- `update_book(id, name)` → `Book`
- `delete_book(id)` → `()`

### 条目
- `list_entries(book_id, filter?)` → `Vec<Entry>`
- `get_entry(id)` → `Entry`
- `create_entry(params)` → `Entry`
- `update_entry(id, params)` → `Entry`
- `delete_entry(id)` → `()`
- `adjust_entry_value(id, new_value, reason?)` → `()`
- `list_entry_adjustments(entry_id)` → `Vec<EntryAdjustment>`

### 流水
- `list_records(book_id, filter?)` → `Vec<Record>`
- `get_record(id)` → `Record`
- `create_record(params)` → `Record`
- `update_record(id, params)` → `Record`
- `delete_record(id)` → `()`
- `list_feed(book_id, sort?)` → `Vec<FeedItem>`

### 事件
- `list_events(book_id)` → `Vec<Event>`
- `get_event(id)` → `EventWithRecords`
- `create_event(params)` → `Event`
- `update_event(id, params)` → `Event`
- `delete_event(id)` → `()`

### 分类与标签
- `list_categories(domain, level?)` → `Vec<Category>`
- `create_category(domain, level, name, parent_id?, icon?)` → `Category`
- `update_category(id, name?, icon?, parent_id?)` → `Category`
- `delete_category(id)` → `()`
- `list_tags(domain?)` → `Vec<Tag>`
- `create_tag(domain, name, color?)` → `Tag`
- `update_tag(id, name?, color?)` → `Tag`
- `delete_tag(id)` → `()`

### 统计与快照
- `get_book_stats(book_id, from, to)` → `BookStats`
- `list_snapshots(book_id, from?, to?)` → `Vec<Snapshot>`
- `get_snapshot(id)` → `Snapshot`
- `create_snapshot(book_id)` → `Snapshot`
- `diff_snapshots(from_id, to_id)` → `Vec<SnapshotDiffEntry>`
- `list_snapshot_tasks(book_id?)` → `Vec<SnapshotTask>`
- `get_snapshot_task_for_book(book_id)` → `Option<SnapshotTask>`
- `create_snapshot_task(book_id, frequency)` → `SnapshotTask`
- `update_snapshot_task(id, frequency?, is_active?)` → `SnapshotTask`
- `delete_snapshot_task(id)` → `()`
- `check_and_run_snapshot_tasks()` → `()`

### 导入导出
- `export_book(book_id)` → `String` (JSON)
- `import_book(json_data, new_name?)` → `Book`

## 数据库表结构

```sql
-- 账本
books(id, name, created_at)

-- 条目（资产/负债）
entries(id, book_id, name, kind, is_account, valuation_type, value,
        category_l1_id, category_l2_id, extra, opened_at, closed_at)

-- 条目标签关联
entry_tags(entry_id, tag_id)

-- 条目价值调整历史
entry_adjustments(id, entry_id, old_value, new_value, reason, adjusted_at)

-- 事件（流水分组）
events(id, book_id, name, description, created_at)

-- 流水记录
records(id, book_id, event_id, name, type, amount, category_id,
        from_account_id, to_account_id, note, happened_at, created_at)

-- 流水标签关联
record_tags(record_id, tag_id)

-- 两级分类
categories(id, domain, level, parent_id, name, icon)

-- 标签
tags(id, domain, name, color)

-- 净值快照
snapshots(id, book_id, type, base_snapshot_id, data, source, created_at)

-- 自动快照任务
snapshot_tasks(id, book_id, frequency, last_run_at, is_active)
```
