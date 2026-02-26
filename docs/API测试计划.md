# API 测试计划

基于方案.md 中的设计文档，我们将测试以下后端 API：

## 1. 账本管理 API (2026年1-2月测试)

### 1.1 创建账本
- 调用 `create_book(name)`
- 测试参数：name = "2026年1月账本"
- 预期结果：返回带有 id、name、createdAt 的 Book 对象

### 1.2 获取账本列表
- 调用 `list_books()`
- 预期结果：返回包含刚才创建账本的列表

### 1.3 更新账本
- 调用 `update_book(id, name)`
- 测试参数：将账本名称更新为 "2026年2月账本"
- 预期结果：返回更新后的 Book 对象

### 1.4 (如果可用) 归档账本
- 调用 `archive_book(id)`
- 注意：根据之前的检查，此功能可能已被移除

### 1.5 删除账本
- 调用 `delete_book(id)`
- 预期结果：删除成功，列表中不再包含该账本

## 2. 资产/负债条目 API (2026年1-2月测试)

### 2.1 创建资产条目
- 调用 `create_entry(...)`
- 测试参数：创建一个现金资产，价值 10000
- 预期结果：返回创建的 Entry 对象

### 2.2 创建负债条目
- 调用 `create_entry(...)`
- 测试参数：创建一个信用卡负债，价值 5000
- 预期结果：返回创建的 Entry 对象

### 2.3 调整条目价值
- 调用 `adjust_entry_value(id, new_value, reason?)`
- 测试参数：将现金资产调整为 12000，原因："工资收入"
- 预期结果：价值调整成功，产生调整记录

### 2.4 查看调整历史
- 调用 `list_entry_adjustments(entry_id)`
- 预期结果：返回调整历史记录

## 3. 流水记录 API (2026年1-2月测试)

### 3.1 创建收入记录
- 调用 `create_record(...)`
- 测试参数：类型为 "income"，金额 8000，发生时间 "2026-01-15"
- 预期结果：返回创建的 Record 对象

### 3.2 创建支出记录
- 调用 `create_record(...)`
- 测试参数：类型为 "expense"，金额 200，发生时间 "2026-01-20"
- 预期结果：返回创建的 Record 对象

### 3.3 创建转账记录
- 调用 `create_record(...)`
- 测试参数：类型为 "transfer"，金额 1000，发生时间 "2026-02-01"
- 预期结果：返回创建的 Record 对象

### 3.4 查询流水记录
- 调用 `list_records(book_id, filter)`
- 测试参数：按时间段过滤 "2026-01-01" 到 "2026-01-31"
- 预期结果：返回符合时间范围的记录列表

## 4. 分类和标签 API (2026年1-2月测试)

### 4.1 创建收入分类
- 调用 `create_category(domain, level, name, ...)`
- 测试参数：domain = "income", level = 1, name = "工资"
- 预期结果：返回创建的 Category 对象

### 4.2 创建支出分类
- 调用 `create_category(domain, level, name, ...)`
- 测试参数：domain = "expense", level = 1, name = "餐饮"
- 预期结果：返回创建的 Category 对象

### 4.3 创建标签
- 调用 `create_tag(domain, name, ...)`
- 测试参数：domain = "transaction", name = "2026年1月"
- 预期结果：返回创建的 Tag 对象

## 5. 快照 API (2026年1-2月测试)

### 5.1 创建快照
- 调用 `create_snapshot(book_id)`
- 测试参数：对当前账本创建快照
- 预期结果：返回快照对象，记录当前净资产

### 5.2 获取快照列表
- 调用 `list_snapshots(book_id, from?, to?)`
- 测试参数：查询 2026 年的快照
- 预期结果：返回快照列表

### 5.3 比较快照
- 调用 `diff_snapshots(from_id, to_id)`
- 测试参数：比较两个不同时点的快照
- 预期结果：返回差异报告