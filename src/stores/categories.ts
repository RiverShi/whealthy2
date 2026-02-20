import { defineStore } from "pinia";
import { ref } from "vue";
import { categoryApi, tagApi, type Category, type Tag, type CategoryDomain, type TagDomain } from "@/api/categories";

export const useCategoryStore = defineStore("categories", () => {
  const categories = ref<Category[]>([]);
  const tags = ref<Tag[]>([]);
  const loading = ref(false);

  async function fetchCategories(domain?: CategoryDomain) {
    loading.value = true;
    try {
      if (domain) {
        const result = await categoryApi.list(domain);
        // 更新该领域的分类
        categories.value = categories.value.filter(c => c.domain !== domain).concat(result);
      } else {
        // 加载所有领域
        const domains: CategoryDomain[] = ["asset", "liability", "income", "expense"];
        const results = await Promise.all(domains.map((d) => categoryApi.list(d)));
        categories.value = results.flat();
      }
    } finally {
      loading.value = false;
    }
  }

  async function createCategory(domain: CategoryDomain, level: 1 | 2, name: string, icon?: string, parentId?: string) {
    const cat = await categoryApi.create(domain, level, name, icon, parentId);
    categories.value.push(cat);
    return cat;
  }

  async function updateCategory(id: string, name?: string, icon?: string, parentId?: string | null) {
    const updated = await categoryApi.update(id, { name, icon, parentId });
    const idx = categories.value.findIndex((c) => c.id === id);
    if (idx !== -1) categories.value[idx] = updated;
    return updated;
  }

  async function deleteCategory(id: string) {
    await categoryApi.remove(id);
    categories.value = categories.value.filter((c) => c.id !== id);
  }

  async function fetchTags(domain?: TagDomain) {
    loading.value = true;
    try {
      if (domain) {
        const result = await tagApi.list(domain);
        tags.value = tags.value.filter(t => t.domain !== domain).concat(result);
      } else {
        const domains: TagDomain[] = ["asset", "liability", "transaction"];
        const results = await Promise.all(domains.map((d) => tagApi.list(d)));
        tags.value = results.flat();
      }
    } finally {
      loading.value = false;
    }
  }

  async function createTag(domain: TagDomain, name: string, color?: string) {
    const tag = await tagApi.create(domain, name, color);
    tags.value.push(tag);
    return tag;
  }

  async function updateTag(id: string, name?: string, color?: string) {
    const updated = await tagApi.update(id, { name, color });
    const idx = tags.value.findIndex((t) => t.id === id);
    if (idx !== -1) tags.value[idx] = updated;
    return updated;
  }

  async function deleteTag(id: string) {
    await tagApi.remove(id);
    tags.value = tags.value.filter((t) => t.id !== id);
  }

  return {
    categories,
    tags,
    loading,
    fetchCategories,
    createCategory,
    updateCategory,
    deleteCategory,
    fetchTags,
    createTag,
    updateTag,
    deleteTag,
  };
});
