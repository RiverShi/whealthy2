<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { useRouter } from "vue-router";
import { useCategoryStore } from "@/stores/categories";
import { Plus, Edit2, Trash2, Tag as TagIcon } from "lucide-vue-next";
import type { Tag, TagDomain } from "@/api/categories";

const router = useRouter();

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
  <div class="min-h-full bg-background">
    <!-- 顶部导航头 -->
    <div class="sticky top-0 z-20 bg-card/95 backdrop-blur-xl border-b border-border">
      <div class="flex items-center gap-3 px-4 py-3">
        <button
          @click="router.back()"
          class="p-2 -ml-1 rounded-xl hover:bg-accent transition-colors cursor-pointer text-muted-foreground hover:text-foreground"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
        </button>
        <h1 class="text-xl font-bold flex-1">标签管理</h1>
        <button
          @click="handleCreate"
          class="flex items-center gap-1.5 px-3 py-2 bg-primary text-primary-foreground rounded-xl text-sm font-medium cursor-pointer"
        >
          <Plus class="w-3.5 h-3.5" />新建
        </button>
      </div>
    </div>

    <div class="px-4 py-3 pb-3">

    <!-- Tab -->
    <div class="flex gap-1 bg-muted/50 p-1 rounded-xl mb-4">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        @click="activeTab = tab.key"
        class="flex-1 py-1.5 rounded-lg text-xs font-medium transition-colors cursor-pointer"
        :class="activeTab === tab.key ? 'bg-card shadow text-foreground' : 'text-muted-foreground'"
      >{{ tab.label }}</button>
    </div>

    <div v-if="categoryStore.loading" class="text-center py-16 text-muted-foreground text-sm">加载中…</div>

    <!-- 标签列表 -->
    <div v-else-if="filteredTags.length" class="bg-card border border-border rounded-2xl overflow-hidden">
      <div
        v-for="(tag, i) in filteredTags"
        :key="tag.id"
        class="flex items-center gap-3 px-4 py-3.5"
        :class="i < filteredTags.length - 1 ? 'border-b border-border/60' : ''"
      >
        <div
          class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0"
          :style="{ backgroundColor: (tag.color || '#6366f1') + '20' }"
        >
          <TagIcon class="w-4 h-4" :style="{ color: tag.color || '#6366f1' }" />
        </div>
        <div class="flex-1 min-w-0">
          <span class="text-sm font-medium">{{ tag.name }}</span>
        </div>
        <div class="flex gap-0.5 shrink-0">
          <button
            @click="handleEdit(tag)"
            class="p-2 rounded-lg hover:bg-accent text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
          ><Edit2 class="w-4 h-4" /></button>
          <button
            @click="handleDelete(tag.id)"
            class="p-2 rounded-lg hover:bg-destructive/10 text-destructive transition-colors cursor-pointer"
          ><Trash2 class="w-4 h-4" /></button>
        </div>
      </div>
    </div>

    <div v-else class="text-center py-16">
      <p class="text-muted-foreground mb-4 text-sm">暂无标签</p>
      <button
        @click="handleCreate"
        class="px-4 py-2.5 bg-primary text-primary-foreground rounded-xl text-sm font-medium cursor-pointer"
      >创建第一个标签</button>
    </div>

    </div>

    <!-- 底部弹窗表单 -->
    <Teleport to="body">
      <div
        v-if="showCreate"
        class="fixed inset-0 z-50 flex items-end justify-center bg-black/40"
        @click.self="showCreate = false"
      >
        <div class="bg-card border border-border rounded-t-3xl w-full max-w-lg p-6 shadow-xl animate-in"
             style="padding-bottom: 24px">
          <div class="w-10 h-1 bg-border rounded-full mx-auto mb-5" />
          <h2 class="text-lg font-bold mb-5">{{ editingTag ? "编辑标签" : "新建标签" }}</h2>

          <div class="space-y-4 mb-6">
            <div>
              <label class="block text-sm font-medium mb-2">标签名称</label>
              <input
                v-model="formName"
                placeholder="输入标签名称"
                class="w-full px-4 py-3.5 rounded-2xl border border-input bg-background text-base focus:outline-none focus:ring-2 focus:ring-ring"
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
                  class="w-9 h-9 rounded-xl transition-transform cursor-pointer"
                  :class="{ 'ring-2 ring-ring ring-offset-2': formColor === color }"
                  :style="{ backgroundColor: color }"
                />
              </div>
            </div>
          </div>

          <div class="flex gap-3">
            <button
              @click="showCreate = false"
              class="flex-1 py-3.5 rounded-2xl border border-border text-sm font-medium hover:bg-accent transition-colors cursor-pointer"
            >取消</button>
            <button
              :disabled="!formName.trim()"
              @click="handleSubmit"
              class="flex-1 py-3.5 rounded-2xl bg-primary text-primary-foreground text-sm font-medium disabled:opacity-50 cursor-pointer"
            >{{ editingTag ? "保存" : "创建" }}</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
