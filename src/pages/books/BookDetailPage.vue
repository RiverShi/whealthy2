<script setup lang="ts">
import { onMounted, ref, computed, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useEntryStore } from "@/stores/entries";
import { useBookStore } from "@/stores/books";
import { useCategoryStore } from "@/stores/categories";
import { entryApi } from "@/api/entries";
import { 
  TrendingUp, 
  TrendingDown, 
  Plus, 
  Edit2, 
  DollarSign,
  Wallet,
  Activity,
  CreditCard,
  ChevronRight,
  CheckCircle2,
  X,
} from "lucide-vue-next";
import EntryForm from "@/components/EntryForm.vue";
import AdjustValueSheet from "@/components/AdjustValueSheet.vue";
import EntryDetailSheet from "@/components/EntryDetailSheet.vue";
import type { Entry, EntryKind } from "@/api/entries";

const route = useRoute();
const router = useRouter();
const entryStore = useEntryStore();
const bookStore = useBookStore();
const categoryStore = useCategoryStore();

const bookId = computed(() => bookStore.activeBookId ?? "");
const book = computed(() => bookStore.activeBook);

const showCreateForm = ref(false);
const createKind = ref<EntryKind>("asset");
const editingEntry = ref<Entry | undefined>();
const adjustingEntry = ref<Entry | undefined>();
const viewingEntry = ref<Entry | undefined>();
const confirmDeleteEntry = ref<Entry | undefined>();
const activeTab = ref<'assets' | 'liabilities'>('assets');

// ── 快速编辑模式 ─────────────────────────────────────────────────────────
const quickEditMode = ref(false);
const quickEditValues = ref<Record<string, string>>({});
const quickEditSaving = ref(false);

function enterQuickEdit() {
  // 将所有条目当前值填入编辑 map
  const vals: Record<string, string> = {};
  for (const e of [...entryStore.assets, ...entryStore.liabilities]) {
    vals[e.id] = String(e.value);
  }
  quickEditValues.value = vals;
  quickEditMode.value = true;
}

function exitQuickEdit() {
  quickEditMode.value = false;
  quickEditValues.value = {};
}

async function confirmQuickEdit() {
  quickEditSaving.value = true;
  try {
    const all = [...entryStore.assets, ...entryStore.liabilities];
    const changed = all.filter(e => {
      const v = parseFloat(quickEditValues.value[e.id] ?? '');
      return !isNaN(v) && v !== e.value;
    });
    await Promise.all(
      changed.map(e => entryApi.adjustValue(e.id, parseFloat(quickEditValues.value[e.id]), '快速更新'))
    );
    await entryStore.fetchEntries(bookId.value);
    quickEditMode.value = false;
    quickEditValues.value = {};
  } finally {
    quickEditSaving.value = false;
  }
}

// FAB 传入 ?quickedit=1 时进入快速编辑
watch(() => route.query.quickedit, (val) => {
  if (val === '1') {
    enterQuickEdit();
    router.replace({ path: '/assets' });
  }
}, { immediate: true });

// ── 滑动切换 ──────────────────────────────────────────────────────────────
let touchStartX = 0;
function onTouchStart(e: TouchEvent) {
  touchStartX = e.touches[0].clientX;
}
function onTouchEnd(e: TouchEvent) {
  const dx = e.changedTouches[0].clientX - touchStartX;
  if (Math.abs(dx) > 50) {
    activeTab.value = dx < 0 ? 'liabilities' : 'assets';
  }
}

onMounted(() => {
  if (bookId.value) {
    entryStore.fetchEntries(bookId.value);
    categoryStore.fetchCategories("asset");
    categoryStore.fetchCategories("liability");
  }
});

watch(bookId, (newId) => {
  if (newId) {
    entryStore.fetchEntries(newId);
    categoryStore.fetchCategories("asset");
    categoryStore.fetchCategories("liability");
  }
});

// 分类 ID → 名称 map
const entryCategoryMap = computed(() => {
  const m: Record<string, string> = {};
  categoryStore.categories.forEach(c => { m[c.id] = c.name; });
  return m;
});

