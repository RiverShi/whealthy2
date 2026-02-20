<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { useCategoryStore } from "@/stores/categories";
import { Plus, Edit2, Trash2, Tag as TagIcon } from "lucide-vue-next";
import type { Tag, TagDomain } from "@/api/categories";

const categoryStore = useCategoryStore();

type Tab = "asset" | "liability" | "transaction";
const activeTab = ref<Tab>("asset");

const tabs: { key: Tab; label: string; domain: TagDomain }[] = [
  { key: "asset", label: "资产", domain: "asset" },
  { key: "liability", label: "负债", domain: "liability" },
  { key: "transaction", label: "交易", domain: "transaction" },
];

const showCreate = ref(false);
const editingTag = ref<Tag | undefined>();
const formName = ref("");
const formColor = ref("#6366f1");

onMounted(() => {
  categoryStore.fetchTags();
});

const currentDomain = computed(() => tabs.find((t) => t.key === activeTab.value)!.domain);

const filteredTags = computed(() => {
  return categoryStore.tags.filter((t) => t.domain === currentDomain.value);
});

function handleCreate() {
  editingTag.value = undefined;
  formName.value = "";
  formColor.value = "#6366f1";
  showCreate.value = true;
}

function handleEdit(tag: Tag) {
  editingTag.value = tag;
  formName.value = tag.name;
  formColor.value = tag.color || "#6366f1";
  showCreate.value = true;
}

async function handleSubmit() {
  if (!formName.value.trim()) return;

  try {
    if (editingTag.value) {
      await categoryStore.updateTag(
        editingTag.value.id,
        formName.value.trim(),
        formColor.value
      );
    } else {
      await categoryStore.createTag(
        currentDomain.value,
        formName.value.trim(),
        formColor.value
      );
    }
    showCreate.value = false;
    categoryStore.fetchTags();
  } catch (error) {
    console.error("Failed to save tag:", error);
  }
}

async function handleDelete(id: string) {
  if (!confirm("确定删除此标签？")) return;
  try {
    await categoryStore.deleteTag(id);
    categoryStore.fetchTags();
  } catch (error) {
    console.error("Failed to delete tag:", error);
  }
}

const colorPresets = [
  "#6366f1", "#8b5cf6", "#ec4899", "#f43f5e",
  "#f97316", "#f59e0b", "#eab308", "#84cc16",
  "#22c55e", "#10b981", "#14b8a6", "#06b6d4",
  "#0ea5e9", "#3b82f6", "#6366f1", "#8b5cf6",
];
</script>

<template>
  <div class="p-8">
    <div class="mb-8">
      <h1 class="text-2xl font-bold mb-1">标签管理</h1>
      <p class="text-sm text-muted-foreground">管理资产、负债、交易的标签</p>
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
        @click="handleCreate"
        class="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:opacity-90 transition-opacity"
      >
        <Plus class="w-4 h-4" />
        新建标签
      </button>
    </div>

    <div v-if="categoryStore.loading" class="text-center py-16 text-muted-foreground">
      加载中…
    </div>

    <div v-else-if="filteredTags.length" class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-3">
      <div
        v-for="tag in filteredTags"
        :key="tag.id"
        class="group relative rounded-xl border border-border bg-card p-4 hover:border-primary/30 transition-all"
      >
        <div class="flex items-center gap-3 mb-3">
          <div
            class="w-8 h-8 rounded-lg flex items-center justify-center"
            :style="{ backgroundColor: tag.color + '20' }"
          >
            <TagIcon class="w-4 h-4" :style="{ color: tag.color }" />
          </div>
          <span class="font-medium text-sm">{{ tag.name }}</span>
        </div>

        <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
          <button
            @click="handleEdit(tag)"
            class="flex-1 p-1.5 rounded-lg hover:bg-accent transition-colors text-xs"
          >
            <Edit2 class="w-3.5 h-3.5 mx-auto" />
          </button>
          <button
            @click="handleDelete(tag.id)"
            class="flex-1 p-1.5 rounded-lg hover:bg-destructive/10 text-destructive transition-colors text-xs"
          >
            <Trash2 class="w-3.5 h-3.5 mx-auto" />
          </button>
        </div>
      </div>
    </div>

    <div v-else class="text-center py-16">
      <p class="text-muted-foreground mb-4">暂无标签</p>
      <button
        @click="handleCreate"
        class="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:opacity-90 transition-opacity"
      >
        创建第一个标签
      </button>
    </div>

    <div
      v-if="showCreate"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    >
      <div class="bg-card rounded-2xl p-6 w-full max-w-md shadow-xl border border-border">
        <h2 class="text-base font-semibold mb-4">
          {{ editingTag ? "编辑标签" : "新建标签" }}
        </h2>

        <div class="space-y-4 mb-6">
          <div>
            <label class="block text-sm font-medium mb-2">标签名称</label>
            <input
              v-model="formName"
              placeholder="输入标签名称"
              class="w-full px-3 py-2 rounded-lg border border-input bg-background text-sm focus:outline-none focus:ring-2 focus:ring-ring"
              autofocus
            />
          </div>

          <div>
            <label class="block text-sm font-medium mb-2">颜色</label>
            <div class="grid grid-cols-8 gap-2">
              <button
                v-for="color in colorPresets"
                :key="color"
                @click="formColor = color"
                class="w-8 h-8 rounded-lg transition-transform hover:scale-110"
                :class="{ 'ring-2 ring-ring ring-offset-2': formColor === color }"
                :style="{ backgroundColor: color }"
              />
            </div>
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
            {{ editingTag ? "保存" : "创建" }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
