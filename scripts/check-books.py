import json, http.client
HOST, PORT = "127.0.0.1", 3030
_id = 0

def call(method, **args):
    global _id; _id += 1
    payload = json.dumps({"jsonrpc":"2.0","method":"tools/call","params":{"name":method,"arguments":args},"id":_id},ensure_ascii=False).encode("utf-8")
    conn = http.client.HTTPConnection(HOST, PORT, timeout=15)
    conn.request("POST", "/mcp", body=payload, headers={"Content-Type":"application/json; charset=utf-8","Content-Length":str(len(payload))})
    data = json.loads(conn.getresponse().read().decode("utf-8"))
    conn.close()
    return json.loads(data["result"]["content"][0]["text"])

books = call("list_books")
print("账本列表:")
for b in books:
    bid = b["id"]
    entries = call("list_entries", bookId=bid)
    records = call("list_records", bookId=bid)
    print("  [" + bid[:8] + "] " + b["name"] + " | 条目:" + str(len(entries)) + " 记录:" + str(len(records)))
