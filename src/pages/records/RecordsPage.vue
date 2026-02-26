<script setup lang="ts">
import { onMounted, ref, computed, watch } from "vue";
import { recordApi, eventApi, feedApi, type FlowRecord, type Event, type EventSummary, type FeedItem } from "@/api/records";
import { useCategoryStore } from "@/stores/categories";
import { useBookStore } from "@/stores/books";
import { useEntryStore } from "@/stores/entries";
import { Plus, ChevronDown, ChevronRight, Pencil, Trash2, Receipt, Calendar, ArrowUpDown } from "lucide-vue-next";
import RecordForm from "@/components/RecordForm.vue";
import EventForm from "@/components/EventForm.vue";
import RecordEditForm from "@/components/RecordEditForm.vue";
import RecordDetailSheet from "@/components/RecordDetailSheet.vue";

const bookStore = useBookStore();
const bookId = computed(() => bookStore.activeBookId ?? "");
const entryStore = useEntryStore();

// ── 数据 ────────────────────────────────────────────────────────────────────────
const records = ref<FlowRecord[]>([]);
const events = ref<Event[]>([]);
const loading = ref(false);
const categoryStore = useCategoryStore();

// ── 查看详情状态 ──────────────────────────────────────────────────────────────────
const viewingRecord = ref<FlowRecord | null>(null);

// ── 视图模式：event 聚合 / 混合 Feed / 日期流水 ──────────────────────────────
type ViewMode = "event" | "date" | "mixed";
const viewMode = ref<ViewMode>("event");

// ── 混合 Feed 状态 ────────────────────────────────────────────────────────────
const feedItems = ref<FeedItem[]>([]);
const feedSortBy = ref<"date" | "amount">("date");

// ── 展开状态（event 视图）────────────────────────────────────────────────────
const expandedEvents = ref<Set<string>>(new Set());
function toggleEvent(id: string) {
  if (expandedEvents.value.has(id)) expandedEvents.value.delete(id);
  else expandedEvents.value.add(id);
}

// ── 表单弹窗状态 ──────────────────────────────────────────────────────────────
const showRecordForm = ref(false);
const recordFormEventId = ref<string | undefined>(undefined);

const showEventForm = ref(false);
const editingEvent = ref<Event | null>(null);

const showRecordEdit = ref(false);
const editingRecord = ref<FlowRecord | null>(null);

// ── 删除确认 ──────────────────────────────────────────────────────────────────
const confirmDeleteId = ref<string | null>(null);
const confirmDeleteType = ref<"event" | "record">("record");

function askDeleteEvent(ev: Event) {
  confirmDeleteId.value = ev.id;
  confirmDeleteType.value = "event";
}
function askDeleteRecord(r: FlowRecord) {
  confirmDeleteId.value = r.id;
  confirmDeleteType.value = "record";
}
async function confirmDelete() {
  if (!confirmDeleteId.value) return;
  try {
    if (confirmDeleteType.value === "event") {
      await eventApi.delete(confirmDeleteId.value);
    } else {
      await recordApi.delete(confirmDeleteId.value);
    }
    await refreshAll();
  } finally {
    confirmDeleteId.value = null;
  }
}

// ── 数据加载 ──────────────────────────────────────────────────────────────────
async function refreshAll() {
  if (!bookId.value) return;
  loading.value = true;
  try {
    await Promise.all([
      categoryStore.fetchCategories("income"),
      categoryStore.fetchCategories("expense"),
      entryStore.fetchEntries(bookId.value),
    ]);
    [records.value, events.value, feedItems.value] = await Promise.all([
      recordApi.list(bookId.value),
      eventApi.list(bookId.value),
      feedApi.list(bookId.value, { sortBy: feedSortBy.value }),
    ]);
  } finally {
    loading.value = false;
  }
}
onMounted(refreshAll);

// 账本切换时自动刷新
watch(bookId, (newId) => { if (newId) refreshAll(); });

// 切换排序时只重新拉取 Feed
watch(feedSortBy, async () => {
  if (bookId.value)
    feedItems.value = await feedApi.list(bookId.value, { sortBy: feedSortBy.value });
});

