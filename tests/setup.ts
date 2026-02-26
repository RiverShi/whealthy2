// tests/setup.ts
// 这里可以放置全局测试设置

// 模拟 Tauri 环境（用于单元测试）
if (typeof window !== 'undefined') {
  // @ts-ignore
  window.__TAURI_INTERNALS__ = {
    transformCallback: () => {},
    invoke: async (cmd: string, args: any) => {
      console.log(`模拟 Tauri 调用: ${cmd}`, args);
      // 根据不同的命令返回不同的模拟数据
      switch(cmd) {
        case 'list_books':
          return [];
        case 'create_book':
          return { id: 'mock-id', name: args.name, createdAt: new Date().toISOString() };
        default:
          return {};
      }
    }
  };

  // 模拟其他 Tauri API
  Object.assign(window, {
    __TAURI__: {
      dialog: {},
      fs: {},
      path: {},
      http: {},
      ipc: {},
      updater: {},
    }
  });
}