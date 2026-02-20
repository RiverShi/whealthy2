<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { transactionApi, type Transaction } from "@/api/transactions";
import { Plus, Receipt } from "lucide-vue-next";
import TransactionForm from "@/components/TransactionForm.vue";

const route = useRoute();
const router = useRouter();
const bookId = route.params.id as string;

const transactions = ref<Transaction[]>([]);
const loading = ref(false);
const showForm = ref(false);

onMounted(() => {
  fetchTransactions();
});

async function fetchTransactions() {
  loading.value = true;
  try {
    transactions.value = await transactionApi.list(bookId);
  } finally {
    loading.value = false;
  }
}

// 按日期分组
const transactionsByDate = computed(() => {
  const groups = new Map<string, Transaction[]>();
  transactions.value.forEach((t) => {
    const date = t.happenedAt.split("T")[0];
    if (!groups.has(date)) groups.set(date, []);
    groups.get(date)!.push(t);
  });
  return Array.from(groups.entries()).sort((a, b) => b[0].localeCompare(a[0]));
});

function handleCreate() {
  showForm.value = true;
}

function handleViewDetail(id: string) {
  // TODO: 查看交易详情
  console.log("View transaction:", id);
}

function handleSuccess() {
  fetchTransactions();
}
</script>

<template>
  <div class="p-8">
    <!-- 页头 -->
    <div class="flex items-center justify-between mb-8">
      <div>
        <button @click="router.back()" class="text-xs text-muted-foreground hover:text-foreground mb-1 flex items-center gap-1">
          ← 返回账本
        </button>
        <h1 class="text-2xl font-bold">记账流水</h1>
      </div>
      <button
        @click="handleCreate"
        class="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:opacity-90 transition-opacity"
      >
        <Plus class="w-4 h-4" />
        新增
      </button>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="text-center py-16 text-muted-foreground">
      加载中…
    </div>

    <!-- 空状态 -->
    <div v-else-if="!transactions.length" class="text-center py-16">
      <div class="w-14 h-14 rounded-2xl bg-muted flex items-center justify-center mb-4 mx-auto">
        <Receipt class="w-7 h-7 text-muted-foreground" />
      </div>
      <p class="text-base font-medium mb-1">还没有记账记录</p>
      <p class="text-sm text-muted-foreground mb-6">开始记录你的收支流水</p>
      <button
        @click="handleCreate"
        class="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:opacity-90 transition-opacity"
      >
        新增第一笔交易
      </button>
    </div>

    <!-- 交易列表（按日期分组） -->
    <div v-else class="space-y-6">
      <div v-for="[date, dateTransactions] in transactionsByDate" :key="date">
        <div class="text-sm font-medium text-muted-foreground mb-3">
          {{ new Date(date).toLocaleDateString("zh-CN", { month: "long", day: "numeric", weekday: "short" }) }}
        </div>
        <div class="space-y-2">
          <button
            v-for="transaction in dateTransactions"
            :key="transaction.id"
            @click="handleViewDetail(transaction.id)"
            class="w-full group flex items-center gap-4 px-4 py-3 rounded-xl border border-border bg-card hover:border-primary/30 transition-all text-left"
          >
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium truncate">
                {{ transaction.description || "交易记录" }}
              </p>
              <p class="text-xs text-muted-foreground mt-0.5">
                {{ new Date(transaction.happenedAt).toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" }) }}
              </p>
            </div>
          </button>
        </div>
      </div>
    </div>

    <!-- 交易表单 -->
    <TransactionForm :book-id="bookId" v-model:open="showForm" @success="handleSuccess" />
  </div>
</template>
