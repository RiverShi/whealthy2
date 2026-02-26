/*
自动化测试脚本 - 用于填充2025年4月财务数据到wealthy应用

使用方法：
1. 在应用启动状态下打开开发者工具
2. 复制此文件的所有内容
3. 在控制台粘贴并执行
4. 然后执行：populateDataForApril2025()
*/

async function populateDataForApril2025() {
  console.log("开始填充2025年4月财务数据...");

  try {
    // 1. 创建主账本
    console.log("创建账本...");
    const book = await invoke('create_book', { name: '2025年4-5月账本' });
    console.log("账本ID:", book.id);

    // 2. 创建主要资产
    console.log("创建资产条目...");

    // 现金及银行账户
    const alipay = await invoke('create_entry', {
      params: {
        book_id: book.id,
        name: '支付宝',
        kind: 'asset',
        is_account: true,
        valuation_type: 'manual',
        value: 13270,
        opened_at: '2025-04-01',
        closed_at: null
      }
    });
    console.log("支付宝账户创建成功，当前值:", alipay.value);

    const wechat = await invoke('create_entry', {
      params: {
        book_id: book.id,
        name: '微信',
        kind: 'asset',
        is_account: true,
        valuation_type: 'manual',
        value: 6750,
        opened_at: '2025-04-01',
        closed_at: null
      }
    });
    console.log("微信账户创建成功，当前值:", wechat.value);

    // 3. 创建一些支出分类
    console.log("创建分类...");
    try {
      await invoke('create_category', {
        domain: 'expense',
        level: 1,
        name: '餐饮',
        parent_id: null,
        icon: '🍜'
      });
    } catch(e) {
      console.log("餐饮分类可能已存在");
    }

    try {
      await invoke('create_category', {
        domain: 'expense',
        level: 1,
        name: '交通',
        parent_id: null,
        icon: '🚗'
      });
    } catch(e) {
      console.log("交通分类可能已存在");
    }

    try {
      await invoke('create_category', {
        domain: 'expense',
        level: 1,
        name: '旅游',
        parent_id: null,
        icon: '✈️'
      });
    } catch(e) {
      console.log("旅游分类可能已存在");
    }

    // 4. 记录主要支出 - 韩国旅游 (5658元)
    console.log("记录韩国旅游支出...");
    await invoke('create_record', {
      params: {
        book_id: book.id,
        type: 'expense',
        amount: 5658, // 韩国旅游费用
        happened_at: '2025-04-29T10:00:00.000Z',
        category_id: null, // 后续可通过UI选择旅游分类
        from_account_id: alipay.id, // 从支付宝支出
        note: '韩国旅游（4.29-5.4），罗奕昂/段虹玲/左伊芮，团费3100 + 购物270 + 购物833 + 购物441 + 交通96 + 餐饮863 + 通信55'
      }
    });
    console.log("韩国旅游支出记录成功");

    // 5. 记录其他收入 - 研究生月补助
    console.log("记录收入...");
    await invoke('create_record', {
      params: {
        book_id: book.id,
        type: 'income',
        amount: 600, // 研究生月补助
        happened_at: '2025-04-01T10:00:00.000Z',
        category_id: null, // 可通过UI选择收入分类
        to_account_id: alipay.id, // 收入支付宝
        note: '研究生月补助'
      }
    });
    console.log("研究生月补助记录成功");

    // 6. 股市投入记录
    console.log("记录股市投入...");
    await invoke('create_record', {
      params: {
        book_id: book.id,
        type: 'expense',
        amount: 65000, // 股市投入
        happened_at: '2025-04-01T10:00:00.000Z',
        category_id: null, // 可通过UI选择投资分类
        from_account_id: alipay.id, // 从支付宝支出
        note: '股市投入'
      }
    });
    console.log("股市投入记录成功");

    console.log("✅ 2025年4月财务数据填充完成！");
    console.log("- 账本: 2025年4-5月账本");
    console.log("- 资产: 支付宝(13270)、微信(6750)");
    console.log("- 支出: 韩国旅游(5658)、股市投入(65000)");
    console.log("- 收入: 研究生月补助(600)");
    console.log("可在应用UI中查看和补充更多详细数据。");

  } catch (error) {
    console.error("❌ 数据填充失败:", error.message);
    console.error("错误详情:", error);
  }
}

console.log("📊 2025年4月财务数据自动填充脚本已准备就绪");
console.log("📖 使用方法: 在应用中打开控制台并运行 populateDataForApril2025()");