# 后端 API 测试指南

## 测试环境设置

### 1. 项目启动
```bash
pnpm tauri dev
```

### 2. 数据库位置
- macOS: `~/Library/Application Support/com.river.wealthy/wealthy.db`
- 数据库会在应用启动时自动创建

## API 功能测试（2026年1-2月数据）

### 1. 账本管理测试

#### 1.1 创建账本
- **API**: `create_book`
- **参数**: `{name: "2026年1月账本"}`
- **时间**: 2026-01-01
- **期望结果**: 返回包含 id、name、createdAt 的 Book 对象

#### 1.2 列出账本
- **API**: `list_books`
- **期望结果**: 包含刚才创建的账本

#### 1.3 更新账本
- **API**: `update_book`
- **参数**: `{id: book_id, name: "2026年1-2月综合账本"}`
- **期望结果**: 返回更新后的账本对象

#### 1.4 删除账本
- **API**: `delete_book`
- **参数**: `{id: book_id}`
- **期望结果**: 成功删除，列表中不再包含该账本

### 2. 资产/负债条目测试（2026年1-2月）

#### 2.1 创建资产条目
- **API**: `create_entry`
- **参数**:
  ```javascript
  {
    book_id: "xxx",
    name: "现金",
    kind: "asset",
    is_account: true,
    valuation_type: "manual",
    value: 10000,
    opened_at: "2026-01-01"
  }
  ```
- **时间**: 2026-01-01
- **期望结果**: 成功创建资产条目

#### 2.2 创建负债条目
- **API**: `create_entry`
- **参数**:
  ```javascript
  {
    book_id: "xxx",
    name: "信用卡欠款",
    kind: "liability",
    is_account: true,
    valuation_type: "manual",
    value: 5000,
    opened_at: "2026-01-01"
  }
  ```
- **时间**: 2026-01-01
- **期望结果**: 成功创建负债条目

#### 2.3 调整条目价值
- **API**: `adjust_entry_value`
- **参数**: `{id: entry_id, new_value: 12000, reason: "工资收入"}`
- **时间**: 2026-01-15
- **期望结果**: 价值调整成功，创建调整历史

#### 2.4 查看调整历史
- **API**: `list_entry_adjustments`
- **参数**: `{entry_id: "xxx"}`
- **期望结果**: 返回调整历史记录

### 3. 流水记录测试（2026年1-2月）

#### 3.1 收入记录 - 1月工资
- **API**: `create_record`
- **参数**:
  ```javascript
  {
    book_id: "xxx",
    type: "income",
    amount: 8000,
    happened_at: "2026-01-15",
    category_id: "salary_category_id",  // 需要先创建分类
    to_account_id: "cash_entry_id",    // 现金账户
    note: "1月工资收入"
  }
  ```

#### 3.2 支出记录 - 餐饮
- **API**: `create_record`
- **参数**:
  ```javascript
  {
    book_id: "xxx",
    type: "expense",
    amount: 200,
    happened_at: "2026-01-20",
    category_id: "food_category_id",   // 需要先创建分类
    from_account_id: "cash_entry_id", // 从现金支出
    note: "朋友聚餐"
  }
  ```

#### 3.3 转账记录 - 银行转账
- **API**: `create_record`
- **参数**:
  ```javascript
  {
    book_id: "xxx",
    type: "transfer",
    amount: 3000,
    happened_at: "2026-01-25",
    from_account_id: "cash_entry_id",
    to_account_id: "bank_entry_id",    // 需要先创建银行账户条目
    note: "现金存入银行"
  }
  ```

#### 3.4 2月记录
- 类似创建2月份的收支记录

### 4. 分类管理测试（2026年使用）

#### 4.1 创建收入分类
- **API**: `create_category`
- **参数**: `{domain: "income", level: 1, name: "工资", icon: null}`

#### 4.2 创建支出分类
- **API**: `create_category`
- **参数**: `{domain: "expense", level: 1, name: "餐饮", icon: null}`

#### 4.3 创建资产分类
- **API**: `create_category`
- **参数**: `{domain: "asset", level: 1, name: "现金", icon: null}`

### 5. 事件管理测试（2026年聚合记录）

#### 5.1 创建事件
- **API**: `create_event`
- **参数**: `{book_id: "xxx", name: "春节聚餐", description: "大年初一聚餐费用"}`

#### 5.2 将多条记录关联到事件
- **API**: `update_record`
- **参数**: `{id: record_id, event_id: event_id}`

### 6. 快照功能测试（2026年1月底）

#### 6.1 创建快照
- **API**: `create_snapshot`
- **参数**: `{book_id: "xxx"}`
- **时间**: 2026-01-31
- **期望结果**: 创建月初至今的资产状况快照

#### 6.2 比较快照
- **API**: `diff_snapshots`
- **参数**: `{from_id: snapshot1_id, to_id: snapshot2_id}`
- **期望结果**: 返回两个时间点间的资产变化

## 测试步骤

1. 启动应用: `pnpm tauri dev`
2. 按上述顺序依次测试各项功能
3. 验证每个 API 调用的返回结果
4. 检查数据库中的数据是否正确保存
5. 验证跨时间范围查询的结果准确性（如查询2026年1-2月的数据）

## 预期结果

- 所有 API 调用都返回成功状态
- 数据正确保存到数据库
- 跨时间查询能正确返回对应时间段的数据
- 快照功能正确记录资产变化
- 归档功能（如果可用）能正确标记账本为只读状态