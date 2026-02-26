// 检查 MCP 服务状态
const axios = require('axios');

async function checkMCPService() {
  console.log("🔍 检查 MCP 服务状态...");

  try {
    // 尝试连接到 MCP 服务
    const response = await axios.post('http://localhost:3030/mcp', {
      jsonrpc: "2.0",
      method: "list_books",
      params: {},
      id: 1
    }, {
      headers: {
        'Content-Type': 'application/json'
      },
      timeout: 5000  // 5秒超时
    });

    console.log("✅ MCP 服务可用！");
    console.log("响应:", response.data);
  } catch (error) {
    console.log("❌ MCP 服务不可用或未运行");
    console.log("错误信息:", error.message);
    if (error.response) {
      console.log("响应状态码:", error.response.status);
      console.log("响应数据:", error.response.data);
    }
  }
}

// 如果直接运行此脚本，则执行检查
if (require.main === module) {
  checkMCPService();
}