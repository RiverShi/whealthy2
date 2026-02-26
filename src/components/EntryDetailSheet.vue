<script setup lang="ts">
import { computed } from "vue";
import {
  Wallet, CreditCard, DollarSign, Edit2, Trash2, X,
  Tag, TrendingUp, TrendingDown, Calendar, BadgeCheck,
} from "lucide-vue-next";
import type { Entry } from "@/api/entries";

const props = defineProps<{
  open?: boolean;
  entry?: Entry;
  categoryName?: string;    // 由父组件查好传入
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
  adjust: [entry: Entry];
  edit: [entry: Entry];
  delete: [entry: Entry];
}>();

function close() { emit("update:open", false); }

function fmt(v: number) {
  return v.toLocaleString("zh-CN", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

const isAsset = computed(() => props.entry?.kind === "asset");

const kindLabel = computed(() => {
  if (!props.entry) return "";
  if (props.entry.kind === "asset") return props.entry.isAccount ? "账户（资产）" : "资产";
  return props.entry.isAccount ? "账户（负债）" : "负债";
});

const valuationLabel = computed(() => {
  if (!props.entry) return "";
  return props.entry.valuationType === "fixed" ? "固定值" : "手动估值";
});
</script>

<template>
  <Teleport to="body">
    <!-- 遮罩 -->
    <Transition
      enter-active-class="transition-opacity duration-200"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-opacity duration-200"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div v-if="open" class="fixed inset-0 z-50 bg-black/50" @click="close" />
    </Transition>

    <!-- 底部面板 -->
    <Transition
      enter-active-class="transition-transform duration-300 ease-out"
      enter-from-class="translate-y-full"
      enter-to-class="translate-y-0"
      leave-active-class="transition-transform duration-200 ease-in"
      leave-from-class="translate-y-0"
      leave-to-class="translate-y-full"
    >
      <div
        v-if="open && entry"
        class="fixed bottom-0 left-0 right-0 z-50 bg-card rounded-t-3xl shadow-2xl max-h-[80vh] overflow-y-auto"
        style="padding-bottom: calc(env(safe-area-inset-bottom) + 24px)"
      >
        <!-- 拖动条 -->
        <div class="flex justify-center pt-3 pb-1">
          <div class="w-10 h-1 bg-border rounded-full" />
        </div>

        <!-- 关闭按钮 -->
        <button
          @click="close"
          class="absolute top-4 right-4 p-2 rounded-full hover:bg-accent text-muted-foreground transition-colors cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>

        <div class="px-6 pt-2 pb-6">
          <!-- 顶部：图标 + 名称 + 分类 -->
          <div class="flex items-start gap-4 mb-6">
            <div
              class="w-14 h-14 rounded-2xl flex items-center justify-center shrink-0"
              :class="isAsset ? 'bg-emerald-500/15' : 'bg-rose-500/15'"
            >
              <component
                :is="isAsset ? Wallet : CreditCard"
                class="w-7 h-7"
                :class="isAsset ? 'text-emerald-500' : 'text-rose-500'"
              />
            </div>
            <div class="flex-1 min-w-0 pt-1">
              <h2 class="text-xl font-bold leading-tight mb-1.5 break-words">{{ entry.name }}</h2>
              <div class="flex flex-wrap gap-1.5">
                <!-- 分类标签 -->
                <span
                  v-if="categoryName"
                  class="inline-flex items-center gap-1 text-xs font-medium px-2.5 py-1 rounded-full"
                  :class="isAsset ? 'bg-emerald-500/10 text-emerald-600' : 'bg-rose-500/10 text-rose-600'"
                >
                  <Tag class="w-3 h-3" />
                  {{ categoryName }}
                </span>
                <!-- 类型标签 -->
                <span
                  class="inline-flex items-center gap-1 text-xs px-2.5 py-1 rounded-full bg-muted text-muted-foreground"
                >
                  {{ kindLabel }}
                </span>
              </div>
            </div>
          </div>

          <!-- 当前价值 -->
          <div class="rounded-2xl border border-border bg-background p-4 mb-4">
            <p class="text-xs text-muted-foreground mb-1">
              {{ isAsset ? "当前价值" : "欠款金额" }}
            </p>
            <p
              class="text-3xl font-bold tabular-nums"
              :class="isAsset ? 'text-emerald-500' : 'text-rose-500'"
            >
              <span class="text-lg font-semibold mr-0.5">¥</span>{{ fmt(entry.value) }}
            </p>
          </div>

          <!-- 详细信息 -->
          <div class="rounded-2xl border border-border bg-background divide-y divide-border/60 mb-6">
            <div class="flex items-center justify-between px-4 py-3">
              <span class="text-sm text-muted-foreground flex items-center gap-2">
                <component :is="isAsset ? TrendingUp : TrendingDown" class="w-3.5 h-3.5" />
                类型
              </span>
              <span class="text-sm font-medium">{{ entry.kind === 'asset' ? '资产' : '负债' }}</span>
            </div>
            <div v-if="entry.isAccount" class="flex items-center justify-between px-4 py-3">
              <span class="text-sm text-muted-foreground flex items-center gap-2">
                <BadgeCheck class="w-3.5 h-3.5" />
                账户属性
              </span>
              <span class="text-xs font-medium px-2 py-0.5 rounded-full bg-primary/10 text-primary">可关联流水</span>
            </div>
            <div class="flex items-center justify-between px-4 py-3">
              <span class="text-sm text-muted-foreground flex items-center gap-2">
                <DollarSign class="w-3.5 h-3.5" />
                估值方式
              </span>
              <span class="text-sm font-medium">{{ valuationLabel }}</span>
            </div>
            <div v-if="entry.openedAt" class="flex items-center justify-between px-4 py-3">
              <span class="text-sm text-muted-foreground flex items-center gap-2">
                <Calendar class="w-3.5 h-3.5" />
                创建日期
              </span>
              <span class="text-sm font-medium">{{ entry.openedAt }}</span>
            </div>
          </div>

          <!-- 操作按钮 -->
          <div class="grid grid-cols-3 gap-2">
            <button
              @click="emit('adjust', entry); close()"
              class="flex flex-col items-center gap-1.5 py-3 rounded-2xl border border-border hover:bg-accent transition-colors cursor-pointer"
            >
              <DollarSign class="w-5 h-5 text-muted-foreground" />
              <span class="text-xs font-medium text-muted-foreground">调整价值</span>
            </button>
            <button
              @click="emit('edit', entry); close()"
              class="flex flex-col items-center gap-1.5 py-3 rounded-2xl border border-border hover:bg-accent transition-colors cursor-pointer"
            >
              <Edit2 class="w-5 h-5 text-muted-foreground" />
              <span class="text-xs font-medium text-muted-foreground">编辑</span>
            </button>
            <button
              @click="emit('delete', entry); close()"
              class="flex flex-col items-center gap-1.5 py-3 rounded-2xl border border-destructive/20 bg-destructive/5 hover:bg-destructive/10 transition-colors cursor-pointer"
            >
              <Trash2 class="w-5 h-5 text-destructive" />
              <span class="text-xs font-medium text-destructive">删除</span>
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
