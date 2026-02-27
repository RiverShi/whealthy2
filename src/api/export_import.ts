import { invoke } from "@tauri-apps/api/core";
import type { Book } from "./books";

/**
 * 导出账本为 JSON 字符串（包含分类/标签/条目/调整记录/事件/流水）
 */
export async function exportBook(bookId: string): Promise<string> {
  return await invoke<string>("export_book", { bookId });
}

/**
 * 从 JSON 字符串导入账本，返回新建的账本信息
 * @param jsonData  导出的 JSON 字符串
 * @param newName   可选，指定新账本名称（不填则沿用原名）
 */
export async function importBook(jsonData: string, newName?: string): Promise<Book> {
  return await invoke<Book>("import_book", { jsonData, newName: newName ?? null });
}

/**
 * 触发文件下载（桌面端备用方案）
 */
export function downloadJson(content: string, filename: string) {
  const blob = new Blob([content], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

/**
 * 使用 Web Share API 分享/保存文件（iOS 推荐）
 * 若不支持 share，则退回到直接下载
 */
export async function shareOrDownloadJson(content: string, filename: string): Promise<void> {
  const blob = new Blob([content], { type: "application/json" });
  const file = new File([blob], filename, { type: "application/json" });

  if (navigator.canShare && navigator.canShare({ files: [file] })) {
    await navigator.share({ files: [file], title: filename });
  } else {
    downloadJson(content, filename);
  }
}
