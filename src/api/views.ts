import { invoke } from "@tauri-apps/api/core";

// ---- 统计类型 ----
export interface CategoryStat {
  categoryId: string | null;
  categoryName: string | null;
  amount: number;
}

export interface BookStats {
  totalAssets: number;
  totalLiabilities: number;
  netWorth: number;
  income: number;
  expense: number;
  /** 不计收支类（transfer/inflow/outflow）期间合计 */
  other: number;
  incomeByCategory: CategoryStat[];
  expenseByCategory: CategoryStat[];
}

// ---- 快照类型 ----
export type SnapshotSource = "auto" | "manual";
export type SnapshotFrequency = "daily" | "weekly" | "monthly";

export interface SnapshotEntryData {
  id: string;
  name: string;
  kind: "asset" | "liability";
  value: number;
  categoryL1Id: string | null;
  categoryName: string | null;
}

export interface SnapshotData {
  entries: SnapshotEntryData[];
  totalAssets: number;
  totalLiabilities: number;
  netWorth: number;
}

export interface Snapshot {
  id: string;
  bookId: string;
  source: SnapshotSource;
  netWorth: number;
  totalAssets: number;
  totalLiabilities: number;
  data: SnapshotData;
  createdAt: string;
}

export interface SnapshotTask {
  id: string;
  bookId: string;
  frequency: SnapshotFrequency;
  lastRunAt: string | null;
  isActive: boolean;
}

export interface SnapshotDiffEntry {
  entryId: string;
  entryName: string;
  kind: "asset" | "liability";
  oldValue: number | null;
  newValue: number | null;
  change: number;
}

// ---- API ----
export const statsApi = {
  getBookStats: (bookId: string, from: string, to: string) =>
    invoke<BookStats>("get_book_stats", { bookId, from, to }),
};

export const snapshotApi = {
  list: (bookId: string, from?: string, to?: string) =>
    invoke<Snapshot[]>("list_snapshots", { bookId, from, to }),
  get: (id: string) => invoke<Snapshot>("get_snapshot", { id }),
  create: (bookId: string) => invoke<Snapshot>("create_snapshot", { bookId }),
  diff: (fromId: string, toId: string) =>
    invoke<SnapshotDiffEntry[]>("diff_snapshots", { fromId, toId }),

  listTasks: (bookId?: string) =>
    invoke<SnapshotTask[]>("list_snapshot_tasks", { bookId }),
  getTaskForBook: (bookId: string) =>
    invoke<SnapshotTask | null>("get_snapshot_task_for_book", { bookId }),
  createTask: (bookId: string, frequency: SnapshotFrequency) =>
    invoke<SnapshotTask>("create_snapshot_task", { bookId, frequency }),
  updateTask: (id: string, params: { frequency?: SnapshotFrequency; isActive?: boolean }) =>
    invoke<SnapshotTask>("update_snapshot_task", { id, ...params }),
  deleteTask: (id: string) => invoke<void>("delete_snapshot_task", { id }),
  checkAndRun: () => invoke<void>("check_and_run_snapshot_tasks"),
};

