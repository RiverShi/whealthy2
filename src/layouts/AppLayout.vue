<script setup lang="ts">
import { RouterView, RouterLink, useRoute, useRouter } from "vue-router";
import {
  Home,
  LayoutList,
  Wallet,
  User,
  Plus,
  Pencil,
  BookMarked,
} from "lucide-vue-next";
import { useBookStore } from "@/stores/books";
import { onMounted } from "vue";

const route = useRoute();
const router = useRouter();
const bookStore = useBookStore();

function handleFAB() {
  if (route.path === '/records') {
    router.push({ path: '/records', query: { create: '1' } });
  } else if (route.path === '/assets') {
    router.push({ path: '/assets', query: { quickedit: '1' } });
  }
}

onMounted(async () => {
  await bookStore.fetchBooks();
  // 禁用 WKWebView UIScrollView 的原生滚动，防止 fixed 元素漂移
  // 只阻止 main 以外的区域触发滚动（main 内部有 overflow-y:auto 自己处理）
  document.addEventListener('touchmove', (e) => {
    const main = document.querySelector('main');
    if (main && main.contains(e.target as Node)) return;
    e.preventDefault();
  }, { passive: false });
});

const tabs = [
  { label: "首页", icon: Home, to: "/home" },
  { label: "账目", icon: LayoutList, to: "/records" },
  { label: "资产", icon: Wallet, to: "/assets" },
  { label: "我的", icon: User, to: "/more" },
];

function isActive(path: string) {
  return route.path === path || route.path.startsWith(path + "/");
}

// 是否需要显示"无账本"引导（我的页/首页/账本管理页不需要）
const needsBook = () =>
  !bookStore.activeBook &&
  !bookStore.loading &&
  route.path !== "/more" &&
  route.path !== "/home" &&
  route.path !== "/books";
</script>

<template>
  <div class="bg-background" style="position: fixed; inset: 0; display: flex; flex-direction: column; padding-top: env(safe-area-inset-top);">

    <!-- ══ 主内容区（可滚动） ════════════════════════════════════════════ -->
    <main
      class="overflow-y-auto overflow-x-hidden"
      style="flex: 1; min-height: 0; -webkit-overflow-scrolling: touch;"
    >
      <!-- 无账本引导页 -->
      <div
        v-if="needsBook()"
        class="h-full flex flex-col items-center justify-center gap-5 text-center px-8"
      >
        <div class="w-20 h-20 rounded-3xl bg-primary/10 flex items-center justify-center">
          <BookMarked class="w-10 h-10 text-primary" />
        </div>
        <div>
          <p class="text-xl font-bold mb-2">还没有账本</p>
          <p class="text-sm text-muted-foreground leading-relaxed">
            前往<strong>「我的」</strong>页面<br />创建第一本账本
          </p>
        </div>
        <RouterLink
          to="/more"
          class="flex items-center gap-2 px-6 py-3 bg-primary text-primary-foreground rounded-2xl font-medium text-sm cursor-pointer"
        >
          去创建账本
        </RouterLink>
      </div>

      <RouterView v-else />
    </main>

    <!-- ══ 全局 FAB ═══════════════════════════════════════════════════ -->
    <!-- 仅在账目页(+流水) 和资产页(快速更新) 显示 -->
    <button
      v-if="bookStore.activeBook && (route.path === '/records' || route.path === '/assets')"
      @click="handleFAB"
      class="fixed z-40 w-14 h-14 rounded-full bg-primary text-primary-foreground shadow-lg shadow-primary/30 flex items-center justify-center cursor-pointer hover:scale-105 active:scale-95 transition-transform duration-150"
      style="position: fixed; bottom: calc(36px + env(safe-area-inset-bottom) + 16px); right: 20px;"
      :aria-label="route.path === '/assets' ? '快速更新' : '新增流水'"
    >
      <Pencil v-if="route.path === '/assets'" class="w-5 h-5" />
      <Plus v-else class="w-6 h-6" />
    </button>

    <!-- ══ 底部标签栏 ════════════════════════════════════════════════════ -->
    <nav
      class="bg-card/95 backdrop-blur-xl border-t border-border"
      style="flex-shrink: 0; padding-bottom: env(safe-area-inset-bottom); z-index: 50;"
    >
      <div class="flex">
        <RouterLink
          v-for="tab in tabs"
          :key="tab.to"
          :to="tab.to"
          class="flex-1 flex flex-col items-center justify-center gap-[1px] pt-0.5 pb-0.5 cursor-pointer select-none transition-colors"
          :class="isActive(tab.to) ? 'text-primary' : 'text-muted-foreground'"
        >
          <div class="relative flex items-center justify-center">
            <div
              v-if="isActive(tab.to)"
              class="absolute -inset-x-3 inset-y-0 rounded-full bg-primary/10"
            />
            <component
              :is="tab.icon"
              class="relative w-[18px] h-[18px] transition-transform duration-200"
              :class="isActive(tab.to) ? 'scale-110' : ''"
            />
          </div>
          <span class="text-[10px] font-medium tracking-wide">{{ tab.label }}</span>
        </RouterLink>
      </div>
    </nav>
  </div>
</template>
