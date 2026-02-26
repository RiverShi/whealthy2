// 自动化测试脚本 - 填充2025年4月财务数据

// 请注意：此脚本应在Tauri应用内部运行（在前端JavaScript环境中）
// 不能在Node.js环境中独立运行

const financialData2025April = {
  // 账本设置
  book: {
    name: "2025年4-5月账本"
  },

  // 资产条目
  assets: [
    { name: "支付宝", value: 13270, category: "现金及银行" },
    { name: "微信", value: 6750, category: "现金及银行" },
    { name: "工商银行", value: 1050, category: "现金及银行" },
    { name: "招商银行", value: 7460, category: "现金及银行" },
    { name: "浦发银行", value: 8, category: "现金及银行" },
    { name: "京东金融", value: 1440, category: "现金及银行" },
    { name: "现金", value: 1300, category: "现金及银行" },
    { name: "黄金", value: 16764, category: "投资资产" }, // 22克按762元/克估算
    { name: "饮月金钞", value: 76, category: "投资资产" }, // 1g黄金按76元/g估算
    { name: "京东e卡", value: 560, category: "其他资产" },
    { name: "韩币", value: 248, category: "其他资产" } // 49000韩元按248元估算
  ],

  // 债权条目
  receivables: [
    { name: "王海-小米平板", value: 111.1 * 23, debtor: "王海" },
    { name: "王海-小米14", value: 187.5 * 4, debtor: "王海" },
    { name: "罗奕昂-Surface Pro", value: 275.86 * 2, debtor: "罗奕昂" },
    { name: "刘再华", value: 230, debtor: "刘再华" },
    { name: "冉磊", value: 300, debtor: "冉磊" }
  ],

  // 债务条目
  liabilities: [
    { name: "花呗", value: 128 },
    { name: "京东白条", value: 0 },
    { name: "招商信用卡", value: 33376 },
    { name: "招行闪电贷", value: 80000 },
    { name: "工行信用卡", value: 6982 },
    { name: "中行信用卡", value: 1 },
    { name: "农行信用卡", value: 4745 }
  ],

  // 收入记录
  incomes: [
    { description: "研究生月补助", amount: 600, date: "2025-04-01" },
    { description: "生活费（父亲）", amount: 2000, date: "2025-04-01" },
    { description: "其他小额进账", amount: 100, date: "2025-04-15" },
    { description: "返现", amount: 180, date: "2025-04-20" }
  ],

  // 支出记录（4.1-5.4期间总计：8572）
  expenses: [
    // 日常餐饮 -1204
    { description: "日常餐饮", amount: 1204, category: "餐饮", date: "2025-04-10" },
    // 社交文娱 -149
    { description: "湘菜-定燚", amount: 41, category: "社交", date: "2025-04-20" },
    { description: "4.25-定燚", amount: 38, category: "社交", date: "2025-04-25" },
    { description: "原神月卡", amount: 30, category: "娱乐", date: "2025-04-01" },
    { description: "洗浴", amount: 40, category: "社交", date: "2025-04-15" },
    // 交通旅行 -5842
    { description: "打车", amount: 156, category: "交通", date: "2025-04-20" },
    { description: "韩国旅游", amount: 5658, category: "旅游", date: "2025-04-29" },
    { description: "太阳岛", amount: 28, category: "交通", date: "2025-05-01" },
    // 购物类 -54
    { description: "京东购物", amount: 54, category: "购物", date: "2025-04-25" },
    // 其他 -1122
    { description: "意外险", amount: 142, category: "保险", date: "2025-04-01" },
    { description: "Apple Care+ 年费", amount: 548, category: "电子设备", date: "2025-04-15" },
    { description: "按摩卡充值", amount: 300, category: "健康", date: "2025-04-20" },
    { description: "拍照", amount: 50, category: "娱乐", date: "2025-04-25" },
    { description: "话费", amount: 30, category: "通讯", date: "2025-05-01" },
    { description: "其他", amount: 50, category: "其他", date: "2025-05-01" },
    { description: "韩币兑换", amount: 203, category: "汇兑", date: "2025-04-20" }
  ],

  // 特殊记录
  specialTransactions: [
    { description: "股市投入", amount: 65000, date: "2025-04-01", type: "投资" }
  ]
};