// 从 feedItems 中的 EventSummary 找回完整 Event 对象（供编辑/删除操作用）
function findEvent(id: string): Event {
  return events.value.find((e) => e.id === id)!;
}

// ── 计算辅助 ──────────────────────────────────────────────────────────────────
const categoryMap = computed(() => {
  const m: { [k: string]: string } = {};
  categoryStore.categories.forEach((c) => (m[c.id] = c.name));
  return m;
});
const accountMap = computed(() => {
  const m: Record<string, string> = {};
  entryStore.entries.forEach(e => { m[e.id] = e.name; });
  return m;
});

// 查看记录的详情计算属性
const viewRecord_categoryName = computed(() =>
  viewingRecord.value?.categoryId ? categoryMap.value[viewingRecord.value.categoryId] : undefined
);
const viewRecord_eventName = computed(() =>
  viewingRecord.value?.eventId ? events.value.find(e => e.id === viewingRecord.value!.eventId)?.name : undefined
);
const viewRecord_fromAccountName = computed(() =>
  viewingRecord.value?.fromAccountId ? accountMap.value[viewingRecord.value.fromAccountId] : undefined
);
const viewRecord_toAccountName = computed(() =>
  viewingRecord.value?.toAccountId ? accountMap.value[viewingRecord.value.toAccountId] : undefined
);

// 记录行第二行文字
function recordSubtitle(r: FlowRecord): string {
  const parts: string[] = [];
  const cat = r.categoryId ? categoryMap.value[r.categoryId] : null;
  if (cat) parts.push(cat);
  parts.push(fmtDate(r.happenedAt));
  return parts.join(" · ");
}
const freeRecords = computed(() =>
  records.value.filter((r) => !r.eventId)
);

const recordsByEvent = computed(() => {
  const m = new Map<string, FlowRecord[]>();
  records.value.forEach((r) => {
    if (r.eventId) {
      if (!m.has(r.eventId)) m.set(r.eventId, []);
      m.get(r.eventId)!.push(r);
    }
  });
  return m;
});

const recordsByDate = computed(() => {
  const groups = new Map<string, FlowRecord[]>();
  records.value.forEach((r) => {
    const date = r.happenedAt.split("T")[0];
    if (!groups.has(date)) groups.set(date, []);
    groups.get(date)!.push(r);
  });
  return Array.from(groups.entries()).sort((a, b) => b[0].localeCompare(a[0]));
});

function recordLabel(r: FlowRecord) {
  return r.name || categoryMap.value[r.categoryId ?? ""] || r.note || "—";
}

function groupTotal(recs: FlowRecord[]) {
  let income = 0, expense = 0;
  recs.forEach((r) => {
    if (r.type === "income") income += r.amount;
    else if (r.type === "expense") expense += r.amount;
  });
  return { income, expense };
}

function fmtDate(iso: string) {
  return new Date(iso.slice(0, 10)).toLocaleDateString("zh-CN", {
    month: "long", day: "numeric", weekday: "short",
  });
}

const typeColor: { [k: string]: string } = {
  income: "text-emerald-500",
  expense: "text-rose-500",
  transfer: "text-muted-foreground",
};
const typeLabel: { [k: string]: string } = {
  income: "收入", expense: "支出", transfer: "不计收支",
};

// ── 操作入口 ──────────────────────────────────────────────────────────────────
function openNewRecord(presetEventId?: string) {
  recordFormEventId.value = presetEventId;
  showRecordForm.value = true;
}
function openNewEvent() {
  editingEvent.value = null;
  showEventForm.value = true;
}
function openEditEvent(ev: Event) {
  editingEvent.value = ev;
  showEventForm.value = true;
}
function openEditRecord(r: FlowRecord) {
  editingRecord.value = r;
  showRecordEdit.value = true;
}
function openViewRecord(r: FlowRecord) {
  viewingRecord.value = r;
}
</script>

