<script setup lang="ts">
import { onMounted, ref, computed, watch } from "vue";
import { useEntryStore } from "@/stores/entries";
import { useBookStore } from "@/stores/books";
import { 
  TrendingUp, 
  TrendingDown, 
  Plus, 
  Edit2, 
  DollarSign,
  PieChart,
  Wallet,
  Activity
} from "lucide-vue-next";
import EntryForm from "@/components/EntryForm.vue";
import AdjustValueSheet from "@/components/AdjustValueSheet.vue";
import type { Entry, EntryKind } from "@/api/entries";

const entryStore = useEntryStore();
const bookStore = useBookStore();

const bookId = computed(() => bookStore.activeBookId ?? "");
const book = computed(() => bookStore.activeBook);

const showCreateForm = ref(false);
const createKind = ref<EntryKind>("asset");
const editingEntry = ref<Entry | undefined>();
const adjustingEntry = ref<Entry | undefined>();
const activeTab = ref<'all' | 'assets' | 'liabilities'>('all');

onMounted(() => {
  if (bookId.value) entryStore.fetchEntries(bookId.value);
});

watch(bookId, (newId) => {
  if (newId) entryStore.fetchEntries(newId);
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

function handleFormSuccess() {
  entryStore.fetchEntries(bookId.value);
}

const filteredAssets = computed(() => {
  if (activeTab.value === 'all' || activeTab.value === 'assets') {
    return entryStore.assets;
  }
  return [];
});

const filteredLiabilities = computed(() => {
  if (activeTab.value === 'all' || activeTab.value === 'liabilities') {
    return entryStore.liabilities;
  }
  return [];
});
</script>

<template>
  <div class="min-h-screen bg-gradient-to-br from-background via-background to-accent/5">
    <!-- 页头 -->
    <div class="border-b border-border/50 bg-card/50 backdrop-blur-sm sticky top-0 z-10">
      <div class="max-w-7xl mx-auto px-8 py-6">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-4">
            <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-primary to-primary/80 flex items-center justify-center shadow-md">
              <Wallet class="w-6 h-6 text-primary-foreground" />
            </div>
            <div>
              <p class="text-xs text-muted-foreground mb-0.5">{{ book?.name }}</p>
              <h1 class="text-2xl font-bold">资产</h1>
            </div>
          </div>
          
          <div class="flex gap-3">
            <button
              @click="handleCreateClick('liability')"
              class="flex items-center gap-2 px-4 py-2.5 rounded-xl border border-destructive/20 bg-destructive/5 text-destructive text-sm font-medium hover:bg-destructive/10 transition-smooth"
            >
              <Plus class="w-4 h-4" />
              新增负债
            </button>
            <button
              @click="handleCreateClick('asset')"
              class="flex items-center gap-2 px-5 py-2.5 bg-primary text-primary-foreground rounded-xl text-sm font-medium hover:shadow-lg hover:scale-105 transition-smooth shadow-md"
            >
              <Plus class="w-4 h-4" />
              新增资产
            </button>
          </div>
        </div>
      </div>
    </div>

    <div class="max-w-7xl mx-auto px-8 py-8">
      <!-- 净资产概览 -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8 animate-in">
        <div class="relative overflow-hidden rounded-2xl border border-border bg-gradient-to-br from-success/10 via-card to-card p-6 shadow-smooth">
          <div class="absolute top-0 right-0 w-32 h-32 bg-success/5 rounded-full -mr-16 -mt-16"></div>
          <div class="relative">
            <div class="flex items-center justify-between mb-3">
              <p class="text-sm font-medium text-muted-foreground">总资产</p>
              <div class="w-10 h-10 rounded-xl bg-success/10 flex items-center justify-center">
                <TrendingUp class="w-5 h-5 text-success" />
              </div>
            </div>
            <p class="text-3xl font-bold text-success mb-1">¥ {{ fmt(entryStore.totalAssets) }}</p>
            <p class="text-xs text-muted-foreground">{{ entryStore.assets.length }} 项资产</p>
          </div>
        </div>

        <div class="relative overflow-hidden rounded-2xl border border-border bg-gradient-to-br from-destructive/10 via-card to-card p-6 shadow-smooth">
          <div class="absolute top-0 right-0 w-32 h-32 bg-destructive/5 rounded-full -mr-16 -mt-16"></div>
          <div class="relative">
            <div class="flex items-center justify-between mb-3">
              <p class="text-sm font-medium text-muted-foreground">总负债</p>
              <div class="w-10 h-10 rounded-xl bg-destructive/10 flex items-center justify-center">
                <TrendingDown class="w-5 h-5 text-destructive" />
              </div>
            </div>
            <p class="text-3xl font-bold text-destructive mb-1">¥ {{ fmt(entryStore.totalLiabilities) }}</p>
            <p class="text-xs text-muted-foreground">{{ entryStore.liabilities.length }} 项负债</p>
          </div>
        </div>

        <div class="relative overflow-hidden rounded-2xl border border-primary/20 bg-gradient-to-br from-primary via-primary to-primary/90 p-6 shadow-smooth-lg">
          <div class="absolute top-0 right-0 w-32 h-32 bg-white/10 rounded-full -mr-16 -mt-16"></div>
          <div class="relative">
            <div class="flex items-center justify-between mb-3">
              <p class="text-sm font-medium text-primary-foreground/80">净资产</p>
              <div class="w-10 h-10 rounded-xl bg-white/10 flex items-center justify-center">
                <PieChart class="w-5 h-5 text-primary-foreground" />
              </div>
            </div>
            <p class="text-3xl font-bold text-primary-foreground mb-1">¥ {{ fmt(entryStore.netWorth) }}</p>
            <p class="text-xs text-primary-foreground/70">总资产 - 总负债</p>
          </div>
        </div>
      </div>

      <!-- 加载中 -->
      <div v-if="entryStore.loading" class="flex flex-col items-center justify-center py-24">
        <div class="w-12 h-12 rounded-xl bg-primary/10 flex items-center justify-center mb-4 animate-pulse">
          <Activity class="w-6 h-6 text-primary" />
        </div>
        <p class="text-muted-foreground">加载中…</p>
      </div>

      <div v-else class="space-y-8 animate-in">
        <!-- 标签页切换 -->
        <div class="flex gap-2 bg-muted/30 p-1.5 rounded-xl w-fit">
          <button
            @click="activeTab = 'all'"
            class="px-4 py-2 rounded-lg text-sm font-medium transition-smooth"
            :class="activeTab === 'all' 
              ? 'bg-card shadow-md text-foreground' 
              : 'text-muted-foreground hover:text-foreground'"
          >
            全部
          </button>
          <button
            @click="activeTab = 'assets'"
            class="px-4 py-2 rounded-lg text-sm font-medium transition-smooth"
            :class="activeTab === 'assets' 
              ? 'bg-card shadow-md text-foreground' 
              : 'text-muted-foreground hover:text-foreground'"
          >
            资产
          </button>
          <button
            @click="activeTab = 'liabilities'"
            class="px-4 py-2 rounded-lg text-sm font-medium transition-smooth"
            :class="activeTab === 'liabilities' 
              ? 'bg-card shadow-md text-foreground' 
              : 'text-muted-foreground hover:text-foreground'"
          >
            负债
          </button>
        </div>

        <!-- 资产列表 -->
        <section v-if="filteredAssets.length > 0">
          <div class="flex items-center gap-3 mb-4">
            <div class="w-8 h-8 rounded-lg bg-success/10 flex items-center justify-center">
              <Wallet class="w-4 h-4 text-success" />
            </div>
            <h2 class="text-lg font-semibold">资产</h2>
            <span class="text-sm text-muted-foreground">({{ filteredAssets.length }})</span>
          </div>
          
          <div v-if="!filteredAssets.length" class="text-center py-12 text-muted-foreground glass rounded-2xl">
            暂无资产条目
          </div>
          <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div
              v-for="entry in filteredAssets"
              :key="entry.id"
              class="group relative overflow-hidden rounded-2xl border border-border bg-card p-5 hover:border-success/30 hover:shadow-lg transition-smooth"
            >
              <div class="flex items-start justify-between mb-4">
                <div class="flex items-center gap-3 flex-1">
                  <div class="w-11 h-11 rounded-xl bg-success/10 flex items-center justify-center shrink-0">
                    <Wallet class="w-5 h-5 text-success" />
                  </div>
                  <div class="flex-1 min-w-0">
                    <h3 class="font-semibold text-base mb-0.5 truncate">{{ entry.name }}</h3>
                    <p class="text-xs text-muted-foreground">
                      {{ entry.isAccount ? "账户" : "资产" }} · 
                      {{ entry.valuationType === "fixed" ? "固定值" : "手动估值" }}
                    </p>
                  </div>
                </div>
                <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-smooth">
                  <button
                    @click="handleAdjustClick(entry)"
                    class="p-2 rounded-lg hover:bg-success/10 transition-smooth"
                    title="调整价值"
                  >
                    <DollarSign class="w-4 h-4 text-success" />
                  </button>
                  <button
                    @click="handleEditClick(entry)"
                    class="p-2 rounded-lg hover:bg-accent transition-smooth"
                    title="编辑"
                  >
                    <Edit2 class="w-4 h-4" />
                  </button>
                </div>
              </div>
              <div class="flex items-baseline gap-2">
                <span class="text-2xl font-bold text-success">¥{{ fmt(entry.value) }}</span>
              </div>
            </div>
          </div>
        </section>

        <!-- 负债列表 -->
        <section v-if="filteredLiabilities.length > 0">
          <div class="flex items-center gap-3 mb-4">
            <div class="w-8 h-8 rounded-lg bg-destructive/10 flex items-center justify-center">
              <CreditCard class="w-4 h-4 text-destructive" />
            </div>
            <h2 class="text-lg font-semibold">负债</h2>
            <span class="text-sm text-muted-foreground">({{ filteredLiabilities.length }})</span>
          </div>
          
          <div v-if="!filteredLiabilities.length" class="text-center py-12 text-muted-foreground glass rounded-2xl">
            暂无负债条目
          </div>
          <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div
              v-for="entry in filteredLiabilities"
              :key="entry.id"
              class="group relative overflow-hidden rounded-2xl border border-border bg-card p-5 hover:border-destructive/30 hover:shadow-lg transition-smooth"
            >
              <div class="flex items-start justify-between mb-4">
                <div class="flex items-center gap-3 flex-1">
                  <div class="w-11 h-11 rounded-xl bg-destructive/10 flex items-center justify-center shrink-0">
                    <CreditCard class="w-5 h-5 text-destructive" />
                  </div>
                  <div class="flex-1 min-w-0">
                    <h3 class="font-semibold text-base mb-0.5 truncate">{{ entry.name }}</h3>
                    <p class="text-xs text-muted-foreground">
                      {{ entry.valuationType === "fixed" ? "固定值" : "手动估值" }}
                    </p>
                  </div>
                </div>
                <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-smooth">
                  <button
                    @click="handleAdjustClick(entry)"
                    class="p-2 rounded-lg hover:bg-destructive/10 transition-smooth"
                    title="调整价值"
                  >
                    <DollarSign class="w-4 h-4 text-destructive" />
                  </button>
                  <button
                    @click="handleEditClick(entry)"
                    class="p-2 rounded-lg hover:bg-accent transition-smooth"
                    title="编辑"
                  >
                    <Edit2 class="w-4 h-4" />
                  </button>
                </div>
              </div>
              <div class="flex items-baseline gap-2">
                <span class="text-2xl font-bold text-destructive">¥{{ fmt(entry.value) }}</span>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>

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
  </div>
</template>
