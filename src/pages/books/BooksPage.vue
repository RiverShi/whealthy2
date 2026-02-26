<script setup lang="ts">
import { onMounted } from "vue";
import { useBookStore } from "@/stores/books";
import { Plus, BookOpen, Clock, Trash2 } from "lucide-vue-next";
import { useRouter } from "vue-router";
import { ref } from "vue";
import type { Book } from "@/api/books";

const bookStore = useBookStore();
const router = useRouter();
const showCreate = ref(false);
const newBookName = ref("");
const creating = ref(false);
const createError = ref("");
const confirmDeleteBook = ref<Book | null>(null);
const deleting = ref(false);

onMounted(() => bookStore.fetchBooks());

async function handleDeleteBook() {
  if (!confirmDeleteBook.value) return;
  deleting.value = true;
  try {
    await bookStore.deleteBook(confirmDeleteBook.value.id);
    if (bookStore.activeBookId === confirmDeleteBook.value.id) {
      const remaining = bookStore.books[0];
      if (remaining) bookStore.setActiveBook(remaining.id);
    }
    confirmDeleteBook.value = null;
  } finally {
    deleting.value = false;
  }
}

async function handleCreate() {
  if (!newBookName.value.trim()) return;
  createError.value = "";
  creating.value = true;
  try {
    const book = await bookStore.createBook(newBookName.value.trim());
    showCreate.value = false;
    newBookName.value = "";
    bookStore.setActiveBook(book.id);
    router.push("/records");
  } catch (e: unknown) {
    createError.value = e instanceof Error ? e.message : "创建失败，请重试";
  } finally {
    creating.value = false;
  }
}

function formatDate(dateStr: string) {
  const date = new Date(dateStr);
  const now = new Date();
  const diffTime = Math.abs(now.getTime() - date.getTime());
  const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));
  
  if (diffDays === 0) return "今天";
  if (diffDays === 1) return "昨天";
  if (diffDays < 7) return `${diffDays}天前`;
  
  return date.toLocaleDateString("zh-CN", { year: 'numeric', month: 'short', day: 'numeric' });
}
</script>

