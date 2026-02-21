import { invoke } from "@tauri-apps/api/core";

// ─── 类型定义 ─────────────────────────────────────────────────────────────────

export type RecordType = "income" | "expense" | "transfer";

/** 流水记录：原子记账单元，可独立展示 */
export interface FlowRecord {
  id: string;
  bookId: string;
  eventId: string | null;
  /** 可选名称，未命名时用分类名作展示 */
  name: string | null;
  type: RecordType;
  amount: number;
  categoryId: string | null;
  /** 支出/转账：资金转出账户（Entry.isAccount=true） */
  fromAccountId: string | null;
  /** 收入/转账：资金转入账户（Entry.isAccount=true） */
  toAccountId: string | null;
  tagIds: string[];
  note: string | null;
  happenedAt: string;
  createdAt: string;
}

/** 事件：轻量聚合单元，将语义相关的多条流水组合在一起 */
export interface Event {
  id: string;
  bookId: string;
  name: string;
  description: string | null;
  createdAt: string;
}

/** 事件 + 其下所有流水记录 */
export interface EventWithRecords extends Event {
  records: FlowRecord[];
}

// ─── 入参类型 ─────────────────────────────────────────────────────────────────

export interface CreateRecordParams {
  bookId: string;
  eventId?: string;
  name?: string;
  type: RecordType;
  amount: number;
  categoryId?: string;
  fromAccountId?: string;
  toAccountId?: string;
  tagIds?: string[];
  note?: string;
  happenedAt: string;
}

export interface UpdateRecordParams {
  eventId?: string | null;
  name?: string | null;
  type?: RecordType;
  amount?: number;
  categoryId?: string | null;
  fromAccountId?: string | null;
  toAccountId?: string | null;
  tagIds?: string[];
  note?: string | null;
  happenedAt?: string;
}

export interface CreateEventParams {
  bookId: string;
  name: string;
  description?: string;
}

export interface UpdateEventParams {
  name?: string;
  description?: string | null;
}

export interface RecordFilter {
  /** 筛选类型：income | expense | transfer */
  recordType?: RecordType;
  /** 按所属事件筛选 */
  eventId?: string;
  /** 按分类筛选 */
  categoryId?: string;
  /** 开始日期（ISO 8601） */
  from?: string;
  /** 结束日期（ISO 8601） */
  to?: string;
}

// ─── API ──────────────────────────────────────────────────────────────────────

/** 流水记录 API */
export const recordApi = {
  /** 列出账本下所有流水记录，支持筛选 */
  list: (bookId: string, filter?: RecordFilter) =>
    invoke<FlowRecord[]>("list_records", { bookId, filter }),

  /** 获取单条流水记录详情 */
  get: (id: string) =>
    invoke<FlowRecord>("get_record", { id }),

  /** 创建流水记录 */
  create: (params: CreateRecordParams) =>
    invoke<FlowRecord>("create_record", { params }),

  /** 更新流水记录（patch，只传需要修改的字段） */
  update: (id: string, params: UpdateRecordParams) =>
    invoke<FlowRecord>("update_record", { id, params }),

  /** 删除流水记录 */
  delete: (id: string) =>
    invoke<void>("delete_record", { id }),
};

/** 事件 API */
export const eventApi = {
  /** 列出账本下所有事件 */
  list: (bookId: string) =>
    invoke<Event[]>("list_events", { bookId }),

  /** 获取事件详情（含旗下全部流水记录） */
  get: (id: string) =>
    invoke<EventWithRecords>("get_event", { id }),

  /** 创建事件 */
  create: (params: CreateEventParams) =>
    invoke<Event>("create_event", { params }),

  /** 更新事件（名称/描述） */
  update: (id: string, params: UpdateEventParams) =>
    invoke<Event>("update_event", { id, params }),

  /**
   * 删除事件（不删除旗下流水记录，记录的 eventId 自动置 null）
   */
  delete: (id: string) =>
    invoke<void>("delete_event", { id }),
};

// ─── 混合 Feed ────────────────────────────────────────────────────────────────

/** 事件摘要：包含旗下流水聚合金额，用于混合 Feed 列表 */
export interface EventSummary {
  id: string;
  bookId: string;
  name: string;
  description: string | null;
  createdAt: string;
  /** 旗下流水收入总计 */
  totalIncome: number;
  /** 旗下流水支出总计 */
  totalExpense: number;
  /** 旗下流水条数 */
  recordCount: number;
  /** 旗下最新流水日期（无流水时为 null，排序时回退到 createdAt） */
  latestHappenedAt: string | null;
}

/** 混合 Feed 项：带 itemType 鉴别符的联合类型 */
export type FeedItem =
  | (EventSummary & { itemType: "event" })
  | (FlowRecord & { itemType: "record" });

export interface FeedSort {
  /** "date"（默认）或 "amount" */
  sortBy?: "date" | "amount";
  /** "desc"（默认）或 "asc" */
  sortOrder?: "asc" | "desc";
}

/** 混合 Feed API */
export const feedApi = {
  /**
   * 返回账本下所有事件（含聚合金额）+ 独立流水（无事件归属）的混合列表，
   * 在后端完成排序。sortBy: "date"（默认）| "amount"
   */
  list: (bookId: string, sort?: FeedSort) =>
    invoke<FeedItem[]>("list_feed", { bookId, sort }),
};
