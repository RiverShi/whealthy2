<script setup lang="ts">
import { RouterView, useRoute, RouterLink } from "vue-router";
import {
  LayoutList,
  BarChart3,
  Settings,
  Wallet,
  ChevronDown,
  Moon,
  Sun,
  BookOpen,
  Tag,
  BookMarked,
  Plus,
  Check,
} from "lucide-vue-next";
import { useBookStore } from "@/stores/books";
import { ref, onMounted } from "vue";

const route = useRoute();
const bookStore = useBookStore();
const isDark = ref(false);
const showBookPicker = ref(false);
const showNewBookInput = ref(false);
const newBookName = ref("");

onMounted(async () => {
  await bookStore.fetchBooks();
});

// 主导航 tab
const mainTabs = [
  { label: "账目", icon: LayoutList, to: "/records" },
  { label: "资产", icon: Wallet, to: "/assets" },
  { label: "统计", icon: BarChart3, to: "/stats" },
];

// 底部管理
const settingLinks = [
  { label: "账本管理", icon: BookOpen, to: "/books" },
  { label: "分类管理", icon: Settings, to: "/settings/categories" },
  { label: "标签管理", icon: Tag, to: "/settings/tags" },
];

function isActive(path: string) {
  return route.path === path || route.path.startsWith(path + "/");
}

function toggleTheme() {
  isDark.value = !isDark.value;
  document.documentElement.classList.toggle("dark");
}

async function selectBook(id: string) {
  bookStore.setActiveBook(id);
  showBookPicker.value = false;
  // 刷新当前页面数据（路由不变，各页面 watch activeBookId）
}

async function createAndSelectBook() {
  const name = newBookName.value.trim();
  if (!name) return;
  const book = await bookStore.createBook(name);
  bookStore.setActiveBook(book.id);
  newBookName.value = "";
  showNewBookInput.value = false;
  showBookPicker.value = false;
}
</script>

<template>
  <div class="flex h-screen bg-background text-foreground overflow-hidden" @click="showBookPicker = false">
    <!-- ══ 左侧边栏 ══════════════════════════════════════════════════════════ -->
    <aside class="w-56 shrink-0 flex flex-col border-r border-border bg-card" @click.stop>

      <!-- 账本切换器（顶部） -->
      <div class="relative px-3 pt-4 pb-3 border-b border-border">
        <button
          @click="showBookPicker = !showBookPicker"
          class="w-full flex items-center gap-2.5 px-3 py-2.5 rounded-xl hover:bg-accent transition-colors text-left"
        >
          <div class="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center shrink-0">
            <BookMarked class="w-4 h-4 text-primary" />
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-[11px] text-muted-foreground leading-none mb-0.5">当前账本</p>
            <p class="text-sm font-semibold truncate leading-tight">
              {{ bookStore.activeBook?.name ?? "请选择账本" }}
            </p>
          </div>
          <ChevronDown class="w-4 h-4 text-muted-foreground shrink-0 transition-transform" :class="showBookPicker ? 'rotate-180' : ''" />
        </button>

        <!-- 账本下拉面板 -->
        <div
          v-if="showBookPicker"
          class="absolute left-3 right-3 top-full mt-1 z-50 rounded-xl border border-border bg-card shadow-lg overflow-hidden"
        >
          <div class="max-h-52 overflow-y-auto py-1">
            <button
              v-for="book in bookStore.books"
              :key="book.id"
              @click="selectBook(book.id)"
              class="w-full flex items-center gap-2.5 px-3 py-2 text-sm hover:bg-accent transition-colors text-left"
            >
              <Check
                class="w-3.5 h-3.5 shrink-0 text-primary"
                :class="bookStore.activeBookId === book.id ? 'opacity-100' : 'opacity-0'"
              />
              <span class="truncate">{{ book.name }}</span>
            </button>
          </div>
          <!-- 新建账本 -->
          <div class="border-t border-border p-2">
            <div v-if="showNewBookInput" class="flex gap-1.5">
              <input
                v-model="newBookName"
                @keyup.enter="createAndSelectBook"
                placeholder="账本名称"
                autofocus
                class="flex-1 text-sm px-2.5 py-1.5 rounded-lg border border-input bg-background focus:outline-none focus:ring-1 focus:ring-ring"
              />
              <button @click="createAndSelectBook" class="px-2.5 py-1.5 rounded-lg bg-primary text-primary-foreground text-xs font-medium">创建</button>
            </div>
            <button
              v-else
              @click="showNewBookInput = true"
              class="w-full flex items-center gap-2 px-3 py-1.5 text-sm text-muted-foreground hover:text-foreground hover:bg-accent rounded-lg transition-colors"
            >
              <Plus class="w-3.5 h-3.5" />
              <span>新建账本</span>
            </button>
          </div>
        </div>
      </div>

      <!-- 主导航 Tab -->
      <nav class="flex-1 py-3 px-3 space-y-0.5">
        <RouterLink
          v-for="tab in mainTabs"
          :key="tab.to"
          :to="tab.to"
          class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium transition-colors"
          :class="isActive(tab.to)
            ? 'bg-primary text-primary-foreground'
            : 'text-foreground/70 hover:bg-accent hover:text-foreground'"
        >
          <component :is="tab.icon" class="w-[18px] h-[18px] shrink-0" />
          <span>{{ tab.label }}</span>
        </RouterLink>
      </nav>

      <!-- 底部管理区 -->
      <div class="py-3 px-3 border-t border-border space-y-0.5">
        <RouterLink
          v-for="item in settingLinks"
          :key="item.to"
          :to="item.to"
          class="flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors"
          :class="isActive(item.to)
            ? 'bg-accent text-foreground font-medium'
            : 'text-muted-foreground hover:bg-accent hover:text-foreground'"
        >
          <component :is="item.icon" class="w-[17px] h-[17px] shrink-0" />
          <span>{{ item.label }}</span>
        </RouterLink>

        <button
          @click="toggleTheme"
          class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors text-muted-foreground hover:bg-accent hover:text-foreground"
        >
          <Moon v-if="!isDark" class="w-[17px] h-[17px] shrink-0" />
          <Sun v-else class="w-[17px] h-[17px] shrink-0" />
          <span>{{ isDark ? "浅色模式" : "深色模式" }}</span>
        </button>
      </div>
    </aside>

    <!-- ══ 主内容区 ══════════════════════════════════════════════════════════ -->
    <main class="flex-1 overflow-y-auto bg-background">
      <!-- 无账本时引导 -->
      <div v-if="!bookStore.activeBook && !bookStore.loading" class="h-full flex flex-col items-center justify-center gap-4 text-center px-8">
        <div class="w-16 h-16 rounded-2xl bg-primary/10 flex items-center justify-center">
          <BookMarked class="w-8 h-8 text-primary" />
        </div>
        <div>
          <p class="text-lg font-semibold mb-1">还没有账本</p>
          <p class="text-sm text-muted-foreground">点击左上角「当前账本」区域创建第一本账本</p>
        </div>
      </div>
      <RouterView v-else />
    </main>
  </div>
</template>
