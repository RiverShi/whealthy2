#!/usr/bin/env python3
"""
补全分类和标签：为已有条目/记录添加分类信息
"""
import json, http.client

HOST, PORT = "127.0.0.1", 3030
BID = "ecaf33af-a900-4b17-84f2-b49a06242da7"
_id = 0

def call(method, **args):
    global _id
    _id += 1
    payload = json.dumps({
        "jsonrpc": "2.0", "method": "tools/call",
        "params": {"name": method, "arguments": args}, "id": _id
    }, ensure_ascii=False).encode("utf-8")
    conn = http.client.HTTPConnection(HOST, PORT, timeout=15)
    conn.request("POST", "/mcp", body=payload,
                 headers={"Content-Type": "application/json; charset=utf-8",
                          "Content-Length": str(len(payload))})
    resp = conn.getresponse()
    data = json.loads(resp.read().decode("utf-8"))
    conn.close()
    if "error" in data:
        raise RuntimeError(data["error"])
    result = json.loads(data["result"]["content"][0]["text"])
    if isinstance(result, dict) and result.get("isError"):
        raise RuntimeError(result)
    return result

def step(msg): print(f"\n{'─'*55}\n▶  {msg}")

# ── 1. 先删测试分类 ────────────────────────────────────────────────────────────
step("清理测试数据")
for domain in ["income", "expense", "asset", "liability"]:
    cats = call("list_categories", domain=domain)
    for c in cats:
        if c["name"] == "__test__":
            call("delete_category", id=c["id"])
            print(f"  ✓ 删除 _test_ ({domain})")

# ── 2. 创建分类 ────────────────────────────────────────────────────────────────
step("创建收入分类")
income_cats = {}
for name in ["研究生补助", "家庭支持", "其他收入", "返现优惠"]:
    c = call("create_category", domain="income", level=1, name=name)
    income_cats[name] = c["id"]
    print(f"  ✓ [{c['id'][:8]}] {name}")

step("创建支出分类")
expense_cats = {}
for name in ["日常餐饮", "社交文娱", "交通旅行", "购物消费", "其他支出"]:
    c = call("create_category", domain="expense", level=1, name=name)
    expense_cats[name] = c["id"]
    print(f"  ✓ [{c['id'][:8]}] {name}")

step("创建资产分类")
asset_cats = {}
for name in ["银行存款", "电子钱包", "证券投资", "债权", "实物资产"]:
    c = call("create_category", domain="asset", level=1, name=name)
    asset_cats[name] = c["id"]
    print(f"  ✓ [{c['id'][:8]}] {name}")

step("创建负债分类")
liab_cats = {}
for name in ["信用卡", "消费贷款"]:
    c = call("create_category", domain="liability", level=1, name=name)
    liab_cats[name] = c["id"]
    print(f"  ✓ [{c['id'][:8]}] {name}")

# ── 3. 更新条目分类 ────────────────────────────────────────────────────────────
# 条目名称 → (kind, isAccount, value_from_import, category)
ENTRY_MAP = {
    "支付宝":             ("asset",  True,  13270, asset_cats),
    "微信":               ("asset",  True,  6750,  asset_cats),
    "工商银行储蓄卡":     ("asset",  True,  1050,  asset_cats),
    "招商银行储蓄卡":     ("asset",  True,  7460,  asset_cats),
    "浦发银行":           ("asset",  True,  8,     asset_cats),
    "京东金融":           ("asset",  True,  1440,  asset_cats),
    "现金":               ("asset",  True,  1300,  asset_cats),
    "东方财富证券":       ("asset",  False, 197374, asset_cats),
    "银河证券":           ("asset",  False, 49922,  asset_cats),
    "黄金 22g":           ("asset",  False, 16764,  asset_cats),
    "京东e卡":            ("asset",  False, 560,    asset_cats),
    "韩币（约¥248）":     ("asset",  False, 248,    asset_cats),
    "王海 · iPad 6s Pro 分期（111.1×23）": ("asset", False, 2555.30, asset_cats),
    "王海 · 小米14 分期（187.5×4）":       ("asset", False, 750,     asset_cats),
    "罗奕昂 · Surface Pro 分期（275.86×2）": ("asset", False, 551.72, asset_cats),
    "刘再华":             ("asset",  False, 230,    asset_cats),
    "冉磊":               ("asset",  False, 300,    asset_cats),
    "花呗":               ("liability", False, 128,   liab_cats),
    "招商信用卡":         ("liability", False, 33376, liab_cats),
    "招行闪电贷":         ("liability", False, 80000, liab_cats),
    "工行信用卡":         ("liability", False, 6982,  liab_cats),
    "中行信用卡":         ("liability", False, 1,     liab_cats),
    "农行信用卡":         ("liability", False, 4745,  liab_cats),
}
# 条目名称 → 分类名
ENTRY_CAT = {
    "支付宝": "电子钱包", "微信": "电子钱包",
    "工商银行储蓄卡": "银行存款", "招商银行储蓄卡": "银行存款", "浦发银行": "银行存款",
    "京东金融": "电子钱包", "现金": "电子钱包",
    "东方财富证券": "证券投资", "银河证券": "证券投资",
    "黄金 22g": "实物资产", "京东e卡": "实物资产", "韩币（约¥248）": "实物资产",
    "王海 · iPad 6s Pro 分期（111.1×23）": "债权",
    "王海 · 小米14 分期（187.5×4）": "债权",
    "罗奕昂 · Surface Pro 分期（275.86×2）": "债权",
    "刘再华": "债权", "冉磊": "债权",
    "花呗": "消费贷款", "招行闪电贷": "消费贷款",
    "招商信用卡": "信用卡", "工行信用卡": "信用卡", "中行信用卡": "信用卡", "农行信用卡": "信用卡",
}

