<script setup lang="ts">
import { RouterView, RouterLink, useRoute } from "vue-router";
import {
  BookOpen,
  BarChart3,
  Settings,
  Wallet,
  ChevronRight,
  Plus,
  Search,
  Moon,
  Sun,
} from "lucide-vue-next";
import { useBookStore } from "@/stores/books";
import { computed, ref } from "vue";

const route = useRoute();
const bookStore = useBookStore();
const isDark = ref(false);
const searchQuery = ref("");

const navMain = [
  { label: "账本", icon: BookOpen, to: "/books", badge: null },
  { label: "统计视图", icon: BarChart3, to: "/views/default", badge: null },
];

const navSettings = [
  { label: "分类管理", icon: Settings, to: "/settings/categories" },
  { label: "标签管理", icon: Settings, to: "/settings/tags" },
];

const recentBooks = computed(() => bookStore.books.slice(0, 5));

function isActive(path: string) {
  return route.path.startsWith(path);
}

function toggleTheme() {
  isDark.value = !isDark.value;
  document.documentElement.classList.toggle("dark");
}
</script>

<template>
  <div class="flex h-screen bg-background text-foreground overflow-hidden">
    <!-- 侧边栏 -->
    <aside class="w-64 shrink-0 flex flex-col border-r border-border bg-card shadow-smooth">
      <!-- Logo区域 -->
      <div class="flex items-center justify-between px-6 py-6 border-b border-border">
        <div class="flex items-center gap-3">
          <div class="flex items-center justify-center w-10 h-10 rounded-xl bg-gradient-to-br from-primary to-primary/80 text-primary-foreground shadow-md">
            <Wallet class="w-5 h-5" />
          </div>
          <div>
            <span class="font-bold text-lg tracking-tight">Wealthy</span>
            <p class="text-xs text-muted-foreground">财富管理</p>
          </div>
        </div>
      </div>

      <!-- 搜索框 -->
      <div class="px-4 py-3 border-b border-border">
        <div class="relative">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            v-model="searchQuery"
            placeholder="搜索账本..."
            class="w-full pl-9 pr-3 py-2 rounded-lg border border-input bg-background text-sm focus:outline-none focus:ring-2 focus:ring-ring transition-smooth"
          />
        </div>
      </div>

      <!-- 主导航 -->
      <nav class="flex-1 overflow-y-auto py-4 px-3 space-y-1">
        <div class="mb-4">
          <RouterLink
            v-for="item in navMain"
            :key="item.to"
            :to="item.to"
            class="group flex items-center justify-between gap-3 px-3 py-2.5 rounded-xl text-sm font-medium transition-smooth"
            :class="isActive(item.to)
              ? 'bg-primary text-primary-foreground shadow-md'
              : 'text-foreground/70 hover:bg-accent hover:text-accent-foreground'"
          >
            <div class="flex items-center gap-3">
              <component :is="item.icon" class="w-[18px] h-[18px] shrink-0" />
              <span>{{ item.label }}</span>
            </div>
            <ChevronRight 
              v-if="isActive(item.to)" 
              class="w-4 h-4 opacity-70" 
            />
          </RouterLink>
        </div>

        <!-- 账本快捷列表 -->
        <div v-if="recentBooks.length" class="pt-2">
          <div class="flex items-center justify-between px-3 pb-2">
            <p class="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
              最近账本
            </p>
            <button 
              @click="$router.push('/books')"
              class="text-primary hover:text-primary/80 transition-smooth"
            >
              <Plus class="w-4 h-4" />
            </button>
          </div>
          <RouterLink
            v-for="book in recentBooks"
            :key="book.id"
            :to="`/books/${book.id}`"
            class="group flex items-center justify-between gap-2 px-3 py-2 rounded-lg text-sm transition-smooth"
            :class="route.params.id === book.id
              ? 'bg-accent text-accent-foreground font-medium'
              : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'"
          >
            <div class="flex items-center gap-2 flex-1 min-w-0">
              <div class="w-1.5 h-1.5 rounded-full bg-primary shrink-0" 
                   :class="route.params.id === book.id ? 'opacity-100' : 'opacity-0'"></div>
              <span class="truncate">{{ book.name }}</span>
            </div>
            <ChevronRight class="w-3.5 h-3.5 opacity-0 group-hover:opacity-50 transition-smooth shrink-0" />
          </RouterLink>
        </div>
      </nav>

      <!-- 底部设置和主题切换 -->
      <div class="py-3 px-3 border-t border-border space-y-1">
        <RouterLink
          v-for="item in navSettings"
          :key="item.to"
          :to="item.to"
          class="flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-smooth text-muted-foreground hover:bg-accent hover:text-accent-foreground"
        >
          <component :is="item.icon" class="w-[18px] h-[18px] shrink-0" />
          <span>{{ item.label }}</span>
        </RouterLink>
        
        <button
          @click="toggleTheme"
          class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-smooth text-muted-foreground hover:bg-accent hover:text-accent-foreground"
        >
          <Moon v-if="!isDark" class="w-[18px] h-[18px] shrink-0" />
          <Sun v-else class="w-[18px] h-[18px] shrink-0" />
          <span>{{ isDark ? "浅色模式" : "深色模式" }}</span>
        </button>
      </div>
    </aside>

    <!-- 主内容区 -->
    <main class="flex-1 overflow-y-auto bg-background">
      <RouterView />
    </main>
  </div>
</template>
