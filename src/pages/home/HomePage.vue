<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useRouter } from "vue-router";
import { useBookStore } from "@/stores/books";
import { useEntryStore } from "@/stores/entries";
import { useCategoryStore } from "@/stores/categories";
import { statsApi, type BookStats } from "@/api/views";
import { recordApi, type FlowRecord } from "@/api/records";
import {
  ChevronDown,
  TrendingUp,
  TrendingDown,
  Minus,
  ArrowRight,
  Check,
  Plus,
} from "lucide-vue-next";

const router = useRouter();
const bookStore = useBookStore();
const entryStore = useEntryStore();
const categoryStore = useCategoryStore();

const stats = ref<BookStats | null>(null);
const recentRecords = ref<FlowRecord[]>([]);
const loading = ref(false);
const showBookPicker = ref(false);

// ── 当前月范围 ─────────────────────────────────────────────────────────────
const now = new Date();
const monthFrom = computed(() => {
  const d = new Date(now.getFullYear(), now.getMonth(), 1);
  return d.toISOString().slice(0, 10);
});
const monthTo = computed(() => {
  const d = new Date(now.getFullYear(), now.getMonth() + 1, 0);
  return d.toISOString().slice(0, 10);
});

// ── 分类名称映射 ──────────────────────────────────────────────────────────
const categoryMap = computed(() => {
  const m: Record<string, string> = {};
  categoryStore.categories.forEach((c) => (m[c.id] = c.name));
  return m;
});

// ── 加载数据 ──────────────────────────────────────────────────────────────
async function load() {
  const bookId = bookStore.activeBookId;
  if (!bookId) return;
  loading.value = true;
  try {
    await Promise.all([
      entryStore.fetchEntries(bookId),
      categoryStore.fetchCategories(),
      statsApi.getBookStats(bookId, monthFrom.value, monthTo.value).then((s) => {
        stats.value = s;
      }),
      recordApi
        .list(bookId, { from: "2000-01-01", to: monthTo.value })
        .then((rs) => {
          recentRecords.value = [...rs]
            .sort((a, b) => b.happenedAt.localeCompare(a.happenedAt))
            .slice(0, 5);
        }),
    ]);
  } finally {
    loading.value = false;
  }
}

onMounted(load);
watch(() => bookStore.activeBookId, load);

