import { invoke } from "@tauri-apps/api/core";

// ---- 类型定义 ----
export type EntryKind = "asset" | "liability";
export type ValuationType = "fixed" | "manual";

export interface Entry {
  id: string;
  bookId: string;
  name: string;
  kind: EntryKind;
  isAccount: boolean;
  valuationType: ValuationType;
  value: number;
  categoryL1Id: string | null;
  categoryL2Id: string | null;
  tagIds: string[];
  extra: Record<string, unknown> | null;
  openedAt: string;
  closedAt: string | null;
}

export interface EntryAdjustment {
  id: string;
  entryId: string;
  oldValue: number;
  newValue: number;
  reason: string | null;
  adjustedAt: string;
}

export interface CreateEntryParams {
  bookId: string;
  name: string;
  kind: EntryKind;
  isAccount: boolean;
  value: number;
  valuationType: ValuationType;
  categoryL1Id?: string;
  categoryL2Id?: string;
  tagIds?: string[];
  extra?: Record<string, unknown>;
}

export interface EntryFilter {
  kind?: EntryKind;
  isClosed?: boolean;
  categoryL1Id?: string;
  tagIds?: string[];
}

// ---- API ----
export const entryApi = {
  list: (bookId: string, filter?: EntryFilter) =>
    invoke<Entry[]>("list_entries", { bookId, filter }),
  get: (id: string) => invoke<Entry>("get_entry", { id }),
  create: (params: CreateEntryParams) => invoke<Entry>("create_entry", { params }),
  update: (id: string, params: Partial<CreateEntryParams>) =>
    invoke<Entry>("update_entry", { id, params }),
  remove: (id: string) => invoke<void>("delete_entry", { id }),

  adjustValue: (id: string, newValue: number, reason?: string) =>
    invoke<void>("adjust_entry_value", { id, newValue, reason }),
  listAdjustments: (entryId: string) =>
    invoke<EntryAdjustment[]>("list_entry_adjustments", { entryId }),
};