<template>
  <div class="min-h-full bg-background">

    <!-- ── 顶部导航头 ──────────────────────────────────────────────────── -->
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
        <h1 class="text-xl font-bold flex-1">所有账本</h1>
        <button
          @click="showCreate = true"
          class="flex items-center gap-1.5 px-3 py-2 bg-primary text-primary-foreground rounded-xl text-sm font-medium cursor-pointer"
        >
          <Plus class="w-3.5 h-3.5" />新建
        </button>
      </div>
    </div>

    <!-- ── 内容区 ──────────────────────────────────────────────────────── -->
    <div class="px-4 py-3 pb-8">

      <!-- 加载中 -->
      <div v-if="bookStore.loading" class="flex flex-col items-center justify-center py-24">
        <div class="w-12 h-12 rounded-xl bg-primary/10 flex items-center justify-center mb-4 animate-pulse">
          <BookOpen class="w-6 h-6 text-primary" />
        </div>
        <p class="text-muted-foreground text-sm">加载中…</p>
      </div>

      <!-- 空状态 -->
      <div v-else-if="!bookStore.books.length" class="flex flex-col items-center justify-center py-24 animate-in">
        <div class="w-20 h-20 rounded-2xl bg-primary/10 flex items-center justify-center mb-5">
          <BookOpen class="w-10 h-10 text-primary" />
        </div>
        <p class="text-xl font-semibold mb-2">开始你的财富之旅</p>
        <p class="text-sm text-muted-foreground mb-6 text-center leading-relaxed">
          创建第一个账本，开始记录和管理你的资产
        </p>
        <button
          @click="showCreate = true"
          class="flex items-center gap-2 px-6 py-3 bg-primary text-primary-foreground rounded-2xl text-sm font-medium cursor-pointer shadow-smooth"
        >
          <Plus class="w-4 h-4" />
          创建第一个账本
        </button>
      </div>

      <!-- 账本列表 -->
      <div v-else class="space-y-3 animate-in">
        <!-- 统计汇总 -->
        <div class="flex gap-2 mb-4">
          <div class="flex-1 rounded-2xl bg-card border border-border p-3 text-center">
            <p class="text-2xl font-bold">{{ bookStore.books.length }}</p>
            <p class="text-xs text-muted-foreground mt-0.5">账本总数</p>
          </div>
          <div class="flex-1 rounded-2xl bg-card border border-border p-3 text-center">
            <p class="text-2xl font-bold">{{ bookStore.books.length }}</p>
            <p class="text-xs text-muted-foreground mt-0.5">活跃账本</p>
          </div>
        </div>

        <!-- 账本卡片列表 -->
        <div class="bg-card border border-border rounded-2xl overflow-hidden">
          <div
            v-for="(book, i) in bookStore.books"
            :key="book.id"
            @click="bookStore.setActiveBook(book.id); router.push('/records')"
            class="flex items-center gap-3 px-4 py-4 cursor-pointer active:bg-accent/40 transition-colors"
            :class="[
              i < bookStore.books.length - 1 ? 'border-b border-border/60' : '',
              bookStore.activeBookId === book.id ? 'bg-primary/5' : ''
            ]"
          >
            <div
              class="w-11 h-11 rounded-xl flex items-center justify-center shrink-0"
              :class="bookStore.activeBookId === book.id ? 'bg-primary/15' : 'bg-muted/50'"
            >
              <BookOpen
                class="w-5 h-5"
                :class="bookStore.activeBookId === book.id ? 'text-primary' : 'text-muted-foreground'"
              />
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-semibold truncate">{{ book.name }}</p>
              <p class="text-xs text-muted-foreground mt-0.5">
                <Clock class="inline w-3 h-3 mr-0.5" />{{ formatDate(book.createdAt) }}
              </p>
            </div>
            <div class="flex items-center gap-2 shrink-0">
              <span
                v-if="bookStore.activeBookId === book.id"
                class="text-[10px] font-medium px-2 py-0.5 rounded-full bg-primary/10 text-primary"
              >当前</span>
              <button
                @click.stop="confirmDeleteBook = book"
                class="p-1.5 rounded-lg text-muted-foreground/50 hover:text-destructive hover:bg-destructive/10 transition-colors cursor-pointer"
              >
                <Trash2 class="w-4 h-4" />
              </button>
              <svg class="w-4 h-4 text-muted-foreground/50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
              </svg>
            </div>
          </div>
        </div>

        <!-- 新建账本按钮 -->
        <button
          @click="showCreate = true"
          class="w-full flex items-center justify-center gap-2 py-4 rounded-2xl border-2 border-dashed border-border hover:border-primary hover:bg-primary/5 transition-colors text-sm font-medium text-muted-foreground hover:text-primary cursor-pointer"
        >
          <Plus class="w-4 h-4" />
          创建新账本
        </button>
      </div>
    </div>

    <!-- ── 删除账本确认弹窗 ─────────────────────────────────────────────── -->
    <Teleport to="body">
      <div
        v-if="confirmDeleteBook"
        class="fixed inset-0 z-[100] flex items-end justify-center bg-black/40"
        @click.self="confirmDeleteBook = null"
      >
        <div class="bg-card border border-border rounded-t-3xl w-full max-w-lg p-6 shadow-xl"
             style="padding-bottom: calc(env(safe-area-inset-bottom) + 24px)">
          <div class="w-10 h-1 bg-border rounded-full mx-auto mb-5" />
          <div class="w-12 h-12 rounded-2xl bg-destructive/10 flex items-center justify-center mx-auto mb-4">
            <Trash2 class="w-6 h-6 text-destructive" />
          </div>
          <p class="font-bold text-lg text-center mb-1">删除「{{ confirmDeleteBook!.name }}」？</p>
          <p class="text-sm text-muted-foreground text-center mb-6 leading-relaxed">
            账本及其下所有数据将被永久删除，此操作无法撤销。
          </p>
          <div class="flex gap-3">
            <button
              @click="confirmDeleteBook = null"
              class="flex-1 py-3.5 rounded-2xl border border-border text-sm font-medium hover:bg-accent transition-colors cursor-pointer"
            >取消</button>
            <button
              :disabled="deleting"
              @click="handleDeleteBook"
              class="flex-1 py-3.5 rounded-2xl bg-destructive text-destructive-foreground text-sm font-medium disabled:opacity-60 hover:opacity-90 transition-opacity cursor-pointer"
            >{{ deleting ? '删除中…' : '确认删除' }}</button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- ── 新建账本底部弹窗 ─────────────────────────────────────────────── -->
    <Teleport to="body">
      <div
        v-if="showCreate"
        class="fixed inset-0 z-50 flex items-end justify-center bg-black/40"
        @click.self="showCreate = false; newBookName = ''; createError = ''"
      >
        <div class="bg-card border border-border rounded-t-3xl w-full max-w-lg p-6 shadow-xl animate-in"
             style="padding-bottom: calc(env(safe-area-inset-bottom) + 24px)">
          <div class="w-10 h-1 bg-border rounded-full mx-auto mb-5" />
          <div class="flex items-center gap-3 mb-5">
            <div class="w-10 h-10 rounded-xl bg-primary/10 flex items-center justify-center">
              <BookOpen class="w-5 h-5 text-primary" />
            </div>
            <h2 class="text-lg font-bold">新建账本</h2>
          </div>
          <div class="mb-5">
            <label class="block text-sm font-medium mb-2 text-muted-foreground">账本名称</label>
            <input
              v-model="newBookName"
              placeholder="例如：个人资产、家庭账本"
              class="w-full px-4 py-3.5 rounded-2xl border bg-background text-base focus:outline-none focus:ring-2 transition-colors"
              :class="createError ? 'border-destructive focus:ring-destructive/40' : 'border-input focus:ring-ring'"
              @keyup.enter="handleCreate"
              @input="createError = ''"
              autofocus
            />
            <p v-if="createError" class="mt-2 text-xs text-destructive">{{ createError }}</p>
          </div>
          <div class="flex gap-3">
            <button
              @click="showCreate = false; newBookName = ''; createError = ''"
              class="flex-1 py-3.5 rounded-2xl border border-border text-sm font-medium cursor-pointer"
            >取消</button>
            <button
              :disabled="!newBookName.trim() || creating"
              @click="handleCreate"
              class="flex-1 py-3.5 rounded-2xl bg-primary text-primary-foreground text-sm font-medium disabled:opacity-50 cursor-pointer"
            >{{ creating ? "创建中…" : "创建账本" }}</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