<template>
  <div class="min-h-full bg-background" style="padding-top: env(safe-area-inset-top)">

    <!-- ══ 顶部固定头部 ═══════════════════════════════════════════════════ -->
    <div class="sticky top-0 z-20 bg-card/95 backdrop-blur-xl border-b border-border">
      <div class="px-4 py-3">
        <!-- 第一行：标题 + 快捷操作 -->
        <div class="flex items-center justify-between gap-2 mb-2.5">
          <div>
            <p class="text-[11px] text-muted-foreground leading-none mb-0.5">{{ bookStore.activeBook?.name }}</p>
            <h1 class="text-xl font-bold leading-tight">账目</h1>
          </div>
          <div class="flex items-center gap-2">
            <button
              @click="openNewEvent()"
              class="flex items-center gap-1.5 px-3 py-2 border border-border rounded-xl text-sm font-medium hover:bg-accent transition-colors cursor-pointer"
            ><Plus class="w-3.5 h-3.5" />事件</button>
            <button
              @click="openNewRecord()"
              class="flex items-center gap-1.5 px-3 py-2 bg-primary text-primary-foreground rounded-xl text-sm font-medium hover:opacity-90 transition-opacity cursor-pointer"
            ><Plus class="w-3.5 h-3.5" />流水</button>
          </div>
        </div>

        <!-- 第二行：视图模式 + 排序 -->
        <div class="flex items-center gap-2">
          <div class="flex rounded-xl border border-border overflow-hidden text-xs flex-1">
            <button
              @click="viewMode = 'event'"
              class="flex-1 px-2 py-2 transition-colors font-medium"
              :class="viewMode === 'event' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-accent'"
            >按事件</button>
            <button
              @click="viewMode = 'mixed'"
              class="flex-1 px-2 py-2 transition-colors font-medium"
              :class="viewMode === 'mixed' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-accent'"
            >混合</button>
            <button
              @click="viewMode = 'date'"
              class="flex-1 px-2 py-2 transition-colors font-medium"
              :class="viewMode === 'date' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-accent'"
            >按日期</button>
          </div>
          <!-- 混合视图：排序 -->
          <div v-if="viewMode === 'mixed'" class="flex rounded-xl border border-border overflow-hidden text-xs shrink-0">
            <button
              @click="feedSortBy = 'date'"
              class="flex items-center gap-1 px-2.5 py-2 transition-colors"
              :class="feedSortBy === 'date' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-accent'"
            ><Calendar class="w-3 h-3" />时间</button>
            <button
              @click="feedSortBy = 'amount'"
              class="flex items-center gap-1 px-2.5 py-2 transition-colors"
              :class="feedSortBy === 'amount' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-accent'"
            ><ArrowUpDown class="w-3 h-3" />金额</button>
          </div>
        </div>
      </div>
    </div>

    <!-- ══ 内容区 ════════════════════════════════════════════════════════ -->
    <div class="px-4 py-3 pb-6">

    <!-- 加载 -->
    <div v-if="loading" class="text-center py-16 text-muted-foreground text-sm">加载中…</div>

    <!-- ═══ 事件视图 ═══════════════════════════════════════════════════════ -->
    <div v-else-if="viewMode === 'event'" class="space-y-3">

      <!-- 空状态 -->
      <div v-if="!events.length && !freeRecords.length" class="text-center py-16">
        <div class="w-14 h-14 rounded-2xl bg-muted flex items-center justify-center mb-4 mx-auto">
          <Receipt class="w-7 h-7 text-muted-foreground" />
        </div>
        <p class="text-base font-medium mb-1">还没有记录</p>
        <p class="text-sm text-muted-foreground">点击上方「流水」或「事件」添加</p>
      </div>

      <!-- 事件卡片 -->
      <div
        v-for="ev in events"
        :key="ev.id"
        class="rounded-2xl border border-border bg-card overflow-hidden"
      >
        <!-- 事件头部 -->
        <div
          class="flex items-center gap-3 px-4 py-3.5 cursor-pointer active:bg-accent/40 transition-colors select-none"
          @click="toggleEvent(ev.id)"
          style="min-height: 56px"
        >
          <component
            :is="expandedEvents.has(ev.id) ? ChevronDown : ChevronRight"
            class="w-4 h-4 text-muted-foreground shrink-0"
          />
          <div class="flex-1 min-w-0">
            <p class="text-sm font-semibold truncate">{{ ev.name }}</p>
            <p v-if="ev.description" class="text-xs text-muted-foreground truncate">{{ ev.description }}</p>
          </div>
          <!-- 事件合计 -->
          <div class="flex gap-2 text-xs tabular-nums shrink-0 mr-1">
            <span
              v-if="groupTotal(recordsByEvent.get(ev.id) ?? []).income"
              class="text-emerald-500"
            >+{{ groupTotal(recordsByEvent.get(ev.id) ?? []).income.toFixed(2) }}</span>
            <span
              v-if="groupTotal(recordsByEvent.get(ev.id) ?? []).expense"
              class="text-rose-500"
            >-{{ groupTotal(recordsByEvent.get(ev.id) ?? []).expense.toFixed(2) }}</span>
            <span
              v-if="!(recordsByEvent.get(ev.id) ?? []).length"
              class="text-muted-foreground"
            >暂无</span>
          </div>
          <!-- 操作按钮 -->
          <div class="flex gap-0.5 shrink-0" @click.stop>
            <button
              @click="openNewRecord(ev.id)"
              class="p-2 rounded-lg hover:bg-accent text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
            ><Plus class="w-4 h-4" /></button>
            <button
              @click="openEditEvent(ev)"
              class="p-2 rounded-lg hover:bg-accent text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
            ><Pencil class="w-4 h-4" /></button>
            <button
              @click="askDeleteEvent(ev)"
              class="p-2 rounded-lg hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors cursor-pointer"
            ><Trash2 class="w-4 h-4" /></button>
          </div>
        </div>

        <!-- 展开后的流水 -->
        <div v-if="expandedEvents.has(ev.id)" class="border-t border-border">
          <div
            v-for="r in recordsByEvent.get(ev.id) ?? []"
            :key="r.id"
            class="flex items-center gap-3 px-4 py-3 border-b border-border/60 last:border-0 cursor-pointer active:bg-accent/40 transition-colors"
            @click="openViewRecord(r)"
          >
            <span class="text-xs font-medium px-2 py-0.5 rounded-full bg-muted shrink-0" :class="typeColor[r.type]">
              {{ typeLabel[r.type] }}
            </span>
            <div class="flex-1 min-w-0">
              <p class="text-sm truncate">{{ recordLabel(r) }}</p>
              <p class="text-xs text-muted-foreground truncate">{{ recordSubtitle(r) }}</p>
            </div>
            <span class="text-sm font-semibold tabular-nums shrink-0" :class="typeColor[r.type]">
              {{ r.type === 'income' ? '+' : r.type === 'expense' ? '-' : '' }}{{ r.amount.toFixed(2) }}
            </span>
            <div class="flex gap-0.5 shrink-0" @click.stop>
              <button @click="openEditRecord(r)" class="p-2 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors cursor-pointer"><Pencil class="w-3.5 h-3.5" /></button>
              <button @click="askDeleteRecord(r)" class="p-2 rounded hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors cursor-pointer"><Trash2 class="w-3.5 h-3.5" /></button>
            </div>
          </div>
          <div v-if="!(recordsByEvent.get(ev.id) ?? []).length" class="px-4 py-3 text-xs text-muted-foreground flex items-center gap-2">
            <span>暂无流水</span>
            <button @click="openNewRecord(ev.id)" class="text-primary hover:underline cursor-pointer">+ 添加</button>
          </div>
        </div>
      </div>

      <!-- 未归属事件的散记流水 -->
      <div v-if="freeRecords.length" class="rounded-2xl border border-border bg-card overflow-hidden">
        <div class="px-4 py-3 flex items-center gap-2 border-b border-border">
          <Calendar class="w-4 h-4 text-muted-foreground" />
          <span class="text-sm font-semibold text-muted-foreground">未归属事件的流水</span>
        </div>
        <div>
          <div
            v-for="r in freeRecords"
            :key="r.id"
            class="flex items-center gap-3 px-4 py-3 border-b border-border/60 last:border-0 cursor-pointer active:bg-accent/40 transition-colors"
            @click="openViewRecord(r)"
          >
            <span class="text-xs font-medium px-2 py-0.5 rounded-full bg-muted shrink-0" :class="typeColor[r.type]">
              {{ typeLabel[r.type] }}
            </span>
            <div class="flex-1 min-w-0">
              <p class="text-sm truncate">{{ recordLabel(r) }}</p>
              <p class="text-xs text-muted-foreground truncate">{{ recordSubtitle(r) }}</p>
            </div>
            <span class="text-sm font-semibold tabular-nums shrink-0" :class="typeColor[r.type]">
              {{ r.type === 'income' ? '+' : r.type === 'expense' ? '-' : '' }}{{ r.amount.toFixed(2) }}
            </span>
            <div class="flex gap-0.5 shrink-0" @click.stop>
              <button @click="openEditRecord(r)" class="p-2 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors cursor-pointer"><Pencil class="w-3.5 h-3.5" /></button>
              <button @click="askDeleteRecord(r)" class="p-2 rounded hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors cursor-pointer"><Trash2 class="w-3.5 h-3.5" /></button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- ═══ 混合 Feed ════════════════════════════════════════════════════════ -->
    <div v-else-if="viewMode === 'mixed'" class="space-y-3">
      <!-- 空状态 -->
      <div v-if="!feedItems.length" class="text-center py-16">
        <div class="w-14 h-14 rounded-2xl bg-muted flex items-center justify-center mb-4 mx-auto">
          <Receipt class="w-7 h-7 text-muted-foreground" />
        </div>
        <p class="text-base font-medium mb-1">还没有记录</p>
        <p class="text-sm text-muted-foreground">点击上方按钮添加</p>
      </div>

      <template v-for="item in feedItems" :key="item.id">
        <!-- 事件卡片 -->
        <div v-if="item.itemType === 'event'" class="rounded-2xl border border-border bg-card overflow-hidden">
          <div
            class="flex items-center gap-3 px-4 py-3.5 cursor-pointer active:bg-accent/40 transition-colors select-none"
            @click="toggleEvent(item.id)"
            style="min-height: 56px"
          >
            <component
              :is="expandedEvents.has(item.id) ? ChevronDown : ChevronRight"
              class="w-4 h-4 text-muted-foreground shrink-0"
            />
            <div class="flex-1 min-w-0">
              <p class="text-sm font-semibold truncate">{{ item.name }}</p>
              <p class="text-xs text-muted-foreground">
                {{ (item as EventSummary).latestHappenedAt ?? item.createdAt }}
                · {{ (item as EventSummary).recordCount }} 条流水
              </p>
            </div>
            <!-- 聚合金额 -->
            <div class="flex gap-2 text-xs tabular-nums shrink-0 mr-1">
              <span v-if="(item as EventSummary).totalIncome" class="text-emerald-500">
                +{{ (item as EventSummary).totalIncome.toFixed(2) }}
              </span>
              <span v-if="(item as EventSummary).totalExpense" class="text-rose-500">
                -{{ (item as EventSummary).totalExpense.toFixed(2) }}
              </span>
              <span v-if="!(item as EventSummary).recordCount" class="text-muted-foreground">暂无</span>
            </div>
            <!-- 操作按钮 -->
            <div class="flex gap-0.5 shrink-0" @click.stop>
              <button
                @click="openNewRecord(item.id)"
                class="p-2 rounded-lg hover:bg-accent text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
              ><Plus class="w-4 h-4" /></button>
              <button
                @click="openEditEvent(findEvent(item.id))"
                class="p-2 rounded-lg hover:bg-accent text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
              ><Pencil class="w-4 h-4" /></button>
              <button
                @click="askDeleteEvent(findEvent(item.id))"
                class="p-2 rounded-lg hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors cursor-pointer"
              ><Trash2 class="w-4 h-4" /></button>
            </div>
          </div>
          <!-- 展开后的流水 -->
          <div v-if="expandedEvents.has(item.id)" class="border-t border-border">
            <div
              v-for="r in recordsByEvent.get(item.id) ?? []"
              :key="r.id"
              class="flex items-center gap-3 px-4 py-3 border-b border-border/60 last:border-0 cursor-pointer active:bg-accent/40 transition-colors"
              @click="openViewRecord(r)"
            >
              <span class="text-xs font-medium px-2 py-0.5 rounded-full bg-muted shrink-0" :class="typeColor[r.type]">
                {{ typeLabel[r.type] }}
              </span>
              <div class="flex-1 min-w-0">
                <p class="text-sm truncate">{{ recordLabel(r) }}</p>
                <p class="text-xs text-muted-foreground truncate">{{ recordSubtitle(r) }}</p>
              </div>
              <span class="text-sm font-semibold tabular-nums shrink-0" :class="typeColor[r.type]">
                {{ r.type === 'income' ? '+' : r.type === 'expense' ? '-' : '' }}{{ r.amount.toFixed(2) }}
              </span>
              <div class="flex gap-0.5 shrink-0" @click.stop>
                <button @click="openEditRecord(r)" class="p-2 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors cursor-pointer"><Pencil class="w-3.5 h-3.5" /></button>
                <button @click="askDeleteRecord(r)" class="p-2 rounded hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors cursor-pointer"><Trash2 class="w-3.5 h-3.5" /></button>
              </div>
            </div>
            <div v-if="!(recordsByEvent.get(item.id) ?? []).length" class="px-4 py-3 text-xs text-muted-foreground flex items-center gap-2">
              <span>暂无流水</span>
              <button @click="openNewRecord(item.id)" class="text-primary hover:underline cursor-pointer">+ 添加</button>
            </div>
          </div>
        </div>

        <!-- 独立流水卡片 -->
        <div v-else class="rounded-2xl border border-border bg-card flex items-center gap-3 px-4 py-3.5 cursor-pointer active:bg-accent/40 transition-colors" @click="openViewRecord(item as FlowRecord)">
          <span class="text-xs font-medium px-2 py-0.5 rounded-full bg-muted shrink-0" :class="typeColor[(item as FlowRecord).type]">
            {{ typeLabel[(item as FlowRecord).type] }}
          </span>
          <div class="flex-1 min-w-0">
            <p class="text-sm truncate">{{ recordLabel(item as FlowRecord) }}</p>
            <p class="text-xs text-muted-foreground truncate">{{ recordSubtitle(item as FlowRecord) }}</p>
          </div>
          <span class="text-sm font-semibold tabular-nums shrink-0" :class="typeColor[(item as FlowRecord).type]">
            {{ (item as FlowRecord).type === 'income' ? '+' : (item as FlowRecord).type === 'expense' ? '-' : '' }}{{ (item as FlowRecord).amount.toFixed(2) }}
          </span>
          <div class="flex gap-0.5 shrink-0" @click.stop>
            <button @click="openEditRecord(item as FlowRecord)" class="p-2 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors cursor-pointer"><Pencil class="w-3.5 h-3.5" /></button>
            <button @click="askDeleteRecord(item as FlowRecord)" class="p-2 rounded hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors cursor-pointer"><Trash2 class="w-3.5 h-3.5" /></button>
          </div>
        </div>
      </template>
    </div>

    <!-- ═══ 日期视图 ════════════════════════════════════════════════════════ -->
    <div v-else class="space-y-4">
      <div v-if="!records.length" class="text-center py-16">
        <div class="w-14 h-14 rounded-2xl bg-muted flex items-center justify-center mb-4 mx-auto">
          <Receipt class="w-7 h-7 text-muted-foreground" />
        </div>
        <p class="text-base font-medium mb-1">还没有流水记录</p>
      </div>
      <div v-for="[date, dayRecs] in recordsByDate" :key="date">
        <div class="flex items-center justify-between mb-2 px-1">
          <span class="text-sm font-medium text-muted-foreground">{{ fmtDate(date) }}</span>
          <span class="text-xs space-x-2">
            <span v-if="groupTotal(dayRecs).income" class="text-emerald-500">+{{ groupTotal(dayRecs).income.toFixed(2) }}</span>
            <span v-if="groupTotal(dayRecs).expense" class="text-rose-500">-{{ groupTotal(dayRecs).expense.toFixed(2) }}</span>
          </span>
        </div>
        <div class="rounded-2xl border border-border bg-card overflow-hidden">
          <div
            v-for="r in dayRecs"
            :key="r.id"
            class="flex items-center gap-3 px-4 py-3 border-b border-border/60 last:border-0 cursor-pointer active:bg-accent/40 transition-colors"
            @click="openViewRecord(r)"
          >
            <span class="text-xs font-medium px-2 py-0.5 rounded-full bg-muted shrink-0" :class="typeColor[r.type]">
              {{ typeLabel[r.type] }}
            </span>
            <div class="flex-1 min-w-0">
              <p class="text-sm truncate">{{ recordLabel(r) }}</p>
              <p class="text-xs text-muted-foreground truncate">
                <template v-if="r.categoryId && categoryMap[r.categoryId]">{{ categoryMap[r.categoryId] }}</template>
                <template v-else-if="r.eventId">{{ events.find(e => e.id === r.eventId)?.name ?? '事件' }}</template>
              </p>
            </div>
            <span class="text-sm font-semibold tabular-nums shrink-0" :class="typeColor[r.type]">
              {{ r.type === 'income' ? '+' : r.type === 'expense' ? '-' : '' }}{{ r.amount.toFixed(2) }}
            </span>
            <div class="flex gap-0.5 shrink-0" @click.stop>
              <button @click="openEditRecord(r)" class="p-2 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors cursor-pointer"><Pencil class="w-3.5 h-3.5" /></button>
              <button @click="askDeleteRecord(r)" class="p-2 rounded hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors cursor-pointer"><Trash2 class="w-3.5 h-3.5" /></button>
            </div>
          </div>
        </div>
      </div>
    </div>
    </div>
  </div>

  <!-- ══ 弹窗组件 ══════════════════════════════════════════════════════════ -->
  <RecordForm
    :open="showRecordForm"
    :book-id="bookId"
    :preset-event-id="recordFormEventId"
    @update:open="showRecordForm = $event"
    @success="refreshAll"
  />

  <EventForm
    :open="showEventForm"
    :book-id="bookId"
    :event="editingEvent"
    @update:open="showEventForm = $event"
    @success="refreshAll"
  />

  <RecordEditForm
    :open="showRecordEdit"
    :book-id="bookId"
    :record="editingRecord"
    :events="events"
    @update:open="showRecordEdit = $event"
    @success="refreshAll"
  />

  <!-- 记录详情面板 -->
  <RecordDetailSheet
    :open="!!viewingRecord"
    :record="viewingRecord ?? undefined"
    :category-name="viewRecord_categoryName"
    :event-name="viewRecord_eventName"
    :from-account-name="viewRecord_fromAccountName"
    :to-account-name="viewRecord_toAccountName"
    @update:open="(val) => { if (!val) viewingRecord = null }"
    @edit="(r) => { openEditRecord(r) }"
    @delete="(r) => { askDeleteRecord(r) }"
  />

  <!-- 删除确认 -->
  <Teleport to="body">
    <div v-if="confirmDeleteId" class="fixed inset-0 z-[100] flex items-end justify-center bg-black/40">
      <div class="bg-card border border-border rounded-t-3xl w-full max-w-lg p-6 pb-10 shadow-xl animate-in">
        <div class="w-10 h-1 bg-border rounded-full mx-auto mb-5" />
        <p class="font-semibold text-lg mb-2">确认删除？</p>
        <p class="text-sm text-muted-foreground mb-6">
          <template v-if="confirmDeleteType === 'event'">
            删除事件后，其下流水记录不会删除，会变为「未归属」状态。
          </template>
          <template v-else>此操作不可撤销。</template>
        </p>
        <div class="flex gap-3">
          <button
            @click="confirmDeleteId = null"
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
</template>

