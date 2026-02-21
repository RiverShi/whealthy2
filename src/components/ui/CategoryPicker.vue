<script setup lang="ts">
/**
 * CategoryPicker — 紧凑两级分类选择器，支持即时新建
 *
 * v-model: 叶节点分类 id（L1 或 L2），空字符串表示未选
 * @select: { l1Id, l2Id }，l2Id 为空字符串表示选的是 L1
 */
import { ref, computed, watch } from "vue";
import { Plus, X } from "lucide-vue-next";
import { useCategoryStore } from "@/stores/categories";
import type { CategoryDomain } from "@/api/categories";

const props = defineProps<{
  modelValue: string;
  domain: CategoryDomain;
}>();

const emit = defineEmits<{
  "update:modelValue": [id: string];
  select: [payload: { l1Id: string; l2Id: string }];
}>();

const categoryStore = useCategoryStore();

// ── 选中状态 ──────────────────────────────────────────────────────────────────
const selectedL1 = ref("");
const selectedL2 = ref("");

const l1List = computed(() =>
  categoryStore.categories.filter(
    (c) => c.domain === props.domain && c.level === 1
  )
);

const l2List = computed(() =>
  selectedL1.value
    ? categoryStore.categories.filter(
        (c) =>
          c.domain === props.domain &&
          c.level === 2 &&
          c.parentId === selectedL1.value
      )
    : []
);

// 从 modelValue 初始化内部状态
watch(
  () => [props.modelValue, props.domain] as const,
  ([id]) => {
    if (!id) {
      selectedL1.value = "";
      selectedL2.value = "";
      return;
    }
    const cat = categoryStore.categories.find((c) => c.id === id);
    if (!cat) return;
    if (cat.level === 1) {
      selectedL1.value = id;
      selectedL2.value = "";
    } else {
      selectedL1.value = cat.parentId ?? "";
      selectedL2.value = id;
    }
  },
  { immediate: true }
);

// 当 domain 变化时自动加载分类（自给自足）
watch(
  () => props.domain,
  async (domain) => {
    const loaded = categoryStore.categories.some((c) => c.domain === domain);
    if (!loaded) await categoryStore.fetchCategories(domain);
    // 重置选中
    selectedL1.value = "";
    selectedL2.value = "";
  },
  { immediate: true }
);

// ── 点击处理 ──────────────────────────────────────────────────────────────────
function clickL1(id: string) {
  if (selectedL1.value === id && !selectedL2.value) {
    // 再次点击取消
    selectedL1.value = "";
    selectedL2.value = "";
    emit("update:modelValue", "");
    emit("select", { l1Id: "", l2Id: "" });
  } else {
    selectedL1.value = id;
    selectedL2.value = "";
    emit("update:modelValue", id);
    emit("select", { l1Id: id, l2Id: "" });
  }
}

function clickL2(id: string) {
  if (selectedL2.value === id) {
    // 再次点击取消 L2，保留 L1
    selectedL2.value = "";
    emit("update:modelValue", selectedL1.value);
    emit("select", { l1Id: selectedL1.value, l2Id: "" });
  } else {
    selectedL2.value = id;
    emit("update:modelValue", id);
    emit("select", { l1Id: selectedL1.value, l2Id: id });
  }
}

// ── 即时新建 ──────────────────────────────────────────────────────────────────
const showNewL1 = ref(false);
const showNewL2 = ref(false);
const newL1Name = ref("");
const newL2Name = ref("");
const creatingL1 = ref(false);
const creatingL2 = ref(false);

function openNewL1() {
  showNewL1.value = true;
  showNewL2.value = false;
  newL1Name.value = "";
}
function openNewL2() {
  showNewL2.value = true;
  showNewL1.value = false;
  newL2Name.value = "";
}

async function createL1() {
  const name = newL1Name.value.trim();
  if (!name || creatingL1.value) return;
  creatingL1.value = true;
  try {
    const cat = await categoryStore.createCategory(props.domain, 1, name);
    selectedL1.value = cat.id;
    selectedL2.value = "";
    emit("update:modelValue", cat.id);
    emit("select", { l1Id: cat.id, l2Id: "" });
    showNewL1.value = false;
    newL1Name.value = "";
  } finally {
    creatingL1.value = false;
  }
}

async function createL2() {
  const name = newL2Name.value.trim();
  if (!name || !selectedL1.value || creatingL2.value) return;
  creatingL2.value = true;
  try {
    const cat = await categoryStore.createCategory(
      props.domain,
      2,
      name,
      undefined,
      selectedL1.value
    );
    selectedL2.value = cat.id;
    emit("update:modelValue", cat.id);
    emit("select", { l1Id: selectedL1.value, l2Id: cat.id });
    showNewL2.value = false;
    newL2Name.value = "";
  } finally {
    creatingL2.value = false;
  }
}
</script>

