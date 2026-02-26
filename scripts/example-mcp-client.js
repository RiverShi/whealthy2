// 示例 MCP 客户端 - 展示如何通过 MCP 协议访问财务数据
// 此脚本演示了如何与 Wealthy 应用的 MCP 服务交互

const axios = require('axios');

class MCPClient {
  constructor(baseUrl = 'http://localhost:3030') {
    this.baseUrl = baseUrl;
    this.idCounter = 1;
  }

  async call(method, params = {}) {
    const requestId = this.idCounter++;
    const payload = {
      jsonrpc: "2.0",
      method,
      params,
      id: requestId
    };

    try {
      const response = await axios.post(`${this.baseUrl}/mcp`, payload, {
        headers: {
          'Content-Type': 'application/json'
        }
      });

      if (response.data.error) {
        throw new Error(`MCP Error: ${response.data.error.code} - ${response.data.error.message}`);
      }

      return response.data.result;
    } catch (error) {
      console.error(`MCP Call Failed:`, error.message);
      throw error;
    }
  }

  // ─── 账本操作 ────────────────────────────────────────────────────────────

  async listBooks() {
    return await this.call('list_books');
  }

  async createBook(name) {
    return await this.call('create_book', { name });
  }

  // ─── 条目操作 ────────────────────────────────────────────────────────────

  async listEntries(bookId) {
    return await this.call('list_entries', { book_id: bookId });
  }

  async createEntry(params) {
    return await this.call('create_entry', params);
  }

  // ─── 记录操作 ────────────────────────────────────────────────────────────

  async listRecords(bookId) {
    return await this.call('list_records', { book_id: bookId });
  }

  async createRecord(params) {
    return await this.call('create_record', params);
  }

  // ─── 分类操作 ────────────────────────────────────────────────────────────

  async listCategories(domain = 'expense', level = null) {
    const params = { domain };
    if (level !== null) {
      params.level = level;
    }
    return await this.call('list_categories', params);
  }

  async createCategory(params) {
    return await this.call('create_category', params);
  }

  // ─── 快照操作 ────────────────────────────────────────────────────────────

  async createSnapshot(bookId) {
    return await this.call('create_snapshot', { book_id: bookId });
  }

  async listSnapshots(bookId) {
    return await this.call('list_snapshots', { book_id: bookId });
  }
}

// 示例使用
async function demoWealthyMCP() {
  console.log("💰 Wealthy MCP 适配器示例");
  console.log("========================");

  const client = new MCPClient();

  try {
    // 1. 创建账本
    console.log("\n📝 创建新账本...");
    const book = await client.createBook("测试账本");
    console.log(`✅ 账本已创建: ${book.name} (${book.id})`);

    // 2. 列出账本
    console.log("\n📋 列出所有账本...");
    const books = await client.listBooks();
    console.log(`✅ 找到 ${books.length} 个账本:`);
    books.forEach(b => console.log(`  - ${b.name} (${b.id})`));

    // 3. 创建分类
    console.log("\n🏷️ 创建分类...");
    try {
      const foodCategory = await client.createCategory({
        domain: 'expense',
        level: 1,
        name: '餐饮',
        icon: '🍽️'
      });
      console.log(`✅ 餐饮分类已创建: ${foodCategory.name}`);

      const transportCategory = await client.createCategory({
        domain: 'expense',
        level: 1,
        name: '交通',
        icon: '🚗'
      });
      console.log(`✅ 交通分类已创建: ${transportCategory.name}`);
    } catch (err) {
      console.log(`⚠️ 分类创建可能已存在:`, err.message);
    }

    // 4. 列出分类
    console.log("\n🏷️ 列出分类...");
    const categories = await client.listCategories('expense', 1);
    console.log(`✅ 找到 ${categories.length} 个一级支出分类:`);
    categories.forEach(c => console.log(`  - ${c.name} (${c.icon})`));

    // 5. 创建账户条目
    console.log("\n💳 创建账户条目...");
    const alipayAccount = await client.createEntry({
      book_id: book.id,
      name: '支付宝',
      kind: 'asset',
      is_account: true,
      valuation_type: 'manual',
      value: 5000,
      opened_at: '2025-01-01',
      closed_at: null
    });
    console.log(`✅ 账户已创建: ${alipayAccount.name}, 余额 ¥${alipayAccount.value}`);

    const creditCardAccount = await client.createEntry({
      book_id: book.id,
      name: '信用卡',
      kind: 'liability',
      is_account: true,
      valuation_type: 'manual',
      value: 2000,
      opened_at: '2025-01-01',
      closed_at: null
    });
    console.log(`✅ 账户已创建: ${creditCardAccount.name}, 欠款 ¥${creditCardAccount.value}`);

    // 6. 记录收支
    console.log("\n📝 记录收支...");
    const expenseRecord = await client.createRecord({
      book_id: book.id,
      type: 'expense',
      amount: 45.5,
      happened_at: '2025-01-15T12:00:00.000Z',
      from_account_id: alipayAccount.id,
      category_id: categories.find(c => c.name === '餐饮')?.id || null,
      note: '午餐费用'
    });
    console.log(`✅ 支出已记录: ¥${expenseRecord.amount} (${expenseRecord.note})`);

    const incomeRecord = await client.createRecord({
      book_id: book.id,
      type: 'income',
      amount: 5000,
      happened_at: '2025-01-01T09:00:00.000Z',
      to_account_id: alipayAccount.id,
      note: '工资收入'
    });
    console.log(`✅ 收入已记录: ¥${incomeRecord.amount} (${incomeRecord.note})`);

    // 7. 列出记录
    console.log("\n📋 列出流水记录...");
    const records = await client.listRecords(book.id);
    console.log(`✅ 找到 ${records.length} 条记录:`);
    records.forEach(r => {
      const direction = r.type === 'income' ? '收入' : r.type === 'expense' ? '支出' : '转账';
      console.log(`  - ${direction}: ¥${r.amount} (${r.note || '无备注'})`);
    });

    // 8. 列出条目
    console.log("\n📊 列出账户条目...");
    const entries = await client.listEntries(book.id);
    console.log(`✅ 找到 ${entries.length} 个条目:`);
    entries.forEach(e => {
      const type = e.is_account ? '账户' : e.kind;
      console.log(`  - ${e.name} (${type}): ¥${e.value}`);
    });

    // 9. 创建快照
    console.log("\n📸 创建快照...");
    try {
      const snapshot = await client.createSnapshot(book.id);
      console.log(`✅ 快照已创建: ${snapshot.id} (${snapshot.created_at})`);
    } catch (err) {
      console.log(`⚠️ 快照创建可能存在问题:`, err.message);
    }

    console.log("\n🎉 MCP 客户端示例执行完毕！");
    console.log("\n💡 提示:");
    console.log("   - MCP 服务运行在 http://localhost:3030/mcp");
    console.log("   - 支持的方法包括账本、条目、记录、分类、快照等操作");
    console.log("   - 可通过 JSON-RPC 2.0 协议进行调用");
  } catch (error) {
    console.error("❌ 执行失败:", error.message);
  }
}

// 如果直接运行此脚本，则执行演示
if (require.main === module) {
  demoWealthyMCP();
}

module.exports = MCPClient;