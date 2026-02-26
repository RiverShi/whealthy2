import json, http.client
HOST, PORT = "127.0.0.1", 3030
BID = "ecaf33af-a900-4b17-84f2-b49a06242da7"
_id = 0

def call(method, **args):
    global _id; _id += 1
    payload = json.dumps({"jsonrpc":"2.0","method":"tools/call","params":{"name":method,"arguments":args},"id":_id},ensure_ascii=False).encode("utf-8")
    conn = http.client.HTTPConnection(HOST, PORT, timeout=15)
    conn.request("POST", "/mcp", body=payload, headers={"Content-Type":"application/json; charset=utf-8","Content-Length":str(len(payload))})
    data = json.loads(conn.getresponse().read().decode("utf-8"))
    conn.close()
    return json.loads(data["result"]["content"][0]["text"])

entries = call("list_entries", bookId=BID)
records = call("list_records", bookId=BID)

no_cat_entries = [e for e in entries if not e.get("categoryL1Id")]
no_cat_records = [r for r in records if not r.get("categoryId")]

print("条目总数:", len(entries), " 无分类:", len(no_cat_entries))
for e in no_cat_entries:
    print("  [" + e["id"][:8] + "]", e["kind"], e["name"])

print("\n记录总数:", len(records), " 无分类:", len(no_cat_records))
for r in no_cat_records:
    print("  [" + r["id"][:8] + "]", r["type"], r.get("note",""))
