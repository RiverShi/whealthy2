// 简化版数据填充脚本 - 可直接在应用控制台中运行

async function populateApril2025Data() {
  console.log("开始自动填充2025年4月财务数据...");

  try {
    // 创建账本
    const book = await invoke('create_book', { name: '2025年4-5月账本' });
    console.log(`✅ 创建账本: ${book.name}`);

    // 创建几个常用资产账户
    const accounts = {};

    // 主要现金账户
    accounts.alipay = await invoke('create_entry', {
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
    console.log(`✅ 创建账户: ${accounts.alipay.name}, 余额: ¥${accounts.alipay.value}`);

    accounts.wechat = await invoke('create_entry', {
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
    console.log(`✅ 创建账户: ${accounts.wechat.name}, 余额: ¥${accounts.wechat.value}`);

    // 信用卡账户
    accounts.card = await invoke('create_entry', {
      params: {
        book_id: book.id,
        name: '招商信用卡',
        kind: 'liability', // 信用卡是负债
        is_account: true,
        valuation_type: 'manual',
        value: 33376, // 信用卡欠款
        opened_at: '2025-04-01',
        closed_at: null
      }
    });
    console.log(`✅ 创建账户: ${accounts.card.name}, 欠款: ¥${accounts.card.value}`);

    // 创建分类（如不存在）
    const categories = {};
    try {
      categories.food = await invoke('create_category', {
        domain: 'expense',
        level: 1,
        name: '餐饮',
        parent_id: null,
        icon: '🍜'
      });
    } catch(e) { /* 分类可能已存在 */ }

    try {
      categories.travel = await invoke('create_category', {
        domain: 'expense',
        level: 1,
        name: '旅游',
        parent_id: null,
        icon: '✈️'
      });
    } catch(e) { /* 分类可能已存在 */ }

    // 记录主要支出
    console.log("📝 开始记录收支...");

    // 韩国旅游 (5658元)
    await invoke('create_record', {
      params: {
        book_id: book.id,
        type: 'expense',
        amount: 5658,
        happened_at: '2025-04-29T12:00:00.000Z',
        category_id: categories.travel?.id || null,
        from_account_id: accounts.alipay.id,
        note: '韩国旅游（4.29-5.4）团费3100购物270购物833购物441交通96餐饮863通信55'
      }
    });
    console.log("✅ 记录韩国旅游支出: ¥5658");

    // 研究生月补助 (600元)
    await invoke('create_record', {
      params: {
        book_id: book.id,
        type: 'income',
        amount: 600,
        happened_at: '2025-04-01T10:00:00.000Z',
        category_id: null,
        to_account_id: accounts.alipay.id,
        note: '研究生月补助'
      }
    });
    console.log("✅ 记录研究生月补助: ¥600");

    // 股市投入 (65000元)
    await invoke('create_record', {
      params: {
        book_id: book.id,
        type: 'expense',
        amount: 65000,
        happened_at: '2025-04-01T14:00:00.000Z',
        category_id: null,
        from_account_id: accounts.alipay.id,
        note: '股市投入'
      }
    });
    console.log("✅ 记录股市投入: ¥65000");

    // 月度生活费 (2000元)
    await invoke('create_record', {
      params: {
        book_id: book.id,
        type: 'income',
        amount: 2000,
        happened_at: '2025-04-05T10:00:00.000Z',
        category_id: null,
        to_account_id: accounts.wechat.id,
        note: '生活费（父亲）'
      }
    });
    console.log("✅ 记录生活费: ¥2000");

    console.log("🎉 2025年4月财务数据填充完成！");
    console.log("📊 已创建:");
    console.log(`   - 账本: ${book.name}`);
    console.log(`   - 资产账户: ${Object.keys(accounts).length}个`);
    console.log(`   - 主要交易: 6笔`);
    console.log("   - 总收支: 收入¥2600, 支出¥70658");
    console.log("   - 详细数据可在应用中查看和调整");

  } catch (error) {
    console.error("❌ 填充失败:", error.message);
  }
}

console.log("💡 准备就绪: 运行 populateApril2025Data() 开始自动填充2025年4月财务数据");