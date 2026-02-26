import json, http.client
HOST, PORT = "127.0.0.1", 3030
BID = "ecaf33af-a900-4b17-84f2-b49a06242da7"
_id = 0

def call(method, **args):
    global _id
    _id += 1
    payload = json.dumps({"jsonrpc":"2.0","method":"tools/call","params":{"name":method,"arguments":args},"id":_id},ensure_ascii=False).encode("utf-8")
    conn = http.client.HTTPConnection(HOST, PORT, timeout=15)
    conn.request("POST", "/mcp", body=payload, headers={"Content-Type":"application/json; charset=utf-8","Content-Length":str(len(payload))})
    resp = conn.getresponse()
    data = json.loads(resp.read().decode("utf-8"))
    conn.close()
    return json.loads(data["result"]["content"][0]["text"])

entries = call("list_entries", bookId=BID)
asset_cats = call("list_categories", domain="asset")
debt_id = next(c["id"] for c in asset_cats if c["name"] == "债权")
print("债权分类 ID:", debt_id)

for e in entries:
    if any(k in e["name"] for k in ["MiPad", "iPad", "王海了"]):
        print(f"找到: {e['name']} [{e['id'][:8]}]")
        r = call("update_entry", id=e["id"], bookId=BID,
                 name="王海 · iPad 6s Pro 分期（111.1×23）",
                 kind="asset", isAccount=False,
                 value=e["value"], valuationType=e["valuationType"],
                 categoryL1Id=debt_id)
        print(f"✓ 更新成功: {r['name']}")
