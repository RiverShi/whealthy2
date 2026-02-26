#!/usr/bin/env python3
"""
将 example.md 中的 2025年4月财务数据导入 Wealthy（通过 MCP 接口）
"""
import json, http.client

HOST = "127.0.0.1"
PORT = 3030
_id = 0

def call(method, **args):
    global _id
    _id += 1
    payload = json.dumps({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {"name": method, "arguments": args},
        "id": _id
    }, ensure_ascii=False).encode("utf-8")
    conn = http.client.HTTPConnection(HOST, PORT, timeout=15)
    conn.request("POST", "/mcp", body=payload,
                 headers={"Content-Type": "application/json; charset=utf-8",
                          "Content-Length": str(len(payload))})
    resp = conn.getresponse()
    data = json.loads(resp.read().decode("utf-8"))
    conn.close()
    text = data["result"]["content"][0]["text"]
    result = json.loads(text)
    if isinstance(result, dict) and result.get("isError"):
        raise RuntimeError(text)
    return result

def ok(msg):  print(f"  ✓ {msg}")
def step(msg): print(f"\n{'─'*55}\n▶  {msg}")

# ─────────────────────────────────────────────────────────────────────────────
BID = "ecaf33af-a900-4b17-84f2-b49a06242da7"
print(f"[账本] 2025年个人财务  ID: {BID}")

# ── 资产条目：现金 & 账户 ─────────────────────────────────────────────────────
step("资产条目（现金 & 账户类，资金清算值 2025-05-04）")
cash = [
    ("支付宝",            13270, True),
    ("微信",               6750, True),
    ("工商银行储蓄卡",     1050, True),
    ("招商银行储蓄卡",     7460, True),
    ("浦发银行",              8, True),
    ("京东金融",           1440, True),
    ("现金",               1300, True),
]
accounts = {}
for name, val, is_acc in cash:
    e = call("create_entry", bookId=BID, name=name, kind="asset",
             isAccount=is_acc, value=val, valuationType="manual")
    accounts[name] = e["id"]
    ok(f"{name}  ¥{val:,.2f}")

step("资产条目（投资 & 实物）")
for name, val in [
    ("东方财富证券",  197374),
    ("银河证券",       49922),
    ("黄金 22g",       16764),
    ("京东e卡",          560),
    ("韩币（约¥248）",   248),
]:
    call("create_entry", bookId=BID, name=name, kind="asset",
         isAccount=False, value=val, valuationType="manual")
    ok(f"{name}  ¥{val:,.2f}")

step("资产条目（债权）")
for name, val in [
    ("王海 · iPad 6s Pro 分期（111.1×23）", 2555.30),
    ("王海 · 小米14 分期（187.5×4）",        750.00),
    ("罗奕昂 · Surface Pro 分期（275.86×2）", 551.72),
    ("刘再华",  230.00),
    ("冉磊",    300.00),
]:
    call("create_entry", bookId=BID, name=name, kind="asset",
         isAccount=False, value=val, valuationType="manual")
    ok(f"{name}  ¥{val:,.2f}")

step("负债条目（资金清算值 2025-05-04）")
for name, val in [
    ("花呗",         128),
    ("招商信用卡",  33376),
    ("招行闪电贷",  80000),
    ("工行信用卡",   6982),
    ("中行信用卡",      1),
    ("农行信用卡",   4745),
]:
    call("create_entry", bookId=BID, name=name, kind="liability",
         isAccount=False, value=val, valuationType="manual")
    ok(f"{name}  ¥{val:,.2f}")

step("收入记录（2025-04-01）")
for note, amt, to_acc in [
    ("研究生月补助",     600,  "微信"),
    ("生活费（父亲）",  2000,  "支付宝"),
    ("其他小额进账",     100,  "支付宝"),
    ("购物返现",         180,  "支付宝"),
]:
    call("create_record", bookId=BID, type="income", amount=amt,
         happenedAt="2025-04-01", note=note,
         toAccountId=accounts[to_acc])
    ok(f"+¥{amt:,.0f}  {note}")

step("特别出入项：股市投入 -¥65,000")
call("create_record", bookId=BID, type="expense", amount=65000,
     happenedAt="2025-04-01",
     fromAccountId=accounts["招商银行储蓄卡"],
     note="股市投入（转入证券账户）")
ok("-¥65,000  股市投入")

step("事件：韩国旅游 4.29-5.4（¥5,658）")
ev = call("create_event", bookId=BID, name="韩国旅游 4.29-5.4",
          description="与罗奕昂、段虹玲、左伊芮同行")
EID = ev["id"]
ok(f"事件 ID: {EID}")
for note, amt, date in [
    ("团费",                  3100, "2025-04-29"),
    ("Olive Young 化妆品",     270, "2025-04-30"),
    ("高丽参",                 833, "2025-05-01"),
    ("化妆品",                 441, "2025-05-01"),
    ("韩国境内交通",            96, "2025-04-29"),
    ("韩国餐饮",               863, "2025-04-30"),
    ("漫游通信费",              55, "2025-04-29"),
]:
    call("create_record", bookId=BID, type="expense", amount=amt,
         happenedAt=date, eventId=EID, note=note)
    ok(f"-¥{amt:,.0f}  {note} [{date}]")

step("日常支出记录（2025年4月）")
for note, amt, date, from_acc in [
    ("校园卡充值",            789, "2025-04-01", "支付宝"),
    ("其他餐饮果蔬",          415, "2025-04-15", "支付宝"),
    ("湘菜·定燚（4.25）",      41, "2025-04-25", "微信"),
    ("定燚聚餐（4.25）",       38, "2025-04-25", "微信"),
    ("原神月卡",               30, "2025-04-01", "支付宝"),
    ("洗浴",                   40, "2025-04-20", "现金"),
    ("打车",                  156, "2025-04-12", "支付宝"),
    ("太阳岛",                 28, "2025-04-06", "现金"),
    ("京东购物",               54, "2025-04-10", "京东金融"),
    ("意外险（4月）",          142, "2025-04-01", "支付宝"),
    ("Apple Care+ 年费",       548, "2025-04-01", "招商银行储蓄卡"),
    ("按摩卡充值",             300, "2025-04-08", "微信"),
    ("拍照",                   50, "2025-04-15", "支付宝"),
    ("话费充值",               30, "2025-04-01", "支付宝"),
    ("其他杂项",               50, "2025-04-25", "支付宝"),
    ("韩币兑换（4万韩元）",   203, "2025-04-28", "支付宝"),
]:
    call("create_record", bookId=BID, type="expense", amount=amt,
         happenedAt=date, note=note,
         fromAccountId=accounts[from_acc])
    ok(f"-¥{amt:,.0f}  {note}")

step("创建净值快照（2025-05-04 资金清算日）")
snap = call("create_snapshot", bookId=BID)
ok(f"快照 ID: {snap['id']}")

step("收支统计（2025-04-01 ~ 2025-05-04）")
stats = call("get_book_stats", bookId=BID, **{"from":"2025-04-01","to":"2025-05-04"})
print(f"  总资产:   ¥{stats['totalAssets']:>13,.2f}")
print(f"  总负债:   ¥{stats['totalLiabilities']:>13,.2f}")
print(f"  净资产:   ¥{stats['netWorth']:>13,.2f}")
print(f"  期间收入: ¥{stats['income']:>13,.2f}")
print(f"  期间支出: ¥{stats['expense']:>13,.2f}")
print(f"  收支差额: ¥{stats['income']-stats['expense']:>13,.2f}")
print("\n✅ 全部数据导入完成！")
