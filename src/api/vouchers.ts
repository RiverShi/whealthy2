import { invoke } from "@tauri-apps/api/core";

// ---- 类型定义 ----
export type VoucherType = "inflow" | "outflow" | "transfer";
export type PostingDirection = "debit" | "credit";

export interface Posting {
  id: string;
  voucherId: string;
  entryId: string | null;
  direction: PostingDirection;
  amount: number;
  primaryCategoryId: string;
  secondaryCategoryIds: string[];
  tagIds: string[];
  countInStats: boolean;
  note: string | null;
}

export interface Voucher {
  id: string;
  bookId: string;
  type: VoucherType;
  description: string | null;
  happenedAt: string;
  createdAt: string;
  postings?: Posting[];
}

export interface CreatePostingParams {
  entryId?: string;
  direction: PostingDirection;
  amount: number;
  primaryCategoryId: string;
  secondaryCategoryIds?: string[];
  tagIds?: string[];
  countInStats?: boolean;
  note?: string;
}

export interface VoucherFilter {
  type?: VoucherType;
  from?: string;
  to?: string;
  entryId?: string;
  categoryId?: string;
  tagIds?: string[];
}

// ---- API ----
export const voucherApi = {
  list: (bookId: string, filter?: VoucherFilter) =>
    invoke<Voucher[]>("list_vouchers", { bookId, filter }),
  get: (id: string) => invoke<Voucher>("get_voucher", { id }),
  create: (
    bookId: string,
    type: VoucherType,
    happenedAt: string,
    postings: CreatePostingParams[],
    description?: string
  ) => invoke<Voucher>("create_voucher", { bookId, type, happenedAt, description, postings }),
  update: (
    id: string,
    params: { description?: string; happenedAt?: string; postings?: CreatePostingParams[] }
  ) => invoke<Voucher>("update_voucher", { id, ...params }),
  remove: (id: string) => invoke<void>("delete_voucher", { id }),
};
