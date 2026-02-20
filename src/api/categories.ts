import { invoke } from "@tauri-apps/api/core";

// ---- 类型定义 ----
export type CategoryDomain = "asset" | "liability" | "income" | "expense";
export type TagDomain = "asset" | "liability" | "transaction";

export interface Category {
  id: string;
  domain: CategoryDomain;
  level: 1 | 2;
  parentId: string | null;
  name: string;
  icon: string | null;
}

export interface Tag {
  id: string;
  domain: TagDomain;
  name: string;
  color: string | null;
}

// ---- API ----
export const categoryApi = {
  list: (domain: CategoryDomain, level?: 1 | 2) =>
    invoke<Category[]>("list_categories", { domain, level }),
  create: (domain: CategoryDomain, level: 1 | 2, name: string, icon?: string, parentId?: string) =>
    invoke<Category>("create_category", { domain, level, name, parentId, icon }),
  update: (id: string, params: { name?: string; icon?: string; parentId?: string | null }) =>
    invoke<Category>("update_category", { id, name: params.name, icon: params.icon, parentId: params.parentId }),
  remove: (id: string) => invoke<void>("delete_category", { id }),
};

export const tagApi = {
  list: (domain?: TagDomain) => invoke<Tag[]>("list_tags", { domain }),
  create: (domain: TagDomain, name: string, color?: string) =>
    invoke<Tag>("create_tag", { domain, name, color }),
  update: (id: string, params: { name?: string; color?: string }) =>
    invoke<Tag>("update_tag", { id, ...params }),
  remove: (id: string) => invoke<void>("delete_tag", { id }),
};
