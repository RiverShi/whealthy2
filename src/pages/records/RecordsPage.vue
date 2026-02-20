<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { recordApi, eventApi, type FlowRecord, type Event } from "@/api/records";
import { useCategoryStore } from "@/stores/categories";
import { Plus, ChevronDown, ChevronRight, Pencil, Trash2, Receipt, Calendar } from "lucide-vue-next";
import RecordForm from "@/components/RecordForm.vue";
import EventForm from "@/components/EventForm.vue";
import RecordEditForm from "@/components/RecordEditForm.vue";

const route = useRoute();
const router = useRouter();
const bookId = route.params.id as string;

// ── 数据 ──────────────────────────────────────────────────────────────────────
const records = ref<FlowRecord[]>([]);
const events = ref<Event[]>([]);
const loading = ref(false);
const categoryStore = useCategoryStore();

// ── 视图模式：event 聚合 / 日期流水 ──────────────────────────────────────────
type ViewMode = "event" | "date";
const viewMode = ref<ViewMode>("event");

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
  loading.value = true;
  try {
    const [recs, evs] = await Promise.all([
      recordApi.list(bookId),
      eventApi.list(bookId),
      categoryStore.fetchCategories("income"),
      categoryStore.fetchCategories("expense"),
    ]);
    records.value = recs;
    events.value = evs;
  } finally {
    loading.value = false;
  }
}
onMounted(refreshAll);

// ── 计算辅助 ──────────────────────────────────────────────────────────────────
const categoryMap = computed(() => {
  const m: { [k: string]: string } = {};
  categoryStore.categories.forEach((c) => (m[c.id] = c.name));
  return m;
});

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
  return new Date(iso.slice(0, 10) + "T00:00:00").toLocaleDateString("zh-CN", {
    month: "long", day: "numeric", weekday: "short",
  });
}

