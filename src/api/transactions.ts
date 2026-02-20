import { invoke } from "@tauri-apps/api/core";

// ---- 类型定义 ----
export type TransactionItemType = "income" | "expense" | "transfer";

export interface TransactionItem {
  id: string;
  transactionId: string;
  type: TransactionItemType;
  amount: number;
  categoryId: string | null;
  fromAccountId: string | null;
  toAccountId: string | null;
  tagIds: string[];
  note: string | null;
}

export interface Transaction {
  id: string;
  bookId: string;
  description: string | null;
  happenedAt: string;
  createdAt: string;
  items?: TransactionItem[];
}

export interface CreateTransactionItemParams {
  type: TransactionItemType;
  amount: number;
  categoryId?: string;
  fromAccountId?: string;
  toAccountId?: string;
  tagIds?: string[];
  note?: string;
}

export interface CreateTransactionParams {
  bookId: string;
  description?: string;
  happenedAt: string;
  items: CreateTransactionItemParams[];
}

// ---- API ----
export const transactionApi = {
  list: (bookId: string) =>
    invoke<Transaction[]>("list_transactions", { bookId }),
  get: (id: string) => invoke<Transaction>("get_transaction", { id }),
  create: (params: CreateTransactionParams) =>
    invoke<Transaction>("create_transaction", { params }),
  delete: (id: string) => invoke<void>("delete_transaction", { id }),
};
