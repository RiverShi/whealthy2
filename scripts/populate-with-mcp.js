// 使用 MCP 适配器填充示例财务数据
// 该脚本使用 MCP 协议与 Wealthy 应用交互

const axios = require('axios');

class MCPWealthyPopulator {
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

  async populateSampleData() {
    console.log("💰 开始使用 MCP 适配器填充示例财务数据...");

    try {
      // 创建账本
      console.log("📝 创建账本...");
      const book = await this.call('create_book', { name: '2025年4-5月账本' });
      console.log(`✅ 创建账本: ${book.name} (ID: ${book.id})`);

      // 创建账户
      console.log("💳 创建账户...");
      const accounts = {};

      // 支付宝账户
      accounts.alipay = await this.call('create_entry', {
        book_id: book.id,
        name: '支付宝',
        kind: 'asset',
        is_account: true,
        valuation_type: 'manual',
        value: 13270,
        opened_at: '2025-04-01',
        closed_at: null
      });
      console.log(`✅ 创建账户: ${accounts.alipay.name}, 余额: ¥${accounts.alipay.value}`);

      // 微信账户
      accounts.wechat = await this.call('create_entry', {
        book_id: book.id,
        name: '微信',
        kind: 'asset',
        is_account: true,
        valuation_type: 'manual',
        value: 6750,
        opened_at: '2025-04-01',
        closed_at: null
      });
      console.log(`✅ 创建账户: ${accounts.wechat.name}, 余额: ¥${accounts.wechat.value}`);

      // 信用卡账户
      accounts.card = await this.call('create_entry', {
        book_id: book.id,
        name: '招商信用卡',
        kind: 'liability',
        is_account: true,
        valuation_type: 'manual',
        value: 33376,
        opened_at: '2025-04-01',
        closed_at: null
      });
      console.log(`✅ 创建账户: ${accounts.card.name}, 欠款: ¥${accounts.card.value}`);

      // 创建分类
      console.log("🏷️ 创建分类...");
      const categories = {};

      try {
        categories.food = await this.call('create_category', {
          domain: 'expense',
          level: 1,
          name: '餐饮',
          parent_id: null,
          icon: '🍜'
        });
        console.log(`✅ 创建分类: ${categories.food.name} ${categories.food.icon}`);
      } catch (e) {
        console.log("⚠️ 餐饮分类可能已存在");
      }

      try {
        categories.travel = await this.call('create_category', {
          domain: 'expense',
          level: 1,
          name: '旅游',
          parent_id: null,
          icon: '✈️'
        });
        console.log(`✅ 创建分类: ${categories.travel.name} ${categories.travel.icon}`);
      } catch (e) {
        console.log("⚠️ 旅游分类可能已存在");
      }

      // 记录交易
      console.log("📝 记录交易...");

      // 韩国旅游 (5658元)
      await this.call('create_record', {
        book_id: book.id,
        type: 'expense',
        amount: 5658,
        happened_at: '2025-04-29T12:00:00.000Z',
        category_id: categories.travel?.id || null,
        from_account_id: accounts.alipay.id,
        note: '韩国旅游（4.29-5.4）团费3100购物270购物833购物441交通96餐饮863通信55'
      });
      console.log("✅ 记录韩国旅游支出: ¥5658");

      // 研究生月补助 (600元)
      await this.call('create_record', {
        book_id: book.id,
        type: 'income',
        amount: 600,
        happened_at: '2025-04-01T10:00:00.000Z',
        category_id: null,
        to_account_id: accounts.alipay.id,
        note: '研究生月补助'
      });
      console.log("✅ 记录研究生月补助: ¥600");

      // 股市投入 (65000元)
      await this.call('create_record', {
        book_id: book.id,
        type: 'expense',
        amount: 65000,
        happened_at: '2025-04-01T14:00:00.000Z',
        category_id: null,
        from_account_id: accounts.alipay.id,
        note: '股市投入'
      });
      console.log("✅ 记录股市投入: ¥65000");

      // 月度生活费 (2000元)
      await this.call('create_record', {
        book_id: book.id,
        type: 'income',
        amount: 2000,
        happened_at: '2025-04-05T10:00:00.000Z',
        category_id: null,
        to_account_id: accounts.wechat.id,
        note: '生活费（父亲）'
      });
      console.log("✅ 记录生活费: ¥2000");

      // 获取汇总信息
      console.log("\n📈 数据填充完成汇总:");
      console.log(`   - 账本: ${book.name}`);
      console.log(`   - 账户数量: ${Object.keys(accounts).length}`);

      // 获取创建的记录
      const records = await this.call('list_records', { book_id: book.id });
      console.log(`   - 交易记录: ${records.length} 笔`);

      // 计算总收入和支出
      const totalIncome = records
        .filter(r => r.type === 'income')
        .reduce((sum, r) => sum + r.amount, 0);
      const totalExpense = records
        .filter(r => r.type === 'expense')
        .reduce((sum, r) => sum + r.amount, 0);

      console.log(`   - 总收入: ¥${totalIncome}`);
      console.log(`   - 总支出: ¥${totalExpense}`);
      console.log(`   - 净资产变化: ¥${totalIncome - totalExpense}`);

      console.log("\n🎉 示例财务数据已成功通过 MCP 适配器填充！");
      console.log("💡 您现在可以通过 Wealthy 应用查看这些数据");

      return { book, accounts, categories, records };
    } catch (error) {
      console.error("❌ 数据填充失败:", error.message);
      throw error;
    }
  }
}

// 如果直接运行此脚本，则执行数据填充
if (require.main === module) {
  const populator = new MCPWealthyPopulator();

  // 检查 MCP 服务是否可用
  console.log("🔍 检查 MCP 服务是否运行...");

  populator.call('list_books')
    .then(() => {
      console.log("✅ MCP 服务可用，开始填充数据...");
      return populator.populateSampleData();
    })
    .catch(error => {
      console.error("❌ MCP 服务不可用，请确保 Wealthy 应用正在运行且 MCP 服务在 http://localhost:3030/mcp 可访问");
      console.error("   错误详情:", error.message);
    });
}

module.exports = MCPWealthyPopulator;