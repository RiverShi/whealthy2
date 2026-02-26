<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import {
  TrendingUp, TrendingDown, BarChart3, Camera,
  Clock, RefreshCw, ChevronRight, ChevronDown, Plus,
  CheckCircle2, Circle, Scale,
} from "lucide-vue-next";
import { useBookStore } from "@/stores/books";
import {
  statsApi, snapshotApi,
  type BookStats, type Snapshot, type SnapshotEntryData,
  type SnapshotTask, type SnapshotDiffEntry, type SnapshotFrequency
} from "@/api/views";

const bookStore = useBookStore();
const bookId = computed(() => bookStore.activeBookId ?? "");

// ─── 时间范围 ──────────────────────────────────────────────────────────────────
type Range = "thisMonth" | "lastMonth" | "thisYear";
const selectedRange = ref<Range>("thisMonth");

function getDateRange(r: Range): { from: string; to: string } {
  const now = new Date();
  const y = now.getFullYear();
  const m = now.getMonth(); // 0-based
  if (r === "thisMonth") {
    const from = `${y}-${String(m + 1).padStart(2, "0")}-01`;
    const lastDay = new Date(y, m + 1, 0).getDate();
    const to = `${y}-${String(m + 1).padStart(2, "0")}-${lastDay}`;
    return { from, to };
  } else if (r === "lastMonth") {
    const lm = m === 0 ? 12 : m;
    const ly = m === 0 ? y - 1 : y;
    const from = `${ly}-${String(lm).padStart(2, "0")}-01`;
    const lastDay = new Date(ly, lm, 0).getDate();
    const to = `${ly}-${String(lm).padStart(2, "0")}-${lastDay}`;
    return { from, to };
  } else {
    return { from: `${y}-01-01`, to: `${y}-12-31` };
  }
}

const rangeLabels: Record<Range, string> = {
  thisMonth: "本月",
  lastMonth: "上月",
  thisYear: "本年",
};

// ─── 统计数据 ──────────────────────────────────────────────────────────────────
const stats = ref<BookStats | null>(null);
const statsLoading = ref(false);
const statsError = ref("");

async function loadStats() {
  if (!bookId.value) return;
  statsLoading.value = true;
  statsError.value = "";
  try {
    const { from, to } = getDateRange(selectedRange.value);
    stats.value = await statsApi.getBookStats(bookId.value, from, to);
  } catch (e: unknown) {
    statsError.value = String(e);
  } finally {
    statsLoading.value = false;
  }
}

const netWorthSign = computed(() => {
  if (!stats.value) return "neutral";
  return stats.value.netWorth >= 0 ? "positive" : "negative";
});

const maxExpense = computed(() =>
  Math.max(...(stats.value?.expenseByCategory.map((c) => c.amount) ?? [0]), 1)
);
const maxIncome = computed(() =>
  Math.max(...(stats.value?.incomeByCategory.map((c) => c.amount) ?? [0]), 1)
);

// ─── 快照数据 ──────────────────────────────────────────────────────────────────
const snapshots = ref<Snapshot[]>([]);
const snapshotsLoading = ref(false);
const snapshotTask = ref<SnapshotTask | null>(null);
const creatingSnapshot = ref(false);

async function loadSnapshots() {
  if (!bookId.value) return;
  snapshotsLoading.value = true;
  try {
    snapshots.value = await snapshotApi.list(bookId.value);
    snapshotTask.value = await snapshotApi.getTaskForBook(bookId.value);
    if (snapshotTask.value) {
      taskFrequency.value = snapshotTask.value.frequency;
    }
  } catch {
    /* ignore */
  } finally {
    snapshotsLoading.value = false;
  }
}

async function handleCreateSnapshot() {
  if (!bookId.value) return;
  creatingSnapshot.value = true;
  try {
    const snap = await snapshotApi.create(bookId.value);
    snapshots.value.push(snap);
  } finally {
    creatingSnapshot.value = false;
  }
}

