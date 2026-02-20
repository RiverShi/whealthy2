<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { useCategoryStore } from "@/stores/categories";
import { Plus, Edit2, Trash2, FolderTree } from "lucide-vue-next";
import type { Category, CategoryDomain } from "@/api/categories";

const categoryStore = useCategoryStore();

type Tab = "asset" | "liability" | "income" | "expense";
const activeTab = ref<Tab>("asset");

const tabs: { key: Tab; label: string; domain: CategoryDomain }[] = [
  { key: "asset", label: "资产", domain: "asset" },
  { key: "liability", label: "负债", domain: "liability" },
  { key: "income", label: "收入", domain: "income" },
  { key: "expense", label: "支出", domain: "expense" },
];

const showCreate = ref(false);
const editingCategory = ref<Category | undefined>();
const formName = ref("");
const formIcon = ref("");
const formLevel = ref<1 | 2>(1);
const formParentId = ref<string | undefined>();

onMounted(() => {
  categoryStore.fetchCategories();
});

const currentDomain = computed(() => tabs.find((t) => t.key === activeTab.value)!.domain);

const level1Categories = computed(() => {
  return categoryStore.categories.filter(
    (c) => c.domain === currentDomain.value && c.level === 1
  );
});

const level2Categories = computed(() => {
  return categoryStore.categories.filter(
    (c) => c.domain === currentDomain.value && c.level === 2
  );
});

function getCategoryTree() {
  return level1Categories.value.map((parent) => ({
    ...parent,
    children: level2Categories.value.filter((c) => c.parentId === parent.id),
  }));
}

function handleCreate(level: 1 | 2, parentId?: string) {
  editingCategory.value = undefined;
  formLevel.value = level;
  formParentId.value = parentId;
  formName.value = "";
  formIcon.value = "";
  showCreate.value = true;
}

function handleEdit(category: Category) {
  editingCategory.value = category;
  formName.value = category.name;
  formIcon.value = category.icon || "";
  formLevel.value = category.level;
  formParentId.value = category.parentId;
  showCreate.value = true;
}

async function handleSubmit() {
  if (!formName.value.trim()) return;

  try {
    if (editingCategory.value) {
      await categoryStore.updateCategory(
        editingCategory.value.id,
        formName.value.trim(),
        formIcon.value || undefined,
        editingCategory.value.level === 2 ? formParentId.value : undefined
      );
    } else {
      await categoryStore.createCategory(
        currentDomain.value,
        formLevel.value,
        formName.value.trim(),
        formIcon.value || undefined,
        formParentId.value
      );
    }
    showCreate.value = false;
    categoryStore.fetchCategories();
  } catch (error) {
    console.error("Failed to save category:", error);
  }
}

async function handleDelete(id: string) {
  if (!confirm("确定删除此分类？子分类也将被删除。")) return;
  try {
    await categoryStore.deleteCategory(id);
    categoryStore.fetchCategories();
  } catch (error) {
    console.error("Failed to delete category:", error);
  }
}
</script>