// ── 格式化 ────────────────────────────────────────────────────────────────
function fmt(v: number) {
  return new Intl.NumberFormat("zh-CN", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(v);
}

function fmtDate(s: string) {
  const d = new Date(s);
  const today = new Date();
  const diff = Math.floor((today.getTime() - d.getTime()) / 86400000);
  if (diff === 0) return "今天";
  if (diff === 1) return "昨天";
  if (diff < 7) return `${diff}天前`;
  return `${d.getMonth() + 1}月${d.getDate()}日`;
}

function recordLabel(r: FlowRecord) {
  if (r.categoryId && categoryMap.value[r.categoryId]) return categoryMap.value[r.categoryId];
  if (r.name) return r.name;
  return r.type === "income" ? "收入" : r.type === "expense" ? "支出" : "转账";
}

const netIncome = computed(() => (stats.value ? stats.value.income - stats.value.expense : 0));

async function selectBook(id: string) {
  bookStore.setActiveBook(id);
  showBookPicker.value = false;
}
</script>

<template>
  <div class="min-h-full bg-background" style="padding-top: env(safe-area-inset-top)">

    <!-- ── 顶部账本选择器 ──────────────────────────────────────────────── -->
    <div class="sticky top-0 z-20 bg-background/95 backdrop-blur-xl px-4 pt-3 pb-2">
      <button
        @click="showBookPicker = !showBookPicker"
        class="flex items-center gap-1.5 px-3 py-2 rounded-xl bg-card border border-border text-sm font-semibold cursor-pointer hover:bg-accent transition-colors min-h-[44px]"
      >
        <span class="max-w-[180px] truncate">{{ bookStore.activeBook?.name ?? "选择账本" }}</span>
        <ChevronDown
          class="w-4 h-4 text-muted-foreground shrink-0 transition-transform duration-200"
          :class="showBookPicker ? 'rotate-180' : ''"
        />
      </button>
    </div>

    <!-- ── 账本下拉 Picker ─────────────────────────────────────────────── -->
    <Teleport to="body">
      <div
        v-if="showBookPicker"
        class="fixed inset-0 z-50 flex items-start justify-start"
        @click.self="showBookPicker = false"
      >
        <div
          class="absolute left-4 top-[calc(env(safe-area-inset-top)+58px)] bg-card border border-border rounded-2xl shadow-xl overflow-hidden min-w-[220px]"
        >
          <div
            v-for="book in bookStore.books"
            :key="book.id"
            @click="selectBook(book.id)"
            class="flex items-center gap-3 px-4 py-3.5 cursor-pointer hover:bg-accent transition-colors min-h-[44px]"
          >
            <Check
              class="w-4 h-4 text-primary shrink-0"
              :class="book.id === bookStore.activeBookId ? 'opacity-100' : 'opacity-0'"
            />
            <span class="text-sm font-medium truncate">{{ book.name }}</span>
          </div>
          <div class="border-t border-border">
            <button
              @click="showBookPicker = false; router.push('/books')"
              class="flex items-center gap-2 w-full px-4 py-3.5 text-sm text-muted-foreground hover:bg-accent transition-colors cursor-pointer min-h-[44px]"
            >
              <Plus class="w-4 h-4" />
              管理账本
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <div class="px-4 pb-8 space-y-4">

      <!-- ── 净资产卡片 ──────────────────────────────────────────────────── -->
      <div class="rounded-3xl bg-primary p-6 text-primary-foreground">
        <p class="text-sm font-medium opacity-80 mb-1">净资产</p>
        <p class="text-4xl font-bold tracking-tight mb-4">
          ¥{{ fmt(entryStore.netWorth) }}
        </p>
        <div class="flex gap-4">
          <div>
            <p class="text-xs opacity-70 mb-0.5">总资产</p>
            <p class="text-base font-semibold">¥{{ fmt(entryStore.totalAssets) }}</p>
          </div>
          <div class="w-px bg-primary-foreground/20" />
          <div>
            <p class="text-xs opacity-70 mb-0.5">总负债</p>
            <p class="text-base font-semibold">¥{{ fmt(entryStore.totalLiabilities) }}</p>
          </div>
        </div>
      </div>

      <!-- ── 本月概览 ────────────────────────────────────────────────────── -->
      <div>
        <p class="text-xs font-medium text-muted-foreground px-1 mb-2">本月概览</p>
        <div class="grid grid-cols-3 gap-2">
          <div class="bg-card border border-border rounded-2xl p-4">
            <div class="flex items-center gap-1 mb-2">
              <TrendingUp class="w-3.5 h-3.5 text-emerald-500" />
              <p class="text-xs text-muted-foreground">收入</p>
            </div>
            <p class="text-lg font-bold text-emerald-600 dark:text-emerald-400">
              ¥{{ fmt(stats?.income ?? 0) }}
            </p>
          </div>
          <div class="bg-card border border-border rounded-2xl p-4">
            <div class="flex items-center gap-1 mb-2">
              <TrendingDown class="w-3.5 h-3.5 text-rose-500" />
              <p class="text-xs text-muted-foreground">支出</p>
            </div>
            <p class="text-lg font-bold text-rose-600 dark:text-rose-400">
              ¥{{ fmt(stats?.expense ?? 0) }}
            </p>
          </div>
          <div class="bg-card border border-border rounded-2xl p-4">
            <div class="flex items-center gap-1 mb-2">
              <Minus class="w-3.5 h-3.5 text-muted-foreground" />
              <p class="text-xs text-muted-foreground">净收入</p>
            </div>
            <p
              class="text-lg font-bold"
              :class="netIncome >= 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'"
            >
              {{ netIncome >= 0 ? "+" : "" }}¥{{ fmt(Math.abs(netIncome)) }}
            </p>
          </div>
        </div>
      </div>

      <!-- ── 最近记录 ────────────────────────────────────────────────────── -->
      <div>
        <div class="flex items-center justify-between px-1 mb-2">
          <p class="text-xs font-medium text-muted-foreground">最近记录</p>
          <button
            @click="router.push('/records')"
            class="flex items-center gap-0.5 text-xs text-primary cursor-pointer min-h-[44px] pr-1"
          >
            查看全部 <ArrowRight class="w-3.5 h-3.5" />
          </button>
        </div>

        <!-- 骨架屏 -->
        <div v-if="loading" class="bg-card border border-border rounded-2xl overflow-hidden">
          <div v-for="i in 3" :key="i" class="flex items-center gap-3 px-4 py-4 border-b border-border/60 last:border-0">
            <div class="w-9 h-9 rounded-xl bg-muted animate-pulse shrink-0" />
            <div class="flex-1 space-y-2">
              <div class="h-3.5 bg-muted rounded animate-pulse w-24" />
              <div class="h-3 bg-muted rounded animate-pulse w-16" />
            </div>
            <div class="h-4 bg-muted rounded animate-pulse w-16" />
          </div>
        </div>

        <div v-else-if="recentRecords.length === 0" class="bg-card border border-border rounded-2xl p-8 text-center">
          <p class="text-sm text-muted-foreground">暂无记录</p>
        </div>

        <div v-else class="bg-card border border-border rounded-2xl overflow-hidden">
          <div
            v-for="(r, i) in recentRecords"
            :key="r.id"
            class="flex items-center gap-3 px-4 py-3.5 cursor-pointer hover:bg-accent/40 active:bg-accent/60 transition-colors min-h-[64px]"
            :class="i < recentRecords.length - 1 ? 'border-b border-border/60' : ''"
            @click="router.push('/records')"
          >
            <!-- 类型指示点 -->
            <div
              class="w-9 h-9 rounded-xl shrink-0 flex items-center justify-center"
              :class="{
                'bg-emerald-100 dark:bg-emerald-900/40': r.type === 'income',
                'bg-rose-100 dark:bg-rose-900/40': r.type === 'expense',
                'bg-blue-100 dark:bg-blue-900/40': r.type === 'transfer',
              }"
            >
              <TrendingUp v-if="r.type === 'income'" class="w-4 h-4 text-emerald-600 dark:text-emerald-400" />
              <TrendingDown v-else-if="r.type === 'expense'" class="w-4 h-4 text-rose-600 dark:text-rose-400" />
              <Minus v-else class="w-4 h-4 text-blue-600 dark:text-blue-400" />
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium truncate">{{ recordLabel(r) }}</p>
              <p class="text-xs text-muted-foreground mt-0.5">{{ fmtDate(r.happenedAt) }}</p>
            </div>
            <p
              class="text-sm font-semibold shrink-0"
              :class="{
                'text-emerald-600 dark:text-emerald-400': r.type === 'income',
                'text-rose-600 dark:text-rose-400': r.type === 'expense',
                'text-muted-foreground': r.type === 'transfer',
              }"
            >
              {{ r.type === "income" ? "+" : r.type === "expense" ? "-" : "" }}¥{{ fmt(r.amount) }}
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