// 以下是自动化脚本函数
async function populateFinancialData() {
  console.log("开始自动填充2025年4月财务数据...");

  try {
    // 1. 创建账本
    console.log("1. 创建账本...");
    const book = await window.__TAURI__.invoke('create_book', {
      name: financialData2025April.book.name
    });
    console.log("账本创建成功:", book);

    // 2. 创建分类（如果不存在）
    console.log("2. 设置分类...");
    await setupCategories();

    // 3. 创建资产条目
    console.log("3. 创建资产条目...");
    const assetEntries = [];
    for (const asset of financialData2025April.assets) {
      const entry = await window.__TAURI__.invoke('create_entry', {
        params: {
          book_id: book.id,
          name: asset.name,
          kind: 'asset',
          is_account: true,
          valuation_type: 'manual',
          value: asset.value,
          opened_at: '2025-04-01',
          closed_at: null
        }
      });
      assetEntries.push(entry);
      console.log(`  创建资产: ${asset.name} = ${asset.value}`);
    }

    // 4. 创建债务条目
    console.log("4. 创建债务条目...");
    const liabilityEntries = [];
    for (const liability of financialData2025April.liabilities) {
      if (liability.value > 0) { // 只创建有价值（正数）的债务
        const entry = await window.__TAURI__.invoke('create_entry', {
          params: {
            book_id: book.id,
            name: liability.name,
            kind: 'liability',
            is_account: true,
            valuation_type: 'manual',
            value: liability.value,
            opened_at: '2025-04-01',
            closed_at: null
          }
        });
        liabilityEntries.push(entry);
        console.log(`  创建债务: ${liability.name} = ${liability.value}`);
      }
    }

    // 5. 创建收入记录
    console.log("5. 创建收入记录...");
    for (const income of financialData2025April.incomes) {
      // 这里需要查找对应的收入账户（资产）
      const accountEntry = assetEntries[0]; // 简化：使用第一个资产账户
      await window.__TAURI__.invoke('create_record', {
        params: {
          book_id: book.id,
          type: 'income',
          amount: income.amount,
          happened_at: income.date + 'T12:00:00.000Z',
          category_id: 'income_salary', // 需要根据实际分类ID调整
          to_account_id: accountEntry.id,
          note: income.description
        }
      });
      console.log(`  创建收入: ${income.description} = ${income.amount}`);
    }

    // 6. 创建支出记录
    console.log("6. 创建支出记录...");
    for (const expense of financialData2025April.expenses) {
      // 这里需要查找对应的支出账户（资产）
      const accountEntry = assetEntries[0]; // 简化：使用第一个资产账户
      await window.__TAURI__.invoke('create_record', {
        params: {
          book_id: book.id,
          type: 'expense',
          amount: expense.amount,
          happened_at: expense.date + 'T12:00:00.000Z',
          category_id: 'expense_general', // 需要根据实际分类ID调整
          from_account_id: accountEntry.id,
          note: expense.description
        }
      });
      console.log(`  创建支出: ${expense.description} = ${expense.amount}`);
    }

    console.log("数据填充完成！");

  } catch (error) {
    console.error("数据填充失败:", error);
  }
}

async function setupCategories() {
  // 创建一些常用的分类
  try {
    // 收入分类
    await window.__TAURI__.invoke('create_category', {
      domain: 'income',
      level: 1,
      name: '工资',
      parent_id: null,
      icon: '💰'
    });

    await window.__TAURI__.invoke('create_category', {
      domain: 'income',
      level: 1,
      name: '补贴',
      parent_id: null,
      icon: '🎓'
    });

    // 支出分类
    await window.__TAURI__.invoke('create_category', {
      domain: 'expense',
      level: 1,
      name: '餐饮',
      parent_id: null,
      icon: '🍜'
    });

    await window.__TAURI__.invoke('create_category', {
      domain: 'expense',
      level: 1,
      name: '交通',
      parent_id: null,
      icon: '🚗'
    });

    await window.__TAURI__.invoke('create_category', {
      domain: 'expense',
      level: 1,
      name: '旅游',
      parent_id: null,
      icon: '✈️'
    });

    await window.__TAURI__.invoke('create_category', {
      domain: 'expense',
      level: 1,
      name: '购物',
      parent_id: null,
      icon: '🛍️'
    });

    await window.__TAURI__.invoke('create_category', {
      domain: 'expense',
      level: 1,
      name: '娱乐',
      parent_id: null,
      icon: '🎮'
    });

    console.log("分类设置完成");
  } catch (error) {
    // 如果分类已存在会报错，这是正常的
    console.log("分类设置完成（可能部分已存在）");
  }
}

// 导出函数供外部调用
window.populateFinancialData = populateFinancialData;

console.log("自动化数据填充脚本已加载。调用 window.populateFinancialData() 开始填充数据。");