<template>
  <div class="p-8">
    <div class="mb-8">
      <h1 class="text-2xl font-bold mb-1">分类管理</h1>
      <p class="text-sm text-muted-foreground">管理资产、负债、收入、支出的分类</p>
    </div>

    <div class="flex gap-2 mb-6 border-b border-border">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        @click="activeTab = tab.key"
        :class="[
          'px-4 py-2 text-sm font-medium transition-colors relative',
          activeTab === tab.key
            ? 'text-primary'
            : 'text-muted-foreground hover:text-foreground',
        ]"
      >
        {{ tab.label }}
        <div
          v-if="activeTab === tab.key"
          class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary"
        />
      </button>
    </div>

    <div class="flex justify-end mb-4">
      <button
        @click="handleCreate(1)"
        class="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:opacity-90 transition-opacity"
      >
        <Plus class="w-4 h-4" />
        新建一级分类
      </button>
    </div>

    <div v-if="categoryStore.loading" class="text-center py-16 text-muted-foreground">
      加载中…
    </div>

    <div v-else-if="level1Categories.length" class="space-y-3">
      <div
        v-for="parent in getCategoryTree()"
        :key="parent.id"
        class="rounded-xl border border-border bg-card overflow-hidden"
      >
        <div class="group flex items-center justify-between px-4 py-3 bg-accent/30">
          <div class="flex items-center gap-3">
            <FolderTree class="w-4 h-4 text-muted-foreground" />
            <span v-if="parent.icon" class="text-lg">{{ parent.icon }}</span>
            <span class="font-medium">{{ parent.name }}</span>
          </div>
          <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              @click="handleCreate(2, parent.id)"
              class="p-1.5 rounded-lg hover:bg-accent transition-colors"
              title="添加子分类"
            >
              <Plus class="w-4 h-4" />
            </button>
            <button
              @click="handleEdit(parent)"
              class="p-1.5 rounded-lg hover:bg-accent transition-colors"
              title="编辑"
            >
              <Edit2 class="w-4 h-4" />
            </button>
            <button
              @click="handleDelete(parent.id)"
              class="p-1.5 rounded-lg hover:bg-destructive/10 text-destructive transition-colors"
              title="删除"
            >
              <Trash2 class="w-4 h-4" />
            </button>
          </div>
        </div>

        <div v-if="parent.children && parent.children.length" class="divide-y divide-border">
          <div
            v-for="child in parent.children"
            :key="child.id"
            class="group flex items-center justify-between px-4 py-2.5 pl-12 hover:bg-accent/20 transition-colors"
          >
            <div class="flex items-center gap-3">
              <span v-if="child.icon" class="text-base">{{ child.icon }}</span>
              <span class="text-sm">{{ child.name }}</span>
            </div>
            <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                @click="handleEdit(child)"
                class="p-1.5 rounded-lg hover:bg-accent transition-colors"
                title="编辑"
              >
                <Edit2 class="w-3.5 h-3.5" />
              </button>
              <button
                @click="handleDelete(child.id)"
                class="p-1.5 rounded-lg hover:bg-destructive/10 text-destructive transition-colors"
                title="删除"
              >
                <Trash2 class="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="text-center py-16">
      <p class="text-muted-foreground mb-4">暂无分类</p>
      <button
        @click="handleCreate(1)"
        class="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:opacity-90 transition-opacity"
      >
        创建第一个分类
      </button>
    </div>

    <div
      v-if="showCreate"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    >
      <div class="bg-card rounded-2xl p-6 w-full max-w-md shadow-xl border border-border">
        <h2 class="text-base font-semibold mb-4">
          {{ editingCategory ? "编辑分类" : "新建分类" }}
        </h2>

        <div class="space-y-4 mb-6">
          <div>
            <label class="block text-sm font-medium mb-2">分类名称</label>
            <input
              v-model="formName"
              placeholder="输入分类名称"
              class="w-full px-3 py-2 rounded-lg border border-input bg-background text-sm focus:outline-none focus:ring-2 focus:ring-ring"
              autofocus
            />
          </div>

          <div>
            <label class="block text-sm font-medium mb-2">图标（可选）</label>
            <input
              v-model="formIcon"
              placeholder="如：💰 🏠 🚗"
              class="w-full px-3 py-2 rounded-lg border border-input bg-background text-sm focus:outline-none focus:ring-2 focus:ring-ring"
            />
          </div>

          <div v-if="formLevel === 2">
            <label class="block text-sm font-medium mb-2">所属一级分类</label>
            <select
              v-model="formParentId"
              class="w-full px-3 py-2 rounded-lg border border-input bg-background text-sm focus:outline-none focus:ring-2 focus:ring-ring"
            >
              <option :value="undefined">请选择</option>
              <option
                v-for="cat in level1Categories"
                :key="cat.id"
                :value="cat.id"
              >
                {{ cat.icon }} {{ cat.name }}
              </option>
            </select>
          </div>

          <div v-if="!editingCategory && formLevel === 1" class="text-xs text-muted-foreground">
            将创建一级分类
          </div>
        </div>

        <div class="flex gap-2 justify-end">
          <button
            @click="showCreate = false"
            class="px-4 py-2 rounded-lg text-sm text-muted-foreground hover:bg-accent transition-colors"
          >
            取消
          </button>
          <button
            :disabled="!formName.trim()"
            @click="handleSubmit"
            class="px-4 py-2 rounded-lg bg-primary text-primary-foreground text-sm font-medium disabled:opacity-50 hover:opacity-90 transition-opacity"
          >
            {{ editingCategory ? "保存" : "创建" }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
