<script setup lang="ts">
import { onMounted } from "vue";
import { useBookStore } from "@/stores/books";
import { Plus, BookOpen, Archive, TrendingUp, Clock, Sparkles } from "lucide-vue-next";
import { useRouter } from "vue-router";
import { ref } from "vue";

const bookStore = useBookStore();
const router = useRouter();
const showCreate = ref(false);
const newBookName = ref("");
const creating = ref(false);

onMounted(() => bookStore.fetchBooks());

async function handleCreate() {
  if (!newBookName.value.trim()) return;
  creating.value = true;
  try {
    const book = await bookStore.createBook(newBookName.value.trim());
    showCreate.value = false;
    newBookName.value = "";
    router.push(`/books/${book.id}`);
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
  <div class="min-h-screen bg-gradient-to-br from-background via-background to-accent/5">
    <!-- 页头 -->
    <div class="border-b border-border/50 bg-card/50 backdrop-blur-sm sticky top-0 z-10">
      <div class="max-w-7xl mx-auto px-8 py-6">
        <div class="flex items-center justify-between">
          <div>
            <div class="flex items-center gap-3 mb-2">
              <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-primary to-primary/80 flex items-center justify-center shadow-md">
                <BookOpen class="w-5 h-5 text-primary-foreground" />
              </div>
              <h1 class="text-3xl font-bold bg-gradient-to-r from-foreground to-foreground/70 bg-clip-text text-transparent">
                我的账本
              </h1>
            </div>
            <p class="text-sm text-muted-foreground ml-[52px]">
              管理和追踪你的所有资产与负债
            </p>
          </div>
          <button
            @click="showCreate = true"
            class="group flex items-center gap-2 px-5 py-3 bg-primary text-primary-foreground rounded-xl text-sm font-medium hover:shadow-lg hover:scale-105 transition-smooth shadow-md"
          >
            <Plus class="w-4 h-4 group-hover:rotate-90 transition-smooth" />
            新建账本
          </button>
        </div>
      </div>
    </div>

    <div class="max-w-7xl mx-auto px-8 py-8">
      <!-- 加载中 -->
      <div v-if="bookStore.loading" class="flex flex-col items-center justify-center py-24">
        <div class="w-12 h-12 rounded-xl bg-primary/10 flex items-center justify-center mb-4 animate-pulse">
          <BookOpen class="w-6 h-6 text-primary" />
        </div>
        <p class="text-muted-foreground">加载中…</p>
      </div>

      <!-- 空状态 -->
      <div v-else-if="!bookStore.books.length" 
           class="flex flex-col items-center justify-center py-32 animate-in">
        <div class="relative mb-8">
          <div class="w-20 h-20 rounded-2xl bg-gradient-to-br from-primary/20 to-primary/10 flex items-center justify-center">
            <BookOpen class="w-10 h-10 text-primary" />
          </div>
          <div class="absolute -top-2 -right-2 w-6 h-6 rounded-full bg-primary/20 flex items-center justify-center">
            <Sparkles class="w-3 h-3 text-primary" />
          </div>
        </div>
        <h3 class="text-xl font-semibold mb-2">开始你的财富之旅</h3>
        <p class="text-sm text-muted-foreground mb-8 max-w-md text-center">
          创建第一个账本，开始记录和管理你的资产，让财富增长更清晰可见
        </p>
        <button
          @click="showCreate = true"
          class="group flex items-center gap-2 px-6 py-3 bg-primary text-primary-foreground rounded-xl text-sm font-medium hover:shadow-lg hover:scale-105 transition-smooth shadow-md"
        >
          <Plus class="w-4 h-4 group-hover:rotate-90 transition-smooth" />
          创建第一个账本
        </button>
      </div>

      <!-- 账本列表 -->
      <div v-else class="animate-in">
        <!-- 统计卡片 -->
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
          <div class="glass rounded-2xl p-6 shadow-smooth">
            <div class="flex items-center justify-between mb-3">
              <p class="text-sm text-muted-foreground">账本总数</p>
              <div class="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center">
                <BookOpen class="w-4 h-4 text-primary" />
              </div>
            </div>
            <p class="text-3xl font-bold">{{ bookStore.books.length }}</p>
            <p class="text-xs text-muted-foreground mt-2">
              活跃账本 {{ bookStore.books.filter(b => !b.archivedAt).length }} 个
            </p>
          </div>

          <div class="glass rounded-2xl p-6 shadow-smooth">
            <div class="flex items-center justify-between mb-3">
              <p class="text-sm text-muted-foreground">最近活动</p>
              <div class="w-8 h-8 rounded-lg bg-success/10 flex items-center justify-center">
                <Clock class="w-4 h-4 text-success" />
              </div>
            </div>
            <p class="text-3xl font-bold">
              {{ bookStore.books.length > 0 ? formatDate(bookStore.books[0].createdAt) : '-' }}
            </p>
            <p class="text-xs text-muted-foreground mt-2">上次更新时间</p>
          </div>

          <div class="glass rounded-2xl p-6 shadow-smooth">
            <div class="flex items-center justify-between mb-3">
              <p class="text-sm text-muted-foreground">管理效率</p>
              <div class="w-8 h-8 rounded-lg bg-info/10 flex items-center justify-center">
                <TrendingUp class="w-4 h-4 text-info" />
              </div>
            </div>
            <p class="text-3xl font-bold">优秀</p>
            <p class="text-xs text-muted-foreground mt-2">财务记录完整度</p>
          </div>
        </div>

        <!-- 账本网格 -->
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
          <router-link
            v-for="(book, index) in bookStore.books"
            :key="book.id"
            :to="`/books/${book.id}`"
            class="group block rounded-2xl border border-border bg-card p-6 hover:border-primary/40 hover:shadow-lg transition-smooth animate-in"
            :style="{ animationDelay: `${index * 50}ms` }"
          >
            <div class="flex items-start justify-between mb-5">
              <div class="flex items-center gap-3">
                <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-primary/20 to-primary/10 flex items-center justify-center group-hover:scale-110 transition-smooth">
                  <BookOpen class="w-5 h-5 text-primary" />
                </div>
                <div>
                  <h3 class="font-semibold text-base group-hover:text-primary transition-smooth">
                    {{ book.name }}
                  </h3>
                  <p class="text-xs text-muted-foreground mt-0.5">
                    账本 ID: {{ book.id.slice(0, 8) }}
                  </p>
                </div>
              </div>
              <Archive 
                v-if="book.archivedAt" 
                class="w-5 h-5 text-muted-foreground/50" 
                title="已归档"
              />
            </div>

            <div class="space-y-3">
              <div class="flex items-center gap-2 text-sm">
                <Clock class="w-4 h-4 text-muted-foreground/50" />
                <span class="text-muted-foreground">
                  创建于 {{ formatDate(book.createdAt) }}
                </span>
              </div>
              
              <div class="pt-3 border-t border-border/50">
                <div class="flex items-center justify-between text-xs text-muted-foreground">
                  <span>查看详情</span>
                  <span class="opacity-0 group-hover:opacity-100 transition-smooth">→</span>
                </div>
              </div>
            </div>
          </router-link>

          <!-- 新建账本卡片 -->
          <button
            @click="showCreate = true"
            class="group flex flex-col items-center justify-center rounded-2xl border-2 border-dashed border-border p-6 hover:border-primary hover:bg-accent/30 transition-smooth min-h-[200px]"
          >
            <div class="w-12 h-12 rounded-xl bg-primary/10 flex items-center justify-center mb-4 group-hover:scale-110 transition-smooth">
              <Plus class="w-6 h-6 text-primary group-hover:rotate-90 transition-smooth" />
            </div>
            <p class="font-medium text-sm text-muted-foreground group-hover:text-foreground transition-smooth">
              创建新账本
            </p>
          </button>
        </div>
      </div>
    </div>

    <!-- 新建对话框 -->
    <div 
      v-if="showCreate" 
      class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 animate-in"
      @click.self="showCreate = false; newBookName = ''"
    >
      <div class="bg-card rounded-2xl p-8 w-full max-w-md shadow-2xl border border-border animate-in">
        <div class="flex items-center gap-3 mb-6">
          <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-primary to-primary/80 flex items-center justify-center">
            <BookOpen class="w-5 h-5 text-primary-foreground" />
          </div>
          <h2 class="text-xl font-bold">新建账本</h2>
        </div>
        
        <div class="mb-6">
          <label class="block text-sm font-medium mb-2 text-muted-foreground">账本名称</label>
          <input
            v-model="newBookName"
            placeholder="例如：个人资产、家庭账本"
            class="w-full px-4 py-3 rounded-xl border border-input bg-background text-sm focus:outline-none focus:ring-2 focus:ring-ring focus:border-transparent transition-smooth"
            @keyup.enter="handleCreate"
            autofocus
          />
        </div>
        
        <div class="flex gap-3">
          <button
            @click="showCreate = false; newBookName = ''"
            class="flex-1 px-4 py-3 rounded-xl text-sm font-medium text-muted-foreground hover:bg-accent transition-smooth"
          >
            取消
          </button>
          <button
            :disabled="!newBookName.trim() || creating"
            @click="handleCreate"
            class="flex-1 px-4 py-3 rounded-xl bg-primary text-primary-foreground text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed hover:shadow-lg hover:scale-105 transition-smooth enabled:shadow-md"
          >
            {{ creating ? "创建中…" : "创建账本" }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