step("更新条目分类")
entries = call("list_entries", bookId=BID)
updated = 0
for e in entries:
    name = e["name"]
    if name not in ENTRY_MAP:
        print(f"  ⚠ 未找到映射: [{e['id'][:8]}] {name}")
        continue
    kind, is_acc, val, cat_dict = ENTRY_MAP[name]
    cat_name = ENTRY_CAT.get(name)
    cat_id = cat_dict.get(cat_name) if cat_name else None
    call("update_entry",
         id=e["id"], bookId=BID, name=name, kind=kind,
         isAccount=is_acc, value=e["value"],
         valuationType=e["valuationType"],
         categoryL1Id=cat_id)
    print(f"  ✓ {name} → {cat_name}")
    updated += 1
print(f"  合计更新 {updated} 条")

# ── 4. 更新记录分类 ────────────────────────────────────────────────────────────
# 备注 → 分类名
RECORD_CAT = {
    "研究生月补助": ("income", "研究生补助"),
    "生活费（父亲）": ("income", "家庭支持"),
    "其他小额进账": ("income", "其他收入"),
    "购物返现": ("income", "返现优惠"),
    "股市投入（转入证券账户）": ("expense", "其他支出"),
    # 旅游
    "团费": ("expense", "交通旅行"),
    "Olive Young 化妆品": ("expense", "购物消费"),
    "高丽参": ("expense", "购物消费"),
    "化妆品": ("expense", "购物消费"),
    "韩国境内交通": ("expense", "交通旅行"),
    "韩国餐饮": ("expense", "日常餐饮"),
    "漫游通信费": ("expense", "其他支出"),
    # 日常
    "校园卡充值": ("expense", "日常餐饮"),
    "其他餐饮果蔬": ("expense", "日常餐饮"),
    "湘菜·定燚（4.25）": ("expense", "社交文娱"),
    "定燚聚餐（4.25）": ("expense", "社交文娱"),
    "原神月卡": ("expense", "社交文娱"),
    "洗浴": ("expense", "社交文娱"),
    "打车": ("expense", "交通旅行"),
    "太阳岛": ("expense", "交通旅行"),
    "京东购物": ("expense", "购物消费"),
    "意外险（4月）": ("expense", "其他支出"),
    "Apple Care+ 年费": ("expense", "其他支出"),
    "按摩卡充值": ("expense", "其他支出"),
    "拍照": ("expense", "其他支出"),
    "话费充值": ("expense", "其他支出"),
    "其他杂项": ("expense", "其他支出"),
    "韩币兑换（4万韩元）": ("expense", "其他支出"),
}

step("更新记录分类")
records = call("list_records", bookId=BID)
updated = 0
no_match = []
for r in records:
    note = r.get("note") or ""
    if note not in RECORD_CAT:
        no_match.append(f"[{r['id'][:8]}] {note}")
        continue
    rtype, cat_name = RECORD_CAT[note]
    cat_dict = income_cats if rtype == "income" else expense_cats
    cat_id = cat_dict.get(cat_name)
    call("update_record", id=r["id"], categoryId=cat_id)
    print(f"  ✓ {note[:30]} → {cat_name}")
    updated += 1

if no_match:
    print(f"\n  ⚠ 未找到分类映射（{len(no_match)} 条）:")
    for n in no_match:
        print(f"    {n}")
print(f"\n  合计更新 {updated} 条")

print("\n\n✅ 分类补全完成！")