function fmt(v: number) {
  return v.toLocaleString("zh-CN", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

function handleCreateClick(kind: EntryKind) {
  createKind.value = kind;
  editingEntry.value = undefined;
  showCreateForm.value = true;
}

function handleEditClick(entry: Entry) {
  editingEntry.value = entry;
  showCreateForm.value = true;
}

function handleAdjustClick(entry: Entry) {
  adjustingEntry.value = entry;
}

function handleViewClick(entry: Entry) {
  if (quickEditMode.value) return;
  viewingEntry.value = entry;
}

async function handleDeleteFromDetail(entry: Entry) {
  confirmDeleteEntry.value = entry;
}

async function confirmDelete() {
  if (!confirmDeleteEntry.value) return;
  await entryStore.deleteEntry(confirmDeleteEntry.value.id);
  confirmDeleteEntry.value = undefined;
}

function handleFormSuccess() {
  entryStore.fetchEntries(bookId.value);
}

const filteredAssets = computed(() => entryStore.assets);
const filteredLiabilities = computed(() => entryStore.liabilities);

// ── 按分类分组 ──────────────────────────────────────────────────────────────
const collapsedGroups = ref<Set<string>>(new Set());

function toggleGroup(id: string) {
  if (quickEditMode.value) return; // 快速编辑中禁止折叠
  const s = new Set(collapsedGroups.value);
  if (s.has(id)) s.delete(id);
  else s.add(id);
  collapsedGroups.value = s;
}

function groupEntries(entries: typeof entryStore.assets, prefix: string) {
  const groups = new Map<string, { id: string; name: string; entries: typeof entryStore.assets }>();
  for (const e of entries) {
    const catId = e.categoryL1Id ?? "__uncategorized__";
    const catName = entryCategoryMap.value[catId] ?? "未分类";
    const key = `${prefix}__${catId}`;
    if (!groups.has(key)) {
      groups.set(key, { id: key, name: catName, entries: [] });
    }
    groups.get(key)!.entries.push(e);
  }
  return Array.from(groups.values());
}

const groupedAssets = computed(() => groupEntries(filteredAssets.value, "asset"));
const groupedLiabilities = computed(() => groupEntries(filteredLiabilities.value, "liability"));
</script>

<template>
  <div class="min-h-full bg-background">

    <!-- ══ 顶部固定头部 ════════════════════════════════════════════════════ -->
    <div class="sticky top-0 z-20 bg-card/95 backdrop-blur-xl border-b border-border">
      <div class="px-4 py-3">
        <div class="flex items-center justify-between gap-2">
          <div>
            <p class="text-[11px] text-muted-foreground leading-none mb-0.5">{{ book?.name }}</p>
            <h1 class="text-xl font-bold leading-tight">资产</h1>
          </div>
          <div class="flex gap-2">
            <button
              @click="handleCreateClick('liability')"
              class="flex items-center gap-1.5 px-3 py-2 border border-destructive/20 bg-destructive/5 text-destructive rounded-xl text-sm font-medium cursor-pointer"
            ><Plus class="w-3.5 h-3.5" />负债</button>
            <button
              @click="handleCreateClick('asset')"
              class="flex items-center gap-1.5 px-3 py-2 bg-primary text-primary-foreground rounded-xl text-sm font-medium cursor-pointer"
            ><Plus class="w-3.5 h-3.5" />资产</button>
          </div>
        </div>
      </div>
    </div>

    <!-- ══ 内容区 ════════════════════════════════════════════════════════ -->
    <div class="px-4 py-3 space-y-4" :class="quickEditMode ? 'pb-32' : 'pb-6'">

      <!-- ── 净资产汇总卡片 ──────────────────────────────────────────── -->
      <div class="rounded-2xl bg-primary px-5 py-4 text-primary-foreground">
        <p class="text-xs font-medium opacity-70 mb-1">净资产</p>
        <p class="text-3xl font-bold tabular-nums tracking-tight mb-3">¥{{ fmt(entryStore.netWorth) }}</p>
        <div class="flex items-center gap-4">
          <div class="flex items-center gap-1.5">
            <TrendingUp class="w-3.5 h-3.5 opacity-70" />
            <span class="text-xs opacity-70">总资产</span>
            <span class="text-sm font-semibold tabular-nums">¥{{ fmt(entryStore.totalAssets) }}</span>
          </div>
          <div class="w-px bg-primary-foreground/20 self-stretch" />
          <div class="flex items-center gap-1.5">
            <TrendingDown class="w-3.5 h-3.5 opacity-70" />
            <span class="text-xs opacity-70">负债</span>
            <span class="text-sm font-semibold tabular-nums">¥{{ fmt(entryStore.totalLiabilities) }}</span>
          </div>
        </div>
      </div>

      <!-- ── Tab 切换（资产 / 负债） ──────────────────────────────────── -->
      <div class="flex gap-1 bg-muted/50 p-1 rounded-xl">
        <button
          @click="activeTab = 'assets'"
          class="flex-1 py-1.5 rounded-lg text-xs font-medium transition-colors cursor-pointer"
          :class="activeTab === 'assets' ? 'bg-card shadow text-foreground' : 'text-muted-foreground'"
        >资产</button>
        <button
          @click="activeTab = 'liabilities'"
          class="flex-1 py-1.5 rounded-lg text-xs font-medium transition-colors cursor-pointer"
          :class="activeTab === 'liabilities' ? 'bg-card shadow text-foreground' : 'text-muted-foreground'"
        >负债</button>
      </div>

      <!-- ── 加载中 ──────────────────────────────────────────────────── -->
      <div v-if="entryStore.loading" class="flex flex-col items-center justify-center py-16">
        <div class="w-12 h-12 rounded-xl bg-primary/10 flex items-center justify-center mb-4 animate-pulse">
          <Activity class="w-6 h-6 text-primary" />
        </div>
        <p class="text-muted-foreground text-sm">加载中…</p>
      </div>

      <div v-else class="space-y-4" @touchstart="onTouchStart" @touchend="onTouchEnd">
        <!-- ── 资产列表（按分类折叠分组） ─────────────────────────────── -->
        <section v-if="activeTab === 'assets' && groupedAssets.length > 0">
          <div class="flex items-center gap-2 mb-2 px-1">
            <Wallet class="w-4 h-4 text-emerald-500" />
            <h2 class="text-sm font-semibold text-muted-foreground">资产</h2>
            <span class="text-xs text-muted-foreground">({{ filteredAssets.length }})</span>
          </div>
          <div class="space-y-2">
            <div v-for="group in groupedAssets" :key="group.id" class="bg-card border border-border rounded-2xl overflow-hidden">
              <!-- 分组标题可点击折叠 -->
              <button
                class="w-full flex items-center gap-2 px-4 py-3 text-left hover:bg-accent/40 transition-colors cursor-pointer"
                @click="toggleGroup(group.id)"
              >
                <ChevronRight
                  class="w-4 h-4 text-muted-foreground transition-transform duration-200 shrink-0"
                  :class="!collapsedGroups.has(group.id) ? 'rotate-90' : ''"
                />
                <span class="text-sm font-medium flex-1">{{ group.name }}</span>
                <span class="text-xs text-muted-foreground">{{ group.entries.length }} 项</span>
              </button>
              <div v-if="quickEditMode || !collapsedGroups.has(group.id)" class="border-t border-border/60">
                <div
                  v-for="(entry, i) in group.entries"
                  :key="entry.id"
                  class="flex items-center gap-3 px-4 py-3.5 transition-colors"
                  :class="[
                    i < group.entries.length - 1 ? 'border-b border-border/60' : '',
                    !quickEditMode ? 'cursor-pointer active:bg-accent/40' : ''
                  ]"
                  @click="handleViewClick(entry)"
                >
                  <div class="w-10 h-10 rounded-xl bg-emerald-500/10 flex items-center justify-center shrink-0">
                    <Wallet class="w-5 h-5 text-emerald-500" />
                  </div>
                  <div class="flex-1 min-w-0">
                    <p class="text-sm font-semibold truncate">{{ entry.name }}</p>
                    <p class="text-xs text-muted-foreground">{{ entry.isAccount ? '账户' : '资产' }}</p>
                  </div>
                  <!-- 快速编辑模式：内联 input -->
                  <div v-if="quickEditMode" class="flex items-center gap-1 shrink-0">
                    <span class="text-sm text-muted-foreground">¥</span>
                    <input
                      type="number"
                      inputmode="decimal"
                      step="0.01"
                      v-model="quickEditValues[entry.id]"
                      class="w-28 text-right text-sm font-bold tabular-nums bg-muted/60 border border-primary/30 rounded-lg px-2 py-1.5 focus:outline-none focus:border-primary text-emerald-600"
                      @click.stop
                    />
                  </div>
                  <div v-else class="text-right mr-1">
                    <p class="text-sm font-bold text-emerald-500 tabular-nums">¥{{ fmt(entry.value) }}</p>
                  </div>
                  <ChevronRight v-if="!quickEditMode" class="w-4 h-4 text-muted-foreground/50 shrink-0" />
                </div>
              </div>
            </div>
          </div>
        </section>

        <!-- ── 负债列表（按分类折叠分组） ─────────────────────────────── -->
        <section v-if="activeTab === 'liabilities' && groupedLiabilities.length > 0">
          <div class="flex items-center gap-2 mb-2 px-1">
            <CreditCard class="w-4 h-4 text-rose-500" />
            <h2 class="text-sm font-semibold text-muted-foreground">负债</h2>
            <span class="text-xs text-muted-foreground">({{ filteredLiabilities.length }})</span>
          </div>
          <div class="space-y-2">
            <div v-for="group in groupedLiabilities" :key="group.id" class="bg-card border border-border rounded-2xl overflow-hidden">
              <button
                class="w-full flex items-center gap-2 px-4 py-3 text-left hover:bg-accent/40 transition-colors cursor-pointer"
                @click="toggleGroup(group.id)"
              >
                <ChevronRight
                  class="w-4 h-4 text-muted-foreground transition-transform duration-200 shrink-0"
                  :class="!collapsedGroups.has(group.id) ? 'rotate-90' : ''"
                />
                <span class="text-sm font-medium flex-1">{{ group.name }}</span>
                <span class="text-xs text-muted-foreground">{{ group.entries.length }} 项</span>
              </button>
              <div v-if="quickEditMode || !collapsedGroups.has(group.id)" class="border-t border-border/60">
                <div
                  v-for="(entry, i) in group.entries"
                  :key="entry.id"
                  class="flex items-center gap-3 px-4 py-3.5 transition-colors"
                  :class="[
                    i < group.entries.length - 1 ? 'border-b border-border/60' : '',
                    !quickEditMode ? 'cursor-pointer active:bg-accent/40' : ''
                  ]"
                  @click="handleViewClick(entry)"
                >
                  <div class="w-10 h-10 rounded-xl bg-rose-500/10 flex items-center justify-center shrink-0">
                    <CreditCard class="w-5 h-5 text-rose-500" />
                  </div>
                  <div class="flex-1 min-w-0">
                    <p class="text-sm font-semibold truncate">{{ entry.name }}</p>
                    <p class="text-xs text-muted-foreground">{{ entry.valuationType === 'fixed' ? '固定值' : '手动估值' }}</p>
                  </div>
                  <!-- 快速编辑模式：内联 input -->
                  <div v-if="quickEditMode" class="flex items-center gap-1 shrink-0">
                    <span class="text-sm text-muted-foreground">¥</span>
                    <input
                      type="number"
                      inputmode="decimal"
                      step="0.01"
                      v-model="quickEditValues[entry.id]"
                      class="w-28 text-right text-sm font-bold tabular-nums bg-muted/60 border border-destructive/30 rounded-lg px-2 py-1.5 focus:outline-none focus:border-destructive text-rose-600"
                      @click.stop
                    />
                  </div>
                  <div v-else class="text-right mr-1">
                    <p class="text-sm font-bold text-rose-500 tabular-nums">¥{{ fmt(entry.value) }}</p>
                  </div>
                  <ChevronRight v-if="!quickEditMode" class="w-4 h-4 text-muted-foreground/50 shrink-0" />
                </div>
              </div>
            </div>
          </div>
        </section>

        <!-- 空状态 -->
        <div
          v-if="(activeTab === 'assets' ? !groupedAssets.length : !groupedLiabilities.length) && !entryStore.loading"
          class="text-center py-16"
        >
          <div class="w-14 h-14 rounded-2xl bg-muted flex items-center justify-center mb-4 mx-auto">
            <Wallet class="w-7 h-7 text-muted-foreground" />
          </div>
          <p class="text-base font-medium mb-1">还没有{{ activeTab === 'assets' ? '资产' : '负债' }}条目</p>
          <p class="text-sm text-muted-foreground">点击上方「资产」或「负债」添加</p>
        </div>
      </div>
    </div>

    <!-- ── 快速编辑底部确认栏 ────────────────────────────────────────── -->
    <Teleport to="body">
      <Transition
        enter-active-class="transition-transform duration-200"
        enter-from-class="translate-y-full"
        enter-to-class="translate-y-0"
        leave-active-class="transition-transform duration-200"
        leave-from-class="translate-y-0"
        leave-to-class="translate-y-full"
      >
        <div
          v-if="quickEditMode"
          class="fixed left-0 right-0 z-[60] bg-card border-t-2 border-primary shadow-2xl px-4 pt-4"
          style="bottom: calc(49px + env(safe-area-inset-bottom)); padding-bottom: 12px;"
        >
          <div class="flex gap-3 items-center">
            <button
              @click="exitQuickEdit"
              class="px-5 py-3 rounded-2xl border border-border text-sm font-medium cursor-pointer"
            >取消</button>
            <button
              @click="confirmQuickEdit"
              :disabled="quickEditSaving"
              class="flex-1 flex items-center justify-center gap-2 py-3 rounded-2xl bg-primary text-primary-foreground text-sm font-semibold cursor-pointer disabled:opacity-60"
            >
              <CheckCircle2 class="w-4 h-4" />
              {{ quickEditSaving ? '保存中…' : '确认全部' }}
            </button>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 表单 -->
    <EntryForm
      :open="showCreateForm"
      @update:open="showCreateForm = $event"
      :book-id="bookId"
      :entry="editingEntry"
      :default-kind="createKind"
      @success="handleFormSuccess"
    />

    <!-- 价值调整面板 -->
    <AdjustValueSheet
      :open="!!adjustingEntry"
      @update:open="(val) => { if (!val) adjustingEntry = undefined }"
      :entry="adjustingEntry"
      @success="handleFormSuccess"
    />

    <!-- 条目详情面板 -->
    <EntryDetailSheet
      :open="!!viewingEntry"
      :entry="viewingEntry"
      :category-name="viewingEntry?.categoryL1Id ? entryCategoryMap[viewingEntry.categoryL1Id] : undefined"
      @update:open="(val) => { if (!val) viewingEntry = undefined }"
      @adjust="(e) => { adjustingEntry = e }"
      @edit="(e) => { handleEditClick(e) }"
      @delete="handleDeleteFromDetail"
    />

    <!-- 删除确认 -->
    <Teleport to="body">
      <div v-if="confirmDeleteEntry" class="fixed inset-0 z-[100] flex items-end justify-center bg-black/40">
        <div class="bg-card border border-border rounded-t-3xl w-full max-w-lg p-6 pb-10 shadow-xl">
          <div class="w-10 h-1 bg-border rounded-full mx-auto mb-5" />
          <p class="font-semibold text-lg mb-2">删除「{{ confirmDeleteEntry.name }}」？</p>
          <p class="text-sm text-muted-foreground mb-6">此操作不可撤销。</p>
          <div class="flex gap-3">
            <button
              @click="confirmDeleteEntry = undefined"
              class="flex-1 py-3.5 rounded-2xl border border-border text-sm font-medium hover:bg-accent transition-colors cursor-pointer"
            >取消</button>
            <button
              @click="confirmDelete"
              class="flex-1 py-3.5 rounded-2xl bg-destructive text-destructive-foreground text-sm font-medium hover:opacity-90 transition-opacity cursor-pointer"
            >删除</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
