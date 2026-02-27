<script setup lang="ts">
import {
  ArrowDownCircle, ArrowUpCircle, ArrowLeftRight,
  Tag, FileText, Wallet, X, Edit2, Trash2,
  CalendarDays, Link2,
} from "lucide-vue-next";
import type { FlowRecord, RecordType } from "@/api/records";

defineProps<{
  open?: boolean;
  record?: FlowRecord;
  categoryName?: string;
  eventName?: string;
  fromAccountName?: string;
  toAccountName?: string;
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
  edit: [record: FlowRecord];
  delete: [record: FlowRecord];
}>();

function close() { emit("update:open", false); }

function fmt(v: number) {
  return v.toLocaleString("zh-CN", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

function fmtDate(iso: string) {
  return new Date(iso.slice(0, 10)).toLocaleDateString("zh-CN", {
    year: "numeric", month: "long", day: "numeric", weekday: "long",
  });
}

const typeConfig: Record<RecordType, { label: string; icon: typeof ArrowDownCircle; color: string; bg: string; amountPrefix: string }> = {
  income:   { label: "收入",   icon: ArrowDownCircle,  color: "text-emerald-500", bg: "bg-emerald-500/10", amountPrefix: "+" },
  expense:  { label: "支出",   icon: ArrowUpCircle,    color: "text-rose-500",    bg: "bg-rose-500/10",    amountPrefix: "-" },
  transfer: { label: "转账",   icon: ArrowLeftRight,   color: "text-sky-500",     bg: "bg-sky-500/10",     amountPrefix: "" },
  inflow:   { label: "流入",   icon: ArrowDownCircle,  color: "text-emerald-500", bg: "bg-emerald-500/10", amountPrefix: "+" },
  outflow:  { label: "流出",   icon: ArrowUpCircle,    color: "text-rose-500",    bg: "bg-rose-500/10",    amountPrefix: "-" },
};
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
        v-if="open && record"
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

        <div class="px-6 pt-2 pb-6" v-if="record">
          <!-- 顶部：图标 + 金额 + 类型 -->
          <div class="flex items-center gap-4 mb-6">
            <div
              class="w-14 h-14 rounded-2xl flex items-center justify-center shrink-0"
              :class="typeConfig[record.type].bg"
            >
              <component
                :is="typeConfig[record.type].icon"
                class="w-7 h-7"
                :class="typeConfig[record.type].color"
              />
            </div>
            <div>
              <div class="flex items-center gap-2 mb-1">
                <span
                  class="text-xs font-medium px-2.5 py-0.5 rounded-full"
                  :class="[typeConfig[record.type].bg, typeConfig[record.type].color]"
                >
                  {{ typeConfig[record.type].label }}
                </span>
              </div>
              <p
                class="text-3xl font-bold tabular-nums leading-none"
                :class="typeConfig[record.type].color"
              >
                <span class="text-xl font-semibold mr-0.5">{{ typeConfig[record.type].amountPrefix }}¥</span>{{ fmt(record.amount) }}
              </p>
            </div>
          </div>

          <!-- 摘要 / 备注 -->
          <div v-if="record.note" class="rounded-2xl bg-muted/50 px-4 py-3.5 mb-4 flex items-start gap-2.5">
            <FileText class="w-4 h-4 text-muted-foreground mt-0.5 shrink-0" />
            <p class="text-sm text-foreground leading-relaxed">{{ record.note }}</p>
          </div>

          <!-- 详情列表 -->
          <div class="rounded-2xl border border-border bg-background divide-y divide-border/60 mb-6">
            <!-- 日期 -->
            <div class="flex items-center justify-between px-4 py-3">
              <span class="text-sm text-muted-foreground flex items-center gap-2">
                <CalendarDays class="w-3.5 h-3.5" />
                日期
              </span>
              <span class="text-sm font-medium">{{ fmtDate(record.happenedAt) }}</span>
            </div>
            <!-- 分类 -->
            <div v-if="categoryName" class="flex items-center justify-between px-4 py-3">
              <span class="text-sm text-muted-foreground flex items-center gap-2">
                <Tag class="w-3.5 h-3.5" />
                分类
              </span>
              <span
                class="text-xs font-medium px-2.5 py-1 rounded-full"
                :class="[typeConfig[record.type].bg, typeConfig[record.type].color]"
              >{{ categoryName }}</span>
            </div>
            <!-- 收款账户（收入） -->
            <div v-if="toAccountName" class="flex items-center justify-between px-4 py-3">
              <span class="text-sm text-muted-foreground flex items-center gap-2">
                <Wallet class="w-3.5 h-3.5" />
                收款账户
              </span>
              <span class="text-sm font-medium">{{ toAccountName }}</span>
            </div>
            <!-- 付款账户（支出） -->
            <div v-if="fromAccountName" class="flex items-center justify-between px-4 py-3">
              <span class="text-sm text-muted-foreground flex items-center gap-2">
                <Wallet class="w-3.5 h-3.5" />
                付款账户
              </span>
              <span class="text-sm font-medium">{{ fromAccountName }}</span>
            </div>
            <!-- 事件 -->
            <div v-if="eventName" class="flex items-center justify-between px-4 py-3">
              <span class="text-sm text-muted-foreground flex items-center gap-2">
                <Link2 class="w-3.5 h-3.5" />
                所属事件
              </span>
              <span class="text-sm font-medium">{{ eventName }}</span>
            </div>
          </div>

          <!-- 操作按钮 -->
          <div class="grid grid-cols-2 gap-2">
            <button
              @click="emit('edit', record); close()"
              class="flex items-center justify-center gap-2 py-3.5 rounded-2xl border border-border hover:bg-accent transition-colors cursor-pointer"
            >
              <Edit2 class="w-4 h-4 text-muted-foreground" />
              <span class="text-sm font-medium">编辑</span>
            </button>
            <button
              @click="emit('delete', record); close()"
              class="flex items-center justify-center gap-2 py-3.5 rounded-2xl border border-destructive/20 bg-destructive/5 hover:bg-destructive/10 transition-colors cursor-pointer"
            >
              <Trash2 class="w-4 h-4 text-destructive" />
              <span class="text-sm font-medium text-destructive">删除</span>
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