// ─── 净值迷你折线图（SVG）─────────────────────────────────────────────────────
const chartData = computed(() => {
  if (snapshots.value.length < 2) return null;
  const values = snapshots.value.map((s) => s.netWorth);
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = max - min || 1;
  const W = 300;
  const H = 70;
  const pts = values.map((v, i) => {
    const x = (i / (values.length - 1)) * W;
    const y = H - ((v - min) / range) * (H * 0.8) - H * 0.1;
    return `${x.toFixed(1)},${y.toFixed(1)}`;
  });
  return { polyline: pts.join(" "), pts };
});

// ─── 快照任务配置 ──────────────────────────────────────────────────────────────
const showTaskConfig = ref(false);
const taskFrequency = ref<SnapshotFrequency>("monthly");
const savingTask = ref(false);

async function saveSnapshotTask() {
  if (!bookId.value) return;
  savingTask.value = true;
  try {
    if (snapshotTask.value) {
      snapshotTask.value = await snapshotApi.updateTask(snapshotTask.value.id, {
        frequency: taskFrequency.value,
        isActive: true,
      });
    } else {
      snapshotTask.value = await snapshotApi.createTask(bookId.value, taskFrequency.value);
    }
    showTaskConfig.value = false;
  } finally {
    savingTask.value = false;
  }
}

async function toggleTaskActive() {
  if (!snapshotTask.value) return;
  snapshotTask.value = await snapshotApi.updateTask(snapshotTask.value.id, {
    isActive: !snapshotTask.value.isActive,
  });
}

// ─── 快照详情展开 ──────────────────────────────────────────────────────────────
const expandedSnapshotId = ref<string | null>(null);

function toggleSnapshotDetail(id: string) {
  expandedSnapshotId.value = expandedSnapshotId.value === id ? null : id;
}

// 按 kind+category 分组条目，返回结构体
function groupEntries(entries: SnapshotEntryData[]) {
  const assets: Record<string, { categoryName: string | null; entries: SnapshotEntryData[] }> = {};
  const liabilities: Record<string, { categoryName: string | null; entries: SnapshotEntryData[] }> = {};

  for (const e of entries) {
    const key = e.categoryL1Id ?? "__none__";
    if (e.kind === "asset") {
      if (!assets[key]) assets[key] = { categoryName: e.categoryName, entries: [] };
      assets[key].entries.push(e);
    } else {
      if (!liabilities[key]) liabilities[key] = { categoryName: e.categoryName, entries: [] };
      liabilities[key].entries.push(e);
    }
  }
  return {
    assetGroups: Object.values(assets),
    liabilityGroups: Object.values(liabilities),
  };
}

// ─── 快照对比 Diff ─────────────────────────────────────────────────────────────
const diffFromId = ref<string>("");
const diffToId = ref<string>("");
const diffResult = ref<SnapshotDiffEntry[]>([]);
const diffLoading = ref(false);
const showDiff = ref(false);

async function loadDiff() {
  if (!diffFromId.value || !diffToId.value) return;
  diffLoading.value = true;
  showDiff.value = false;
  try {
    diffResult.value = await snapshotApi.diff(diffFromId.value, diffToId.value);
    showDiff.value = true;
  } finally {
    diffLoading.value = false;
  }
}

// ─── 生命周期 ──────────────────────────────────────────────────────────────────
function refresh() {
  loadStats();
  loadSnapshots();
}

watch(bookId, (id) => { if (id) refresh(); });
watch(selectedRange, () => loadStats());

onMounted(() => {
  if (bookId.value) refresh();
});

