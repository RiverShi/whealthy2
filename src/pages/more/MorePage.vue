<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import {
  BookMarked,
  ChevronRight,
  Plus,
  Check,
  BookOpen,
  Settings,
  Tag,
  Moon,
  Sun,
  Wallet,
  ChevronDown,
} from "lucide-vue-next";
import { useBookStore } from "@/stores/books";

const bookStore = useBookStore();
const router = useRouter();
const isDark = ref(false);
const showBookPicker = ref(false);
const showNewBookInput = ref(false);
const newBookName = ref("");

onMounted(async () => {
  await bookStore.fetchBooks();
  // 同步当前主题状态
  isDark.value = document.documentElement.classList.contains("dark");
});

function toggleTheme() {
  isDark.value = !isDark.value;
  document.documentElement.classList.toggle("dark");
}

async function selectBook(id: string) {
  bookStore.setActiveBook(id);
  showBookPicker.value = false;
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

const menuGroups = [
  {
    title: "账本管理",
    items: [
      { label: "所有账本", icon: BookOpen, to: "/books", desc: "查看和管理所有账本" },
    ],
  },
  {
    title: "数据设置",
    items: [
      { label: "分类管理", icon: Settings, to: "/settings/categories", desc: "收支分类设置" },
      { label: "标签管理", icon: Tag, to: "/settings/tags", desc: "账目标签管理" },
    ],
  },
];
</script>

<template>
  <div class="min-h-full bg-background">

    <!-- ── 页头 ────────────────────────────────────────────────────────── -->
    <div class="px-4 pt-4 pb-2">
      <h1 class="text-2xl font-bold">我的</h1>
      <p class="text-sm text-muted-foreground mt-0.5">账本与个人设置</p>
    </div>

    <!-- ── 当前账本卡片 ─────────────────────────────────────────────────── -->
    <div class="px-4 mb-4">
      <div class="bg-gradient-to-br from-primary to-primary/80 rounded-2xl p-4 shadow-smooth-lg">
        <p class="text-xs text-primary-foreground/70 mb-1 font-medium">当前账本</p>
        <div class="flex items-center gap-3 mb-3">
          <div class="w-10 h-10 rounded-xl bg-white/20 flex items-center justify-center shrink-0">
            <BookMarked class="w-5 h-5 text-primary-foreground" />
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-lg font-bold text-primary-foreground truncate">
              {{ bookStore.activeBook?.name ?? "未选择账本" }}
            </p>
          </div>
        </div>
        <button
          @click="showBookPicker = !showBookPicker"
          class="w-full flex items-center justify-center gap-2 py-2 bg-white/15 hover:bg-white/25 text-primary-foreground text-sm font-medium rounded-xl transition-colors cursor-pointer"
        >
          <span>切换账本</span>
          <ChevronDown class="w-4 h-4 transition-transform duration-200" :class="showBookPicker ? 'rotate-180' : ''" />
        </button>
      </div>
    </div>

    <!-- ── 账本选择器展开 ──────────────────────────────────────────────── -->
    <div v-if="showBookPicker" class="px-4 mb-4 animate-in">
      <div class="bg-card border border-border rounded-2xl overflow-hidden shadow-smooth">
        <div class="max-h-60 overflow-y-auto">
          <button
            v-for="book in bookStore.books"
            :key="book.id"
            @click="selectBook(book.id)"
            class="w-full flex items-center gap-3 px-4 py-3.5 border-b border-border/60 last:border-0 hover:bg-accent transition-colors cursor-pointer text-left"
          >
            <div class="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center shrink-0">
              <BookMarked class="w-4 h-4 text-primary" />
            </div>
            <span class="flex-1 text-sm font-medium truncate">{{ book.name }}</span>
            <Check
              class="w-4 h-4 text-primary shrink-0 transition-opacity"
              :class="bookStore.activeBookId === book.id ? 'opacity-100' : 'opacity-0'"
            />
          </button>
        </div>

        <!-- 新建账本 -->
        <div class="border-t border-border p-3">
          <div v-if="showNewBookInput" class="flex gap-2">
            <input
              v-model="newBookName"
              @keyup.enter="createAndSelectBook"
              placeholder="输入账本名称"
              autofocus
              class="flex-1 text-sm px-3 py-2.5 rounded-xl border border-input bg-background focus:outline-none focus:ring-2 focus:ring-ring"
            />
            <button
              @click="createAndSelectBook"
              class="px-4 py-2.5 rounded-xl bg-primary text-primary-foreground text-sm font-medium cursor-pointer"
            >创建</button>
          </div>
          <button
            v-else
            @click="showNewBookInput = true"
            class="w-full flex items-center gap-2.5 px-3 py-3 text-sm text-primary font-medium hover:bg-accent rounded-xl transition-colors cursor-pointer"
          >
            <Plus class="w-4 h-4" />
            <span>新建账本</span>
          </button>
        </div>
      </div>
    </div>

    <!-- ── 功能菜单 ────────────────────────────────────────────────────── -->
    <div class="px-4 space-y-4 pb-8">
      <div v-for="group in menuGroups" :key="group.title">
        <p class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2 px-1">{{ group.title }}</p>
        <div class="bg-card border border-border rounded-2xl overflow-hidden">
          <button
            v-for="(item, i) in group.items"
            :key="item.to"
            @click="router.push(item.to)"
            class="w-full flex items-center gap-3 px-4 py-4 hover:bg-accent transition-colors cursor-pointer text-left"
            :class="i < group.items.length - 1 ? 'border-b border-border/60' : ''"
          >
            <div class="w-9 h-9 rounded-xl bg-primary/10 flex items-center justify-center shrink-0">
              <component :is="item.icon" class="w-[18px] h-[18px] text-primary" />
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium">{{ item.label }}</p>
              <p class="text-xs text-muted-foreground mt-0.5">{{ item.desc }}</p>
            </div>
            <ChevronRight class="w-4 h-4 text-muted-foreground shrink-0" />
          </button>
        </div>
      </div>

      <!-- ── 外观设置 ─────────────────────────────────────────────────── -->
      <div>
        <p class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2 px-1">外观</p>
        <div class="bg-card border border-border rounded-2xl overflow-hidden">
          <div class="flex items-center gap-3 px-4 py-4">
            <div class="w-9 h-9 rounded-xl bg-primary/10 flex items-center justify-center shrink-0">
              <Moon v-if="!isDark" class="w-[18px] h-[18px] text-primary" />
              <Sun v-else class="w-[18px] h-[18px] text-primary" />
            </div>
            <div class="flex-1">
              <p class="text-sm font-medium">{{ isDark ? "深色模式" : "浅色模式" }}</p>
              <p class="text-xs text-muted-foreground mt-0.5">点击切换主题</p>
            </div>
            <!-- Toggle Switch -->
            <button
              @click="toggleTheme"
              class="relative w-12 h-6 rounded-full transition-colors duration-200 cursor-pointer shrink-0"
              :class="isDark ? 'bg-primary' : 'bg-border'"
            >
              <span
                class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform duration-200"
                :class="isDark ? 'translate-x-6' : 'translate-x-0'"
              />
            </button>
          </div>
        </div>
      </div>

      <!-- ── 应用信息 ─────────────────────────────────────────────────── -->
      <div class="text-center mt-4 pb-4">
        <div class="w-16 h-16 rounded-2xl bg-primary/10 flex items-center justify-center mx-auto mb-3">
          <Wallet class="w-8 h-8 text-primary" />
        </div>
        <p class="text-base font-bold">Wealthy</p>
        <p class="text-xs text-muted-foreground mt-1">个人资产与记账管理</p>
      </div>
    </div>
  </div>
</template>
