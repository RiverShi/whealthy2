import { invoke } from "@tauri-apps/api/core";

// ---- 类型定义 ----
export type SnapshotType = "full" | "incremental";
export type SnapshotSource = "auto" | "manual";
export type SnapshotFrequency = "daily" | "weekly" | "monthly";

export interface Snapshot {
  id: string;
  bookId: string;
  type: SnapshotType;
  baseSnapshotId: string | null;
  data: Record<string, unknown>;
  source: SnapshotSource;
  createdAt: string;
  netWorth?: number;
}

export interface SnapshotTask {
  id: string;
  bookId: string;
  frequency: SnapshotFrequency;
  lastRunAt: string | null;
  isActive: boolean;
}

export interface View {
  id: string;
  name: string;
  bookIds: string[];
}

export interface NetWorthResult {
  totalAssets: number;
  totalLiabilities: number;
  netWorth: number;
}

export interface IncomeExpenseResult {
  income: number;
  expense: number;
  byCategory: { categoryId: string; categoryName: string; amount: number }[];
}

// ---- API ----
export const snapshotApi = {
  list: (bookId: string, from?: string, to?: string) =>
    invoke<Snapshot[]>("list_snapshots", { bookId, from, to }),
  get: (id: string) => invoke<Snapshot>("get_snapshot", { id }),
  create: (bookId: string) => invoke<Snapshot>("create_snapshot", { bookId }),
  diff: (fromId: string, toId: string) =>
    invoke<Record<string, unknown>>("diff_snapshots", { fromId, toId }),

  listTasks: () => invoke<SnapshotTask[]>("list_snapshot_tasks"),
  createTask: (bookId: string, frequency: SnapshotFrequency) =>
    invoke<SnapshotTask>("create_snapshot_task", { bookId, frequency }),
  updateTask: (id: string, params: { frequency?: SnapshotFrequency; isActive?: boolean }) =>
    invoke<SnapshotTask>("update_snapshot_task", { id, ...params }),
  deleteTask: (id: string) => invoke<void>("delete_snapshot_task", { id }),
  checkAndRun: () => invoke<void>("check_and_run_snapshot_tasks"),
};

export const viewApi = {
  list: () => invoke<View[]>("list_views"),
  create: (name: string, bookIds: string[]) => invoke<View>("create_view", { name, bookIds }),
  update: (id: string, params: { name?: string; bookIds?: string[] }) =>
    invoke<View>("update_view", { id, ...params }),
  remove: (id: string) => invoke<void>("delete_view", { id }),

  getNetWorth: (viewId: string, at?: string) =>
    invoke<NetWorthResult>("get_view_net_worth", { viewId, at }),
  getBalanceSheet: (viewId: string, at?: string) =>
    invoke<Record<string, unknown>>("get_view_balance_sheet", { viewId, at }),
  getIncomeExpense: (viewId: string, from: string, to: string, groupBy?: string) =>
    invoke<IncomeExpenseResult>("get_view_income_expense", { viewId, from, to, groupBy }),
};