<template>
  <div class="space-y-2">
    <!-- ── 一级分类 ─────────────────────────────────────────────────────────── -->
    <div class="flex flex-wrap gap-1.5 items-center min-h-[28px]">
      <button
        v-for="c in l1List"
        :key="c.id"
        type="button"
        @click="clickL1(c.id)"
        class="px-2.5 py-1 text-xs rounded-lg border transition-colors leading-none"
        :class="
          selectedL1 === c.id
            ? 'bg-primary/12 border-primary text-primary font-semibold'
            : 'border-border text-muted-foreground hover:border-primary/50 hover:text-foreground'
        "
      >
        {{ c.icon ? c.icon + "\u00a0" : "" }}{{ c.name }}
      </button>

      <!-- 内联新建 L1 -->
      <template v-if="!showNewL1">
        <button
          type="button"
          @click="openNewL1"
          class="h-6 px-2 text-xs rounded-lg border border-dashed border-border text-muted-foreground hover:border-primary hover:text-primary transition-colors flex items-center gap-0.5"
        >
          <Plus class="w-3 h-3" />新建
        </button>
      </template>
      <template v-else>
        <div class="flex items-center gap-1">
          <input
            v-model="newL1Name"
            placeholder="分类名"
            autofocus
            @keyup.enter="createL1"
            @keyup.esc="showNewL1 = false"
            class="h-7 w-20 px-2 text-xs rounded-lg border border-primary bg-background focus:outline-none"
          />
          <button
            type="button"
            @click="createL1"
            :disabled="!newL1Name.trim() || creatingL1"
            class="h-7 px-2 text-xs rounded-lg bg-primary text-primary-foreground disabled:opacity-40"
          >
            {{ creatingL1 ? "…" : "确定" }}
          </button>
          <button
            type="button"
            @click="showNewL1 = false"
            class="h-7 w-7 flex items-center justify-center rounded-lg bg-muted text-muted-foreground"
          >
            <X class="w-3 h-3" />
          </button>
        </div>
      </template>
    </div>

    <!-- ── 二级分类（仅当选中 L1 时显示） ────────────────────────────────────── -->
    <div
      v-if="selectedL1"
      class="flex flex-wrap gap-1.5 items-center pl-3 border-l-2 border-primary/25 min-h-[28px]"
    >
      <button
        v-for="c in l2List"
        :key="c.id"
        type="button"
        @click="clickL2(c.id)"
        class="px-2.5 py-1 text-xs rounded-lg border transition-colors leading-none"
        :class="
          selectedL2 === c.id
            ? 'bg-primary/12 border-primary text-primary font-semibold'
            : 'border-border text-muted-foreground/80 hover:border-primary/50 hover:text-foreground bg-muted/30'
        "
      >
        {{ c.icon ? c.icon + "\u00a0" : "" }}{{ c.name }}
      </button>
      <span
        v-if="l2List.length === 0 && !showNewL2"
        class="text-xs text-muted-foreground/50 italic"
      >无子分类</span>

      <!-- 内联新建 L2 -->
      <template v-if="!showNewL2">
        <button
          type="button"
          @click="openNewL2"
          class="h-6 px-2 text-xs rounded-lg border border-dashed border-border text-muted-foreground hover:border-primary hover:text-primary transition-colors flex items-center gap-0.5"
        >
          <Plus class="w-3 h-3" />子分类
        </button>
      </template>
      <template v-else>
        <div class="flex items-center gap-1">
          <input
            v-model="newL2Name"
            placeholder="子分类名"
            autofocus
            @keyup.enter="createL2"
            @keyup.esc="showNewL2 = false"
            class="h-7 w-20 px-2 text-xs rounded-lg border border-primary bg-background focus:outline-none"
          />
          <button
            type="button"
            @click="createL2"
            :disabled="!newL2Name.trim() || creatingL2"
            class="h-7 px-2 text-xs rounded-lg bg-primary text-primary-foreground disabled:opacity-40"
          >
            {{ creatingL2 ? "…" : "确定" }}
          </button>
          <button
            type="button"
            @click="showNewL2 = false"
            class="h-7 w-7 flex items-center justify-center rounded-lg bg-muted text-muted-foreground"
          >
            <X class="w-3 h-3" />
          </button>
        </div>
      </template>
    </div>
  </div>
</template>