function fmt(n: number) {
  return n.toLocaleString("zh-CN", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}
</script>

<template>
  <div class="min-h-full bg-background">
    <!-- ── 顶栏 ────────────────────────────────────────────────────────────── -->
    <div class="sticky top-0 z-20 bg-card/95 backdrop-blur-xl border-b border-border">
      <div class="px-4 py-3">
        <div class="flex items-center justify-between mb-2.5">
          <div>
            <p class="text-[11px] text-muted-foreground leading-none mb-0.5">{{ bookStore.activeBook?.name }}</p>
            <h1 class="text-xl font-bold leading-tight">统计</h1>
          </div>
          <button
            @click="refresh"
            class="p-2 rounded-xl hover:bg-accent transition-colors text-muted-foreground cursor-pointer"
            :class="{ 'animate-spin text-primary': statsLoading }"
          >
            <RefreshCw class="w-5 h-5" />
          </button>
        </div>
        <!-- 时间范围选择器 -->
        <div class="flex gap-1.5">
          <button
            v-for="(label, key) in rangeLabels"
            :key="key"
            @click="selectedRange = key as Range"
            class="flex-1 py-1.5 text-xs rounded-xl font-medium transition-colors cursor-pointer"
            :class="selectedRange === key
              ? 'bg-primary text-primary-foreground'
              : 'bg-muted text-muted-foreground hover:bg-accent'"
          >
            {{ label }}
          </button>
        </div>
      </div>
    </div>

    <!-- ── 无账本提示 ──────────────────────────────────────────────────────── -->
    <div v-if="!bookId" class="flex items-center justify-center py-24 text-muted-foreground">
      <div class="text-center">
        <BarChart3 class="w-12 h-12 mx-auto mb-3 opacity-30" />
        <p class="text-sm">请先选择账本</p>
      </div>
    </div>

    <div v-else class="px-4 py-3 space-y-4 pb-8">

      <!-- ── 净资产大卡 ─────────────────────────────────────────────────────── -->
      <div class="rounded-2xl border border-border bg-card p-6">
        <p class="text-xs text-muted-foreground mb-1">净资产（当前）</p>
        <div v-if="statsLoading" class="space-y-2">
          <div class="h-10 bg-muted rounded-lg animate-pulse w-40"></div>
          <div class="h-4 bg-muted rounded animate-pulse w-64"></div>
        </div>
        <template v-else-if="stats">
          <p
            class="text-4xl font-bold tabular-nums tracking-tight"
            :class="netWorthSign === 'positive' ? 'text-emerald-500' : netWorthSign === 'negative' ? 'text-red-500' : 'text-foreground'"
          >
            ¥{{ fmt(stats.netWorth) }}
          </p>
          <div class="flex items-center gap-4 mt-3 text-sm">
            <div>
              <p class="text-xs text-muted-foreground">总资产</p>
              <p class="font-semibold text-emerald-600">¥{{ fmt(stats.totalAssets) }}</p>
            </div>
            <div class="h-8 w-px bg-border"></div>
            <div>
              <p class="text-xs text-muted-foreground">总负债</p>
              <p class="font-semibold text-red-500">¥{{ fmt(stats.totalLiabilities) }}</p>
            </div>
          </div>
        </template>
        <p v-else-if="statsError" class="text-xs text-destructive mt-2">{{ statsError }}</p>
      </div>

      <!-- ── 收支汇总卡 ─────────────────────────────────────────────────────── -->
      <div class="grid grid-cols-2 gap-3">
        <div class="rounded-2xl border border-border bg-card p-4">
          <div class="flex items-center gap-2 mb-2">
            <div class="w-7 h-7 rounded-lg bg-emerald-500/10 flex items-center justify-center">
              <TrendingUp class="w-4 h-4 text-emerald-500" />
            </div>
            <span class="text-xs text-muted-foreground">{{ rangeLabels[selectedRange] }}收入</span>
          </div>
          <div v-if="statsLoading" class="h-7 bg-muted rounded animate-pulse w-20 mt-1"></div>
          <p v-else class="text-xl font-bold text-emerald-500 tabular-nums">
            ¥{{ fmt(stats?.income ?? 0) }}
          </p>
        </div>
        <div class="rounded-2xl border border-border bg-card p-4">
          <div class="flex items-center gap-2 mb-2">
            <div class="w-7 h-7 rounded-lg bg-red-500/10 flex items-center justify-center">
              <TrendingDown class="w-4 h-4 text-red-500" />
            </div>
            <span class="text-xs text-muted-foreground">{{ rangeLabels[selectedRange] }}支出</span>
          </div>
          <div v-if="statsLoading" class="h-7 bg-muted rounded animate-pulse w-20 mt-1"></div>
          <p v-else class="text-xl font-bold text-red-500 tabular-nums">
            ¥{{ fmt(stats?.expense ?? 0) }}
          </p>
        </div>
      </div>

      <!-- ── 收支结余行 ─────────────────────────────────────────────────────── -->
      <div v-if="stats && (stats.income > 0 || stats.expense > 0)"
           class="flex items-center justify-between px-4 py-2.5 rounded-xl bg-muted/40 text-sm">
        <span class="text-muted-foreground text-xs">{{ rangeLabels[selectedRange] }}结余</span>
        <span
          class="font-semibold tabular-nums"
          :class="stats.income - stats.expense >= 0 ? 'text-emerald-600' : 'text-red-500'"
        >
          {{ stats.income - stats.expense >= 0 ? "+" : "" }}¥{{ fmt(stats.income - stats.expense) }}
        </span>
      </div>

      <!-- ── 支出分类 ───────────────────────────────────────────────────────── -->
      <div
        v-if="stats && stats.expenseByCategory.length > 0"
        class="rounded-2xl border border-border bg-card p-5"
      >
        <p class="text-sm font-semibold mb-4">支出构成</p>
        <div class="space-y-3">
          <div
            v-for="cat in stats.expenseByCategory"
            :key="cat.categoryId ?? '__none__'"
          >
            <div class="flex items-center justify-between text-xs mb-1">
              <span class="font-medium">{{ cat.categoryName ?? "未分类" }}</span>
              <div class="flex items-center gap-2">
                <span class="text-muted-foreground">
                  {{ ((cat.amount / (stats?.expense || 1)) * 100).toFixed(0) }}%
                </span>
                <span class="tabular-nums text-red-500">¥{{ fmt(cat.amount) }}</span>
              </div>
            </div>
            <div class="h-2 bg-muted rounded-full overflow-hidden">
              <div
                class="h-full bg-red-400 rounded-full transition-all duration-500"
                :style="{ width: `${(cat.amount / maxExpense) * 100}%` }"
              ></div>
            </div>
          </div>
        </div>
      </div>

      <!-- ── 收入分类 ───────────────────────────────────────────────────────── -->
      <div
        v-if="stats && stats.incomeByCategory.length > 0"
        class="rounded-2xl border border-border bg-card p-5"
      >
        <p class="text-sm font-semibold mb-4">收入构成</p>
        <div class="space-y-3">
          <div
            v-for="cat in stats.incomeByCategory"
            :key="cat.categoryId ?? '__none__'"
          >
            <div class="flex items-center justify-between text-xs mb-1">
              <span class="font-medium">{{ cat.categoryName ?? "未分类" }}</span>
              <div class="flex items-center gap-2">
                <span class="text-muted-foreground">
                  {{ ((cat.amount / (stats?.income || 1)) * 100).toFixed(0) }}%
                </span>
                <span class="tabular-nums text-emerald-600">¥{{ fmt(cat.amount) }}</span>
              </div>
            </div>
            <div class="h-2 bg-muted rounded-full overflow-hidden">
              <div
                class="h-full bg-emerald-400 rounded-full transition-all duration-500"
                :style="{ width: `${(cat.amount / maxIncome) * 100}%` }"
              ></div>
            </div>
          </div>
        </div>
      </div>

      <!-- ── 净值快照区块 ────────────────────────────────────────────────────── -->
      <div class="rounded-2xl border border-border bg-card overflow-hidden">
        <!-- 头部栏 -->
        <div class="flex items-center justify-between px-5 py-4 border-b border-border">
          <div class="flex items-center gap-2">
            <Camera class="w-4 h-4 text-muted-foreground" />
            <span class="text-sm font-semibold">净值快照</span>
            <span class="text-xs text-muted-foreground">({{ snapshots.length }})</span>
          </div>
          <div class="flex items-center gap-2">
            <button
              @click="showTaskConfig = !showTaskConfig"
              class="p-1.5 rounded-lg hover:bg-accent transition-colors"
              :class="snapshotTask?.isActive ? 'text-primary' : 'text-muted-foreground'"
              title="自动快照设置"
            >
              <Clock class="w-3.5 h-3.5" />
            </button>
            <button
              @click="handleCreateSnapshot"
              :disabled="creatingSnapshot"
              class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50 transition-colors"
            >
              <Plus class="w-3 h-3" />
              立即快照
            </button>
          </div>
        </div>

        <!-- 自动快照任务配置 -->
        <div v-if="showTaskConfig" class="px-5 py-4 border-b border-border bg-muted/20">
          <div class="flex items-center justify-between mb-2">
            <p class="text-xs font-semibold">自动快照配置</p>
            <button
              v-if="snapshotTask"
              @click="toggleTaskActive"
              class="flex items-center gap-1 text-xs"
              :class="snapshotTask.isActive ? 'text-primary' : 'text-muted-foreground'"
            >
              <CheckCircle2 v-if="snapshotTask.isActive" class="w-3.5 h-3.5" />
              <Circle v-else class="w-3.5 h-3.5" />
              {{ snapshotTask.isActive ? "已启用" : "已停用" }}
            </button>
          </div>
          <div class="flex items-center gap-2">
            <select
              v-model="taskFrequency"
              class="flex-1 text-xs px-3 py-1.5 rounded-lg border border-border bg-background"
            >
              <option value="daily">每天</option>
              <option value="weekly">每周</option>
              <option value="monthly">每月</option>
            </select>
            <button
              @click="saveSnapshotTask"
              :disabled="savingTask"
              class="px-3 py-1.5 text-xs font-medium rounded-lg bg-primary text-primary-foreground disabled:opacity-50"
            >
              {{ snapshotTask ? "更新" : "启用" }}
            </button>
          </div>
          <p v-if="snapshotTask?.lastRunAt" class="text-xs text-muted-foreground mt-2">
            上次自动运行：{{ snapshotTask.lastRunAt }}
          </p>
        </div>

        <!-- 趋势折线图（SVG） -->
        <div v-if="chartData" class="px-5 pt-4 pb-2 border-b border-border">
          <p class="text-xs text-muted-foreground mb-3">净值走势</p>
          <svg viewBox="0 0 300 70" class="w-full h-16 overflow-visible" preserveAspectRatio="none">
            <defs>
              <linearGradient id="lineGrad" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="rgb(16,185,129)" stop-opacity="0.3" />
                <stop offset="100%" stop-color="rgb(16,185,129)" stop-opacity="0.02" />
              </linearGradient>
            </defs>
            <polyline
              :points="chartData.polyline"
              fill="none"
              stroke="rgb(16,185,129)"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
            <circle
              v-for="(pt, i) in chartData.pts"
              :key="i"
              :cx="pt.split(',')[0]"
              :cy="pt.split(',')[1]"
              r="2.5"
              fill="rgb(16,185,129)"
            />
          </svg>
          <div class="flex justify-between text-xs text-muted-foreground mt-1 pb-2">
            <span>{{ snapshots[0]?.createdAt }}</span>
            <span>{{ snapshots[snapshots.length - 1]?.createdAt }}</span>
          </div>
        </div>

        <!-- 快照对比 -->
        <div v-if="snapshots.length >= 2" class="px-5 py-4 border-b border-border">
          <p class="text-xs font-semibold mb-2 text-muted-foreground">快照对比</p>
          <div class="flex items-center gap-2">
            <select
              v-model="diffFromId"
              class="flex-1 text-xs px-2 py-1.5 rounded-lg border border-border bg-background"
            >
              <option value="">起始快照</option>
              <option v-for="s in snapshots" :key="s.id" :value="s.id">
                {{ s.createdAt }}（¥{{ fmt(s.netWorth) }}）
              </option>
            </select>
            <ChevronRight class="w-3 h-3 text-muted-foreground shrink-0" />
            <select
              v-model="diffToId"
              class="flex-1 text-xs px-2 py-1.5 rounded-lg border border-border bg-background"
            >
              <option value="">对比快照</option>
              <option v-for="s in snapshots" :key="s.id" :value="s.id">
                {{ s.createdAt }}（¥{{ fmt(s.netWorth) }}）
              </option>
            </select>
            <button
              @click="loadDiff"
              :disabled="!diffFromId || !diffToId || diffLoading"
              class="px-3 py-1.5 text-xs font-medium rounded-lg bg-secondary text-secondary-foreground hover:bg-secondary/80 disabled:opacity-40"
            >
              对比
            </button>
          </div>

          <!-- Diff 结果 -->
          <div v-if="showDiff" class="mt-3">
            <div v-if="diffResult.length === 0" class="text-xs text-muted-foreground text-center py-2">
              两份快照无差异
            </div>
            <div v-else class="space-y-1.5">
              <div
                v-for="d in diffResult"
                :key="d.entryId"
                class="flex items-center justify-between text-xs py-2 px-3 rounded-xl bg-muted/40"
              >
                <div class="flex items-center gap-2 min-w-0">
                  <div
                    class="w-1.5 h-1.5 rounded-full shrink-0"
                    :class="d.kind === 'asset' ? 'bg-emerald-500' : 'bg-red-400'"
                  ></div>
                  <span class="font-medium truncate">{{ d.entryName }}</span>
                </div>
                <div class="flex items-center gap-2 shrink-0 ml-3">
                  <span class="text-muted-foreground">
                    {{ d.oldValue != null ? `¥${fmt(d.oldValue)}` : "新增" }}
                    →
                    {{ d.newValue != null ? `¥${fmt(d.newValue)}` : "删除" }}
                  </span>
                  <span
                    class="font-semibold tabular-nums"
                    :class="d.change > 0 ? 'text-emerald-500' : d.change < 0 ? 'text-red-500' : 'text-muted-foreground'"
                  >
                    {{ d.change > 0 ? "+" : "" }}¥{{ fmt(d.change) }}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 快照列表 -->
        <div v-if="snapshotsLoading" class="p-5 space-y-2">
          <div v-for="i in 3" :key="i" class="h-10 bg-muted rounded-xl animate-pulse"></div>
        </div>
        <div v-else-if="snapshots.length === 0" class="flex flex-col items-center justify-center py-10 text-muted-foreground">
          <Camera class="w-8 h-8 mb-2 opacity-30" />
          <p class="text-sm">暂无快照</p>
          <p class="text-xs mt-1 opacity-60">点击「立即快照」记录当前净值</p>
        </div>
        <div v-else class="divide-y divide-border">
          <div v-for="snap in [...snapshots].reverse()" :key="snap.id">
            <!-- 快照摘要行（可点击展开） -->
            <button
              class="w-full flex items-center justify-between px-5 py-3.5 hover:bg-muted/20 transition-colors text-left"
              @click="toggleSnapshotDetail(snap.id)"
            >
              <div class="flex items-center gap-3">
                <div
                  class="w-2 h-2 rounded-full shrink-0"
                  :class="snap.source === 'manual' ? 'bg-primary' : 'bg-muted-foreground/30'"
                  :title="snap.source === 'manual' ? '手动快照' : '自动快照'"
                ></div>
                <div class="text-left">
                  <p class="text-sm font-bold tabular-nums">¥{{ fmt(snap.netWorth) }}</p>
                  <p class="text-xs text-muted-foreground">
                    {{ snap.createdAt }}
                    <span class="ml-1 opacity-50">{{ snap.source === "manual" ? "手动" : "自动" }}</span>
                    <span class="ml-1.5 opacity-50">· {{ snap.data.entries.length }} 项</span>
                  </p>
                </div>
              </div>
              <div class="flex items-center gap-3">
                <div class="flex flex-col items-end gap-0.5 text-xs">
                  <span class="text-emerald-600">资 ¥{{ fmt(snap.totalAssets) }}</span>
                  <span class="text-red-500">负 ¥{{ fmt(snap.totalLiabilities) }}</span>
                </div>
                <ChevronDown
                  class="w-3.5 h-3.5 text-muted-foreground transition-transform shrink-0"
                  :class="expandedSnapshotId === snap.id ? 'rotate-180' : ''"
                />
              </div>
            </button>

            <!-- 展开：完整资产负债明细 -->
            <div
              v-if="expandedSnapshotId === snap.id"
              class="bg-muted/10 border-t border-dashed border-border"
            >
              <div class="px-5 py-4 space-y-4">
                <!-- 净值汇总条 -->
                <div class="flex items-center justify-between text-xs py-2.5 px-4 rounded-xl bg-card border border-border">
                  <div class="flex items-center gap-2">
                    <Scale class="w-3.5 h-3.5 text-muted-foreground" />
                    <span class="text-muted-foreground">净资产</span>
                  </div>
                  <span
                    class="font-bold text-base tabular-nums"
                    :class="snap.netWorth >= 0 ? 'text-emerald-600' : 'text-red-500'"
                  >
                    ¥{{ fmt(snap.netWorth) }}
                  </span>
                </div>

                <!-- 资产部分 -->
                <div v-if="groupEntries(snap.data.entries).assetGroups.length > 0">
                  <div class="flex items-center justify-between mb-2">
                    <p class="text-xs font-semibold text-emerald-600">资产</p>
                    <p class="text-xs font-semibold text-emerald-600 tabular-nums">¥{{ fmt(snap.totalAssets) }}</p>
                  </div>
                  <div class="space-y-2">
                    <div
                      v-for="(group, gi) in groupEntries(snap.data.entries).assetGroups"
                      :key="'ag-' + gi"
                    >
                      <!-- 分类标题 -->
                      <p class="text-xs text-muted-foreground mb-1 pl-1">
                        {{ group.categoryName ?? "未分类" }}
                      </p>
                      <!-- 条目列表 -->
                      <div class="space-y-1">
                        <div
                          v-for="entry in group.entries"
                          :key="entry.id"
                          class="flex items-center justify-between text-xs py-1.5 px-3 rounded-lg bg-card hover:bg-accent/50 transition-colors"
                        >
                          <div class="flex items-center gap-2">
                            <div class="w-1 h-3 rounded-full bg-emerald-400 shrink-0"></div>
                            <span class="font-medium">{{ entry.name }}</span>
                          </div>
                          <span class="tabular-nums text-emerald-700 font-semibold">¥{{ fmt(entry.value) }}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- 分隔线 -->
                <div
                  v-if="groupEntries(snap.data.entries).assetGroups.length > 0 && groupEntries(snap.data.entries).liabilityGroups.length > 0"
                  class="border-t border-dashed border-border"
                ></div>

                <!-- 负债部分 -->
                <div v-if="groupEntries(snap.data.entries).liabilityGroups.length > 0">
                  <div class="flex items-center justify-between mb-2">
                    <p class="text-xs font-semibold text-red-500">负债</p>
                    <p class="text-xs font-semibold text-red-500 tabular-nums">¥{{ fmt(snap.totalLiabilities) }}</p>
                  </div>
                  <div class="space-y-2">
                    <div
                      v-for="(group, gi) in groupEntries(snap.data.entries).liabilityGroups"
                      :key="'lg-' + gi"
                    >
                      <p class="text-xs text-muted-foreground mb-1 pl-1">
                        {{ group.categoryName ?? "未分类" }}
                      </p>
                      <div class="space-y-1">
                        <div
                          v-for="entry in group.entries"
                          :key="entry.id"
                          class="flex items-center justify-between text-xs py-1.5 px-3 rounded-lg bg-card hover:bg-accent/50 transition-colors"
                        >
                          <div class="flex items-center gap-2">
                            <div class="w-1 h-3 rounded-full bg-red-400 shrink-0"></div>
                            <span class="font-medium">{{ entry.name }}</span>
                          </div>
                          <span class="tabular-nums text-red-600 font-semibold">¥{{ fmt(entry.value) }}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- 无条目 -->
                <div
                  v-if="snap.data.entries.length === 0"
                  class="text-center py-4 text-xs text-muted-foreground"
                >
                  该快照无条目记录
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="h-4"></div>
    </div>
  </div>
</template>
