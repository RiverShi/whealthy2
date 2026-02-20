import { invoke } from "@tauri-apps/api/core";

// ---- 类型定义 ----
export interface Book {
  id: string;
  name: string;
  createdAt: string;
  archivedAt: string | null;
}

// ---- API ----
export const bookApi = {
  list: () => invoke<Book[]>("list_books"),
  create: (name: string) => invoke<Book>("create_book", { name }),
  update: (id: string, name: string) => invoke<Book>("update_book", { id, name }),
  archive: (id: string) => invoke<void>("archive_book", { id }),
  remove: (id: string) => invoke<void>("delete_book", { id }),
};
