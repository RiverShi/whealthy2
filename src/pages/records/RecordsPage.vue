<script setup lang="ts">
import { onMounted, ref, computed, watch } from "vue";
import { recordApi, eventApi, type FlowRecord, type Event } from "@/api/records";
import { statsApi, type BookStats } from "@/api/views";
import { useCategoryStore } from "@/stores/categories";
import { useBookStore } from "@/stores/books";
import { useEntryStore } from "@/stores/entries";
import { ChevronLeft, ChevronRight, ChevronDown, Pencil, Trash2, TrendingUp, TrendingDown, Minus, FolderOpen } from "lucide-vue-next";
import RecordForm from "@/components/RecordForm.vue";
import RecordEditForm from "@/components/RecordEditForm.vue";
import RecordDetailSheet from "@/components/RecordDetailSheet.vue";

const bookStore = useBookStore();
const bookId = computed(() => bookStore.activeBookId ?? "");
const entryStore = useEntryStore();
const categoryStore = useCategoryStore();

// ── 月份选择 ──────────────────────────────────────────────────────────────
const today = new Date();
const currentYear = ref(today.getFullYear());
const currentMonth = ref(today.getMonth() + 1);

const monthLabel = computed(() => `${currentYear.value}年${currentMonth.value}月`);
const monthFrom = computed(() => `${currentYear.value}-${String(currentMonth.value).padStart(2, "0")}-01`);
const monthTo = computed(() => {
  const d = new Date(currentYear.value, currentMonth.value, 0);
  return `${currentYear.value}-${String(currentMonth.value).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
});

function prevMonth() {
  if (currentMonth.value === 1) { currentYear.value--; currentMonth.value = 12; }
  else currentMonth.value--;
}
function nextMonth() {
  const now = new Date();
  if (currentYear.value > now.getFullYear() || (currentYear.value === now.getFullYear() && currentMonth.value >= now.getMonth() + 1)) return;
  if (currentMonth.value === 12) { currentYear.value++; currentMonth.value = 1; }
  else currentMonth.value++;
}
const isCurrentMonth = computed(() => {
  const now = new Date();
  return currentYear.value === now.getFullYear() && currentMonth.value === now.getMonth() + 1;
});

// ── 数据 ──────────────────────────────────────────────────────────────────
const records = ref<FlowRecord[]>([]);
const events = ref<Event[]>([]);
const stats = ref<BookStats | null>(null);
const loading = ref(false);

async function refreshAll() {
  if (!bookId.value) return;
  loading.value = true;
  try {
    await Promise.all([
      categoryStore.fetchCategories("income"),
      categoryStore.fetchCategories("expense"),
      entryStore.fetchEntries(bookId.value),
    ]);
    [records.value, events.value, stats.value] = await Promise.all([
      recordApi.list(bookId.value, { from: monthFrom.value, to: monthTo.value }),
      eventApi.list(bookId.value),
      statsApi.getBookStats(bookId.value, monthFrom.value, monthTo.value),
    ]);
  } finally {
    loading.value = false;
  }
}
onMounted(refreshAll);
watch(bookId, (newId) => { if (newId) refreshAll(); });
watch([currentYear, currentMonth], () => { if (bookId.value) refreshAll(); });

// ── 分类/账户/事件映射 ────────────────────────────────────────────────────
const categoryMap = computed(() => {
  const m: Record<string, string> = {};
  categoryStore.categories.forEach((c) => (m[c.id] = c.name));
  return m;
});
const accountMap = computed(() => {
  const m: Record<string, string> = {};
  entryStore.entries.forEach((e) => { m[e.id] = e.name; });
  return m;
});
const eventMap = computed(() => {
  const m: Record<string, string> = {};
  events.value.forEach((e) => { m[e.id] = e.name; });
  return m;
});

// 唯一分类列表（筛选 chips）
const filterCategories = computed(() => {
  const ids = new Set(records.value.map((r) => r.categoryId).filter(Boolean) as string[]);
  return Array.from(ids).map((id) => ({ id, name: categoryMap.value[id] ?? id }));
});

// ── 筛选 ──────────────────────────────────────────────────────────────────
const filterCategoryId = ref<string | null>(null);
const filterType = ref<"all" | "income" | "expense" | "other">("all");

const filteredRecords = computed(() => {
  let rs = records.value;
  if (filterType.value === "income") rs = rs.filter((r) => r.type === "income");
  else if (filterType.value === "expense") rs = rs.filter((r) => r.type === "expense");
  else if (filterType.value === "other") rs = rs.filter((r) => ["transfer", "inflow", "outflow"].includes(r.type));
  if (filterCategoryId.value) rs = rs.filter((r) => r.categoryId === filterCategoryId.value);
  return rs;
});

// ── 事件展开状态 ──────────────────────────────────────────────────────────
const expandedEventIds = ref<Set<string>>(new Set());
function toggleEvent(id: string) {
  const s = new Set(expandedEventIds.value);
  if (s.has(id)) s.delete(id); else s.add(id);
  expandedEventIds.value = s;
}

// ── 统一列表项类型 ────────────────────────────────────────────────────────
type ListItem =
  | { kind: "record"; record: FlowRecord }
  | { kind: "event"; eventId: string; eventName: string; records: FlowRecord[] };

// ── 按日期分组（记录 + 事件混合排序） ─────────────────────────────────────
const groupedByDate = computed(() => {
  // 把有 eventId 的记录归到事件组，无 eventId 的独立展示
  const eventGroupMap = new Map<string, FlowRecord[]>();
  const standaloneRecords: FlowRecord[] = [];

  filteredRecords.value.forEach((r) => {
    if (r.eventId) {
      if (!eventGroupMap.has(r.eventId)) eventGroupMap.set(r.eventId, []);
      eventGroupMap.get(r.eventId)!.push(r);
    } else {
      standaloneRecords.push(r);
    }
  });

  // 合并为统一列表项（每项带排序 key 和日期）
  type DateItem = { date: string; sortKey: string; item: ListItem };
  const allItems: DateItem[] = [];

  standaloneRecords.forEach((r) => {
    allItems.push({ date: r.happenedAt.slice(0, 10), sortKey: r.happenedAt, item: { kind: "record", record: r } });
  });

  eventGroupMap.forEach((recs, eventId) => {
    const sorted = [...recs].sort((a, b) => b.happenedAt.localeCompare(a.happenedAt));
    const sortKey = sorted[0].happenedAt;
    allItems.push({
      date: sortKey.slice(0, 10),
      sortKey,
      item: { kind: "event", eventId, eventName: eventMap.value[eventId] ?? "未命名事件", records: sorted },
    });
  });

  // 按日期分组
  const dateMap = new Map<string, DateItem[]>();
  allItems.forEach((it) => {
    if (!dateMap.has(it.date)) dateMap.set(it.date, []);
    dateMap.get(it.date)!.push(it);
  });

  return Array.from(dateMap.entries())
    .sort((a, b) => b[0].localeCompare(a[0]))
    .map(([date, items]) => {
      items.sort((a, b) => b.sortKey.localeCompare(a.sortKey));
      const allRecs = items.flatMap((it) => it.item.kind === "record" ? [it.item.record] : it.item.records);
      const income = allRecs.filter((r) => r.type === "income").reduce((s, r) => s + r.amount, 0);
      const expense = allRecs.filter((r) => r.type === "expense").reduce((s, r) => s + r.amount, 0);
      return { date, items, income, expense };
    });
});

// ── 格式化 ────────────────────────────────────────────────────────────────
function fmtAmount(v: number) {
  return new Intl.NumberFormat("zh-CN", { minimumFractionDigits: 2, maximumFractionDigits: 2 }).format(v);
}
function fmtDateHeader(iso: string) {
  const d = new Date(iso + "T00:00:00");
  const todayD = new Date(); todayD.setHours(0, 0, 0, 0);
  const diff = Math.floor((todayD.getTime() - d.getTime()) / 86400000);
  const weekMap = ["日", "一", "二", "三", "四", "五", "六"];
  const weekday = `周${weekMap[d.getDay()]}`;
  if (diff === 0) return `${d.getMonth() + 1}月${d.getDate()}日  今天`;
  if (diff === 1) return `${d.getMonth() + 1}月${d.getDate()}日  昨天`;
  return `${d.getMonth() + 1}月${d.getDate()}日  ${weekday}`;
}
function recordLabel(r: FlowRecord) {
  if (r.categoryId && categoryMap.value[r.categoryId]) return categoryMap.value[r.categoryId];
  if (r.name) return r.name;
  const typeLabels: Record<string, string> = { income: "收入", expense: "支出", inflow: "流入", outflow: "流出", transfer: "划转" };
  return typeLabels[r.type] ?? r.type;
}
function recordNote(r: FlowRecord): string {
  const parts: string[] = [];
  if (r.note) parts.push(r.note);
  if (r.eventId && eventMap.value[r.eventId]) parts.push(eventMap.value[r.eventId]);
  return parts.join(" · ");
}

// ── 表单状态 ──────────────────────────────────────────────────────────────
const showRecordForm = ref(false);
const showRecordEdit = ref(false);
const editingRecord = ref<FlowRecord | null>(null);
const viewingRecord = ref<FlowRecord | null>(null);
const confirmDeleteId = ref<string | null>(null);

function openEditRecord(r: FlowRecord) { editingRecord.value = r; showRecordEdit.value = true; }
function askDelete(r: FlowRecord) { confirmDeleteId.value = r.id; }
async function confirmDelete() {
  if (!confirmDeleteId.value) return;
  await recordApi.delete(confirmDeleteId.value);
  confirmDeleteId.value = null;
  await refreshAll();
}

// ── 详情面板辅助 ──────────────────────────────────────────────────────────
const viewRecord_categoryName = computed(() => viewingRecord.value?.categoryId ? categoryMap.value[viewingRecord.value.categoryId] : undefined);
const viewRecord_fromAccountName = computed(() => viewingRecord.value?.fromAccountId ? accountMap.value[viewingRecord.value.fromAccountId] : undefined);
const viewRecord_toAccountName = computed(() => viewingRecord.value?.toAccountId ? accountMap.value[viewingRecord.value.toAccountId] : undefined);
const viewRecord_eventName = computed(() => viewingRecord.value?.eventId ? eventMap.value[viewingRecord.value.eventId] : undefined);
</script>

<template>
  <div class="min-h-full bg-background" style="padding-top: env(safe-area-inset-top)">

    <!-- ══ 顶部 ═════════════════════════════════════════════════════════ -->
    <div class="sticky top-0 z-20 bg-background/95 backdrop-blur-xl border-b border-border">
      <div class="flex items-center justify-between px-4 py-3">
        <h1 class="text-xl font-bold">账目</h1>
        <p class="text-xs text-muted-foreground">{{ bookStore.activeBook?.name }}</p>
      </div>

      <!-- 月份导航 -->
      <div class="flex items-center justify-between px-4 pb-3">
        <button @click="prevMonth" class="p-2 rounded-xl hover:bg-accent transition-colors cursor-pointer min-h-[44px] min-w-[44px] flex items-center justify-center">
          <ChevronLeft class="w-5 h-5" />
        </button>
        <p class="text-base font-semibold">{{ monthLabel }}</p>
        <button @click="nextMonth" :disabled="isCurrentMonth" class="p-2 rounded-xl hover:bg-accent transition-colors cursor-pointer min-h-[44px] min-w-[44px] flex items-center justify-center disabled:opacity-30">
          <ChevronRight class="w-5 h-5" />
        </button>
      </div>

      <!-- 月度概览 -->
      <div class="flex gap-4 px-4 pb-3 text-sm overflow-x-auto scrollbar-hide">
        <div class="flex items-center gap-1.5 shrink-0">
          <span class="text-muted-foreground">收入</span>
          <span class="font-semibold text-emerald-600 dark:text-emerald-400">¥{{ fmtAmount(stats?.income ?? 0) }}</span>
        </div>
        <div class="w-px bg-border shrink-0" />
        <div class="flex items-center gap-1.5 shrink-0">
          <span class="text-muted-foreground">支出</span>
          <span class="font-semibold text-rose-600 dark:text-rose-400">¥{{ fmtAmount(stats?.expense ?? 0) }}</span>
        </div>
        <div class="w-px bg-border shrink-0" />
        <div class="flex items-center gap-1.5 shrink-0">
          <span class="text-muted-foreground">其他</span>
          <span class="font-semibold text-muted-foreground">¥{{ fmtAmount(stats?.other ?? 0) }}</span>
        </div>
      </div>

      <!-- 筛选 Chips -->
      <div class="flex gap-2 px-4 pb-3 overflow-x-auto scrollbar-hide">
        <button @click="filterType = 'all'; filterCategoryId = null"
          class="shrink-0 px-3 py-1.5 rounded-full text-xs font-medium transition-colors cursor-pointer min-h-[32px]"
          :class="filterType === 'all' && !filterCategoryId ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground hover:bg-accent'">全部</button>
        <button @click="filterType = 'income'; filterCategoryId = null"
          class="shrink-0 px-3 py-1.5 rounded-full text-xs font-medium transition-colors cursor-pointer min-h-[32px]"
          :class="filterType === 'income' ? 'bg-emerald-500 text-white' : 'bg-muted text-muted-foreground hover:bg-accent'">收入</button>
        <button @click="filterType = 'expense'; filterCategoryId = null"
          class="shrink-0 px-3 py-1.5 rounded-full text-xs font-medium transition-colors cursor-pointer min-h-[32px]"
          :class="filterType === 'expense' ? 'bg-rose-500 text-white' : 'bg-muted text-muted-foreground hover:bg-accent'">支出</button>
        <button @click="filterType = 'other'; filterCategoryId = null"
          class="shrink-0 px-3 py-1.5 rounded-full text-xs font-medium transition-colors cursor-pointer min-h-[32px]"
          :class="filterType === 'other' ? 'bg-slate-500 text-white' : 'bg-muted text-muted-foreground hover:bg-accent'">其他</button>
        <div v-if="filterCategories.length" class="w-px bg-border shrink-0" />
        <button v-for="cat in filterCategories" :key="cat.id"
          @click="filterCategoryId = filterCategoryId === cat.id ? null : cat.id; filterType = 'all'"
          class="shrink-0 px-3 py-1.5 rounded-full text-xs font-medium transition-colors cursor-pointer min-h-[32px]"
          :class="filterCategoryId === cat.id ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground hover:bg-accent'">{{ cat.name }}</button>
      </div>
    </div>

    <!-- ══ 内容区 ════════════════════════════════════════════════════════ -->
    <div class="px-4 py-3 pb-8 space-y-4">

      <!-- 骨架屏 -->
      <template v-if="loading">
        <div v-for="i in 2" :key="i">
          <div class="h-4 bg-muted rounded animate-pulse w-32 mx-1 mb-2" />
          <div class="bg-card border border-border rounded-2xl overflow-hidden">
            <div v-for="j in 2" :key="j" class="flex items-center gap-3 px-4 py-4 border-b border-border/60 last:border-0">
              <div class="w-9 h-9 rounded-xl bg-muted animate-pulse shrink-0" />
              <div class="flex-1 space-y-2">
                <div class="h-3.5 bg-muted rounded animate-pulse w-28" />
                <div class="h-3 bg-muted rounded animate-pulse w-16" />
              </div>
              <div class="h-4 bg-muted rounded animate-pulse w-16" />
            </div>
          </div>
        </div>
      </template>

      <!-- 空状态 -->
      <div v-else-if="groupedByDate.length === 0" class="flex flex-col items-center justify-center py-20">
        <div class="w-16 h-16 rounded-2xl bg-muted flex items-center justify-center mb-4">
          <TrendingUp class="w-8 h-8 text-muted-foreground" />
        </div>
        <p class="text-sm text-muted-foreground">{{ monthLabel }}暂无记录</p>
      </div>

      <!-- 日期分组列表 -->
      <template v-else>
        <div v-for="group in groupedByDate" :key="group.date">
          <!-- 日期头 -->
          <div class="flex items-center justify-between px-1 mb-1.5">
            <p class="text-xs font-medium text-muted-foreground">{{ fmtDateHeader(group.date) }}</p>
            <div class="flex gap-3 text-xs">
              <span v-if="group.income > 0" class="text-emerald-600 dark:text-emerald-400">+{{ fmtAmount(group.income) }}</span>
              <span v-if="group.expense > 0" class="text-rose-600 dark:text-rose-400">-{{ fmtAmount(group.expense) }}</span>
            </div>
          </div>
          <!-- 混合卡片 -->
          <div class="bg-card border border-border rounded-2xl overflow-hidden">
            <template v-for="(it, idx) in group.items" :key="it.item.kind === 'record' ? it.item.record.id : it.item.eventId">

              <!-- ── 普通记录行 ──────────────────────────────────────────── -->
              <template v-if="it.item.kind === 'record'">
                <div
                  @click="viewingRecord = it.item.record"
                  class="flex items-center gap-3 px-4 py-3.5 cursor-pointer hover:bg-accent/40 active:bg-accent/60 transition-colors min-h-[64px]"
                  :class="idx < group.items.length - 1 ? 'border-b border-border/60' : ''"
                >
                  <div class="w-9 h-9 rounded-xl shrink-0 flex items-center justify-center"
                    :class="{
                      'bg-emerald-100 dark:bg-emerald-900/40': it.item.record.type === 'income',
                      'bg-rose-100 dark:bg-rose-900/40': it.item.record.type === 'expense',
                      'bg-blue-100 dark:bg-blue-900/40': it.item.record.type === 'inflow',
                      'bg-orange-100 dark:bg-orange-900/40': it.item.record.type === 'outflow',
                      'bg-slate-100 dark:bg-slate-800': it.item.record.type === 'transfer',
                    }">
                    <TrendingUp v-if="it.item.record.type === 'income'" class="w-4 h-4 text-emerald-600 dark:text-emerald-400" />
                    <TrendingDown v-else-if="it.item.record.type === 'expense'" class="w-4 h-4 text-rose-600 dark:text-rose-400" />
                    <TrendingUp v-else-if="it.item.record.type === 'inflow'" class="w-4 h-4 text-blue-600 dark:text-blue-400" />
                    <TrendingDown v-else-if="it.item.record.type === 'outflow'" class="w-4 h-4 text-orange-600 dark:text-orange-400" />
                    <Minus v-else class="w-4 h-4 text-muted-foreground" />
                  </div>
                  <div class="flex-1 min-w-0">
                    <p class="text-sm font-medium truncate">{{ recordLabel(it.item.record) }}</p>
                    <p v-if="recordNote(it.item.record)" class="text-xs text-muted-foreground mt-0.5 truncate">{{ recordNote(it.item.record) }}</p>
                  </div>
                  <div class="flex items-center gap-1.5 shrink-0">
                    <p class="text-sm font-semibold"
                      :class="{
                        'text-emerald-600 dark:text-emerald-400': it.item.record.type === 'income',
                        'text-rose-600 dark:text-rose-400': it.item.record.type === 'expense',
                        'text-blue-600 dark:text-blue-400': it.item.record.type === 'inflow',
                        'text-orange-600 dark:text-orange-400': it.item.record.type === 'outflow',
                        'text-muted-foreground': it.item.record.type === 'transfer',
                      }">
                      {{ ['income', 'inflow'].includes(it.item.record.type) ? '+' : ['expense', 'outflow'].includes(it.item.record.type) ? '-' : '' }}¥{{ fmtAmount(it.item.record.amount) }}
                    </p>
                    <button @click.stop="openEditRecord(it.item.record)" class="p-1.5 rounded-lg text-muted-foreground/50 hover:text-foreground hover:bg-accent transition-colors cursor-pointer">
                      <Pencil class="w-3.5 h-3.5" />
                    </button>
                    <button @click.stop="askDelete(it.item.record)" class="p-1.5 rounded-lg text-muted-foreground/50 hover:text-destructive hover:bg-destructive/10 transition-colors cursor-pointer">
                      <Trash2 class="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>
              </template>

              <!-- ── 事件行（可展开） ────────────────────────────────────── -->
              <template v-else-if="it.item.kind === 'event'">
                <!-- 事件标题行 -->
                <div
                  class="flex items-center gap-3 px-4 py-3.5 cursor-pointer hover:bg-accent/40 transition-colors min-h-[64px]"
                  :class="idx < group.items.length - 1 || expandedEventIds.has(it.item.eventId) ? 'border-b border-border/60' : ''"
                  @click="toggleEvent(it.item.eventId)"
                >
                  <div class="w-9 h-9 rounded-xl shrink-0 flex items-center justify-center bg-violet-100 dark:bg-violet-900/40">
                    <FolderOpen class="w-4 h-4 text-violet-600 dark:text-violet-400" />
                  </div>
                  <div class="flex-1 min-w-0">
                    <p class="text-sm font-medium truncate">{{ it.item.eventName }}</p>
                    <p class="text-xs text-muted-foreground mt-0.5">{{ it.item.records.length }} 条记录</p>
                  </div>
                  <div class="flex items-center gap-2 shrink-0">
                    <!-- 事件内收入/支出汇总 -->
                    <div class="text-right">
                      <p v-if="it.item.records.filter(r => r.type === 'income').reduce((s, r) => s + r.amount, 0) > 0" class="text-xs text-emerald-600 dark:text-emerald-400">
                        +¥{{ fmtAmount(it.item.records.filter(r => r.type === 'income').reduce((s, r) => s + r.amount, 0)) }}
                      </p>
                      <p v-if="it.item.records.filter(r => r.type === 'expense').reduce((s, r) => s + r.amount, 0) > 0" class="text-xs text-rose-600 dark:text-rose-400">
                        -¥{{ fmtAmount(it.item.records.filter(r => r.type === 'expense').reduce((s, r) => s + r.amount, 0)) }}
                      </p>
                    </div>
                    <ChevronDown
                      class="w-4 h-4 text-muted-foreground transition-transform duration-200"
                      :class="expandedEventIds.has(it.item.eventId) ? '' : '-rotate-90'"
                    />
                  </div>
                </div>
                <!-- 展开后的事件记录 -->
                <template v-if="expandedEventIds.has(it.item.eventId)">
                  <div
                    v-for="(r, rIdx) in it.item.records"
                    :key="r.id"
                    class="flex items-center gap-3 px-4 py-3 pl-14 cursor-pointer hover:bg-accent/20 transition-colors bg-muted/30"
                    :class="(rIdx < it.item.records.length - 1 || idx < group.items.length - 1) ? 'border-b border-border/60' : ''"
                    @click="viewingRecord = r"
                  >
                    <div class="flex-1 min-w-0">
                      <p class="text-sm font-medium truncate">{{ recordLabel(r) }}</p>
                      <p v-if="r.note" class="text-xs text-muted-foreground mt-0.5 truncate">{{ r.note }}</p>
                    </div>
                    <div class="flex items-center gap-1.5 shrink-0">
                      <p class="text-sm font-semibold"
                        :class="{
                          'text-emerald-600 dark:text-emerald-400': r.type === 'income',
                          'text-rose-600 dark:text-rose-400': r.type === 'expense',
                          'text-blue-600 dark:text-blue-400': r.type === 'inflow',
                          'text-orange-600 dark:text-orange-400': r.type === 'outflow',
                          'text-muted-foreground': r.type === 'transfer',
                        }">
                        {{ ['income', 'inflow'].includes(r.type) ? '+' : ['expense', 'outflow'].includes(r.type) ? '-' : '' }}¥{{ fmtAmount(r.amount) }}
                      </p>
                      <button @click.stop="openEditRecord(r)" class="p-1.5 rounded-lg text-muted-foreground/50 hover:text-foreground hover:bg-accent transition-colors cursor-pointer">
                        <Pencil class="w-3.5 h-3.5" />
                      </button>
                      <button @click.stop="askDelete(r)" class="p-1.5 rounded-lg text-muted-foreground/50 hover:text-destructive hover:bg-destructive/10 transition-colors cursor-pointer">
                        <Trash2 class="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </div>
                </template>
              </template>
            </template>
          </div>
        </div>
      </template>
    </div>

    <!-- ══ 弹窗 ══════════════════════════════════════════════════════════ -->
    <RecordForm :open="showRecordForm" :book-id="bookId" @update:open="showRecordForm = $event" @success="refreshAll" />
    <RecordEditForm :open="showRecordEdit" :book-id="bookId" :record="editingRecord" :events="events" @update:open="showRecordEdit = $event" @success="refreshAll" />
    <RecordDetailSheet
      :open="!!viewingRecord" :record="viewingRecord ?? undefined"
      :category-name="viewRecord_categoryName" :event-name="viewRecord_eventName"
      :from-account-name="viewRecord_fromAccountName" :to-account-name="viewRecord_toAccountName"
      @update:open="(v) => { if (!v) viewingRecord = null }"
      @edit="(r) => openEditRecord(r)" @delete="(r) => askDelete(r)"
    />
    <Teleport to="body">
      <div v-if="confirmDeleteId" class="fixed inset-0 z-[100] flex items-end justify-center bg-black/40" @click.self="confirmDeleteId = null">
        <div class="bg-card border border-border rounded-t-3xl w-full max-w-lg p-6 shadow-xl" style="padding-bottom: calc(env(safe-area-inset-bottom) + 24px)">
          <div class="w-10 h-1 bg-border rounded-full mx-auto mb-5" />
          <p class="font-semibold text-lg mb-1">确认删除？</p>
          <p class="text-sm text-muted-foreground mb-6">此操作不可撤销。</p>
          <div class="flex gap-3">
            <button @click="confirmDeleteId = null" class="flex-1 py-3.5 rounded-2xl border border-border text-sm font-medium hover:bg-accent transition-colors cursor-pointer">取消</button>
            <button @click="confirmDelete" class="flex-1 py-3.5 rounded-2xl bg-destructive text-destructive-foreground text-sm font-medium hover:opacity-90 cursor-pointer">删除</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