const typeColor: { [k: string]: string } = {
  income: "text-emerald-500",
  expense: "text-rose-500",
  transfer: "text-muted-foreground",
};
const typeLabel: { [k: string]: string } = {
  income: "收入", expense: "支出", transfer: "转账",
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
</script>

<template>
  <div class="p-6 max-w-2xl mx-auto pb-20">
    <!-- 页头 -->
    <div class="flex items-start justify-between mb-6 gap-4">
      <div class="min-w-0">
        <button
          @click="router.back()"
          class="text-xs text-muted-foreground hover:text-foreground mb-1 flex items-center gap-1"
        >← 返回账本</button>
        <h1 class="text-2xl font-bold">流水记录</h1>
      </div>
      <div class="flex items-center gap-2 shrink-0 flex-wrap justify-end">
        <!-- 视图切换 -->
        <div class="flex rounded-lg border border-border overflow-hidden text-xs">
          <button
            @click="viewMode = 'event'"
            class="px-3 py-1.5 transition-colors"
            :class="viewMode === 'event' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-accent'"
          >按事件</button>
          <button
            @click="viewMode = 'date'"
            class="px-3 py-1.5 transition-colors"
            :class="viewMode === 'date' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-accent'"
          >按日期</button>
        </div>
        <button
          @click="openNewEvent()"
          class="flex items-center gap-1 px-3 py-1.5 border border-border rounded-lg text-sm font-medium hover:bg-accent transition-colors"
        ><Plus class="w-3.5 h-3.5" />新增事件</button>
        <button
          @click="openNewRecord()"
          class="flex items-center gap-1 px-3 py-1.5 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:opacity-90 transition-opacity"
        ><Plus class="w-3.5 h-3.5" />新增流水</button>
      </div>
    </div>

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
        <p class="text-sm text-muted-foreground">点击右上角「新增流水」或「新增事件」</p>
      </div>

      <!-- 事件卡片 -->
      <div
        v-for="ev in events"
        :key="ev.id"
        class="rounded-2xl border border-border bg-card overflow-hidden"
      >
        <!-- 事件头部 -->
        <div
          class="flex items-center gap-3 px-4 py-3 cursor-pointer hover:bg-accent/40 transition-colors select-none"
          @click="toggleEvent(ev.id)"
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
            >暂无流水</span>
          </div>
          <!-- 操作按钮 -->
          <div class="flex gap-0.5 shrink-0" @click.stop>
            <button
              @click="openNewRecord(ev.id)"
              class="p-1.5 rounded-lg hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
              title="添加流水到此事件"
            ><Plus class="w-3.5 h-3.5" /></button>
            <button
              @click="openEditEvent(ev)"
              class="p-1.5 rounded-lg hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
              title="编辑事件"
            ><Pencil class="w-3.5 h-3.5" /></button>
            <button
              @click="askDeleteEvent(ev)"
              class="p-1.5 rounded-lg hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors"
              title="删除事件"
            ><Trash2 class="w-3.5 h-3.5" /></button>
          </div>
        </div>

        <!-- 展开后的流水 -->
        <div v-if="expandedEvents.has(ev.id)" class="border-t border-border">
          <div
            v-for="r in recordsByEvent.get(ev.id) ?? []"
            :key="r.id"
            class="flex items-center gap-3 px-4 py-2.5 border-b border-border/60 last:border-0"
          >
            <span class="text-xs font-medium px-2 py-0.5 rounded-full bg-muted shrink-0" :class="typeColor[r.type]">
              {{ typeLabel[r.type] }}
            </span>
            <div class="flex-1 min-w-0">
              <p class="text-sm truncate">{{ recordLabel(r) }}</p>
              <p class="text-xs text-muted-foreground">{{ fmtDate(r.happenedAt) }}</p>
            </div>
            <span class="text-sm font-semibold tabular-nums shrink-0" :class="typeColor[r.type]">
              {{ r.type === 'income' ? '+' : r.type === 'expense' ? '-' : '' }}{{ r.amount.toFixed(2) }}
            </span>
            <div class="flex gap-0.5 shrink-0">
              <button @click="openEditRecord(r)" class="p-1 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"><Pencil class="w-3 h-3" /></button>
              <button @click="askDeleteRecord(r)" class="p-1 rounded hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors"><Trash2 class="w-3 h-3" /></button>
            </div>
          </div>
          <div v-if="!(recordsByEvent.get(ev.id) ?? []).length" class="px-4 py-3 text-xs text-muted-foreground flex items-center gap-2">
            <span>暂无流水</span>
            <button @click="openNewRecord(ev.id)" class="text-primary hover:underline">+ 添加</button>
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
            class="flex items-center gap-3 px-4 py-2.5 border-b border-border/60 last:border-0"
          >
            <span class="text-xs font-medium px-2 py-0.5 rounded-full bg-muted shrink-0" :class="typeColor[r.type]">
              {{ typeLabel[r.type] }}
            </span>
            <div class="flex-1 min-w-0">
              <p class="text-sm truncate">{{ recordLabel(r) }}</p>
              <p class="text-xs text-muted-foreground">{{ fmtDate(r.happenedAt) }}</p>
            </div>
            <span class="text-sm font-semibold tabular-nums shrink-0" :class="typeColor[r.type]">
              {{ r.type === 'income' ? '+' : r.type === 'expense' ? '-' : '' }}{{ r.amount.toFixed(2) }}
            </span>
            <div class="flex gap-0.5 shrink-0">
              <button @click="openEditRecord(r)" class="p-1 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"><Pencil class="w-3 h-3" /></button>
              <button @click="askDeleteRecord(r)" class="p-1 rounded hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors"><Trash2 class="w-3 h-3" /></button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- ═══ 日期视图 ════════════════════════════════════════════════════════ -->
    <div v-else class="space-y-6">
      <div v-if="!records.length" class="text-center py-16">
        <div class="w-14 h-14 rounded-2xl bg-muted flex items-center justify-center mb-4 mx-auto">
          <Receipt class="w-7 h-7 text-muted-foreground" />
        </div>
        <p class="text-base font-medium mb-1">还没有流水记录</p>
      </div>
      <div v-for="[date, dayRecs] in recordsByDate" :key="date">
        <div class="flex items-center justify-between mb-2">
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
            class="flex items-center gap-3 px-4 py-2.5 border-b border-border/60 last:border-0"
          >
            <span class="text-xs font-medium px-2 py-0.5 rounded-full bg-muted shrink-0" :class="typeColor[r.type]">
              {{ typeLabel[r.type] }}
            </span>
            <div class="flex-1 min-w-0">
              <p class="text-sm truncate">{{ recordLabel(r) }}</p>
              <p v-if="r.eventId" class="text-xs text-muted-foreground">
                📎 {{ events.find(e => e.id === r.eventId)?.name ?? '事件' }}
              </p>
            </div>
            <span class="text-sm font-semibold tabular-nums shrink-0" :class="typeColor[r.type]">
              {{ r.type === 'income' ? '+' : r.type === 'expense' ? '-' : '' }}{{ r.amount.toFixed(2) }}
            </span>
            <div class="flex gap-0.5 shrink-0">
              <button @click="openEditRecord(r)" class="p-1 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"><Pencil class="w-3 h-3" /></button>
              <button @click="askDeleteRecord(r)" class="p-1 rounded hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors"><Trash2 class="w-3 h-3" /></button>
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

  <!-- 删除确认 -->
  <Teleport to="body">
    <div v-if="confirmDeleteId" class="fixed inset-0 z-[100] flex items-center justify-center bg-black/40">
      <div class="bg-card border border-border rounded-2xl p-6 w-80 shadow-xl">
        <p class="font-semibold mb-2">确认删除？</p>
        <p class="text-sm text-muted-foreground mb-6">
          <template v-if="confirmDeleteType === 'event'">
            删除事件后，其下流水记录不会删除，会变为「未归属」状态。
          </template>
          <template v-else>此操作不可撤销。</template>
        </p>
        <div class="flex gap-3">
          <button
            @click="confirmDeleteId = null"
            class="flex-1 py-2 rounded-xl border border-border text-sm font-medium hover:bg-accent transition-colors"
          >取消</button>
          <button
            @click="confirmDelete"
            class="flex-1 py-2 rounded-xl bg-destructive text-destructive-foreground text-sm font-medium hover:opacity-90 transition-opacity"
          >删除</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
