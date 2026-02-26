<script setup lang="ts">
import { ref, computed } from "vue";
import { transactionApi, type CreateTransactionParams } from "@/api/transactions";
import { entryApi, type Entry } from "@/api/entries";
import { useCategoryStore } from "@/stores/categories";
import { Plus, Trash2 } from "lucide-vue-next";
import Button from "@/components/ui/Button.vue";
import Input from "@/components/ui/Input.vue";
import Label from "@/components/ui/Label.vue";
import {
  Sheet,
  SheetClose,
  SheetContent,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";

const props = defineProps<{
  bookId: string;
  open: boolean;
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
  success: [];
}>();

const categoryStore = useCategoryStore();

const loading = ref(false);
const transactionType = ref<"income" | "expense">("expense");
const happenedAt = ref(new Date().toISOString().slice(0, 16));

// 事项列表（每个事项可以有多个条目）
const matters = ref<{
  id: string;
  description: string;
  items: {
    amount: string;
    categoryId: string;
    accountId: string;
    note: string;
  }[];
}[]>([]);

// 无事项的独立条目
const standaloneItems = ref<{
  amount: string;
  categoryId: string;
  accountId: string;
  note: string;
}[]>([
  {
    amount: "",
    categoryId: "",
    accountId: "",
    note: "",
  },
]);

const entries = ref<Entry[]>([]);

// 加载账户和分类
async function loadData() {
  try {
    entries.value = await entryApi.list(props.bookId);
    await categoryStore.fetchCategories();
    console.log("加载的条目数量:", entries.value.length, entries.value);
  } catch (e) {
    console.error("加载数据失败", e);
  }
}

// 可用作账户的条目（资产和负债）
const accountEntries = computed(() =>
  entries.value.filter((e) => e.kind === "asset" || e.kind === "liability")
);

// 收入类分类
const incomeCategories = computed(() =>
  categoryStore.categories.filter((c) => c.domain === "income")
);

// 支出类分类
const expenseCategories = computed(() =>
  categoryStore.categories.filter((c) => c.domain === "expense")
);

// 当前类型的分类
const currentCategories = computed(() =>
  transactionType.value === "income" ? incomeCategories.value : expenseCategories.value
);

function addMatter() {
  matters.value.push({
    id: Date.now().toString(),
    description: "",
    items: [
      {
        amount: "",
        categoryId: "",
        accountId: "",
        note: "",
      },
    ],
  });
}

function removeMatter(index: number) {
  matters.value.splice(index, 1);
}

function addItemToMatter(matterIndex: number) {
  matters.value[matterIndex].items.push({
    amount: "",
    categoryId: "",
    accountId: "",
    note: "",
  });
}

function removeItemFromMatter(matterIndex: number, itemIndex: number) {
  matters.value[matterIndex].items.splice(itemIndex, 1);
}

function addStandaloneItem() {
  standaloneItems.value.push({
    amount: "",
    categoryId: "",
    accountId: "",
    note: "",
  });
}

function removeStandaloneItem(index: number) {
  standaloneItems.value.splice(index, 1);
}

async function handleSubmit() {
  loading.value = true;
  try {
    const allItems: any[] = [];
    
    // 收集事项中的条目
    matters.value.forEach((matter) => {
      matter.items.forEach((item) => {
        allItems.push({
          type: transactionType.value,
          amount: parseFloat(item.amount),
          categoryId: item.categoryId || undefined,
          fromAccountId: transactionType.value === "expense" ? item.accountId : undefined,
          toAccountId: transactionType.value === "income" ? item.accountId : undefined,
          note: matter.description ? `${matter.description}${item.note ? ' - ' + item.note : ''}` : item.note || undefined,
        });
      });
    });
    
    // 收集独立条目
    standaloneItems.value.forEach((item) => {
      allItems.push({
        type: transactionType.value,
        amount: parseFloat(item.amount),
        categoryId: item.categoryId || undefined,
        fromAccountId: transactionType.value === "expense" ? item.accountId : undefined,
        toAccountId: transactionType.value === "income" ? item.accountId : undefined,
        note: item.note || undefined,
      });
    });

    const params: CreateTransactionParams = {
      bookId: props.bookId,
      happenedAt: new Date(happenedAt.value).toISOString(),
      items: allItems,
    };
    
    await transactionApi.create(params);
    emit("success");
    emit("update:open", false);
    resetForm();
  } catch (e: any) {
    alert(e?.message || "创建失败");
  } finally {
    loading.value = false;
  }
}

function resetForm() {
  transactionType.value = "expense";
  happenedAt.value = new Date().toISOString().slice(0, 16);
  matters.value = [];
  standaloneItems.value = [
    {
      amount: "",
      categoryId: "",
      accountId: "",
      note: "",
    },
  ];
}

function handleOpenChange(open: boolean) {
  emit("update:open", open);
  if (open) {
    loadData();
  }
}
</script>

<template>
  <Sheet :open="open" @update:open="handleOpenChange">
    <SheetContent side="right" class="w-full sm:max-w-lg overflow-y-auto">
      <SheetHeader>
        <SheetTitle>新增</SheetTitle>
        <SheetDescription>记录一笔收入或支出</SheetDescription>
      </SheetHeader>

      <div class="space-y-6 py-6">
        <!-- 基本信息 -->
        <div class="space-y-4">
          <div>
            <Label for="transactionType">类型</Label>
            <select
              id="transactionType"
              v-model="transactionType"
              class="mt-1.5 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
            >
              <option value="income">收入</option>
              <option value="expense">支出</option>
            </select>
          </div>
          <div>
            <Label for="happenedAt">时间</Label>
            <input
              id="happenedAt"
              v-model="happenedAt"
              type="datetime-local"
              class="mt-1.5 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
            />
          </div>
        </div>

        <!-- 事项列表 -->
        <div v-if="matters.length > 0" class="space-y-4">
          <div class="flex items-center justify-between">
            <Label>事项</Label>
          </div>
          
          <div v-for="(matter, mIndex) in matters" :key="matter.id" class="p-4 border border-border rounded-lg space-y-3">
            <div class="flex items-center justify-between">
              <Input v-model="matter.description" placeholder="事项说明" class="flex-1 mr-2" />
              <button
                @click="removeMatter(mIndex)"
                class="text-destructive hover:opacity-70 transition-opacity"
              >
                <Trash2 class="w-4 h-4" />
              </button>
            </div>

            <!-- 事项下的条目 -->
            <div v-for="(item, iIndex) in matter.items" :key="iIndex" class="ml-4 pl-4 border-l-2 border-border space-y-3">
              <div class="flex items-center justify-between mb-2">
                <div class="text-xs text-muted-foreground">条目 {{ iIndex + 1 }}</div>
                <button
                  v-if="matter.items.length > 1"
                  @click="removeItemFromMatter(mIndex, iIndex)"
                  class="text-destructive hover:opacity-70 transition-opacity"
                >
                  <Trash2 class="w-3 h-3" />
                </button>
              </div>

              <div>
                <Label :for="`matter-${mIndex}-item-${iIndex}-amount`">金额</Label>
                <Input :id="`matter-${mIndex}-item-${iIndex}-amount`" v-model="item.amount" type="number" step="0.01" class="mt-1.5" />
              </div>

              <div>
                <Label :for="`matter-${mIndex}-item-${iIndex}-category`">分类</Label>
                <select
                  :id="`matter-${mIndex}-item-${iIndex}-category`"
                  v-model="item.categoryId"
                  class="mt-1.5 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                >
                  <option value="">选择分类</option>
                  <option v-for="cat in currentCategories" :key="cat.id" :value="cat.id">
                    {{ cat.name }}
                  </option>
                </select>
              </div>

              <div>
                <Label :for="`matter-${mIndex}-item-${iIndex}-account`">
                  {{ transactionType === "income" ? "收入账户" : "支出账户" }}
                </Label>
                <select
                  :id="`matter-${mIndex}-item-${iIndex}-account`"
                  v-model="item.accountId"
                  class="mt-1.5 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                >
                  <option value="">选择账户</option>
                  <option v-for="entry in accountEntries" :key="entry.id" :value="entry.id">
                    {{ entry.name }} ({{ entry.kind === 'asset' ? '资产' : '负债' }})
                  </option>
                </select>
              </div>

              <div>
                <Label :for="`matter-${mIndex}-item-${iIndex}-note`">备注</Label>
                <Input :id="`matter-${mIndex}-item-${iIndex}-note`" v-model="item.note" placeholder="选填" class="mt-1.5" />
              </div>
            </div>

            <!-- 为事项添加条目 -->
            <button
              @click="addItemToMatter(mIndex)"
              class="w-full mt-2 py-2 border border-dashed border-border rounded-md text-sm text-muted-foreground hover:border-primary hover:text-primary transition-colors flex items-center justify-center gap-1"
            >
              <Plus class="w-4 h-4" />
              添加条目
            </button>
          </div>
        </div>

        <!-- 无事项的独立条目 -->
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <Label>{{ matters.length > 0 ? '无事项条目' : '条目' }}</Label>
            <Button variant="outline" size="sm" @click="addStandaloneItem">
              <Plus class="w-4 h-4 mr-1" />
              添加条目
            </Button>
          </div>

          <div v-for="(item, index) in standaloneItems" :key="index" class="p-4 border border-border rounded-lg space-y-3">
            <div class="flex items-center justify-between mb-2">
              <div class="text-sm font-medium">条目 {{ index + 1 }}</div>
              <button
                v-if="standaloneItems.length > 1"
                @click="removeStandaloneItem(index)"
                class="text-destructive hover:opacity-70 transition-opacity"
              >
                <Trash2 class="w-4 h-4" />
              </button>
            </div>

            <div>
              <Label :for="`amount-${index}`">金额</Label>
              <Input :id="`amount-${index}`" v-model="item.amount" type="number" step="0.01" class="mt-1.5" />
            </div>

            <div>
              <Label :for="`category-${index}`">分类</Label>
              <select
                :id="`category-${index}`"
                v-model="item.categoryId"
                class="mt-1.5 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
              >
                <option value="">选择分类</option>
                <option v-for="cat in currentCategories" :key="cat.id" :value="cat.id">
                  {{ cat.name }}
                </option>
              </select>
            </div>

            <div>
              <Label :for="`account-${index}`">
                {{ transactionType === "income" ? "收入账户" : "支出账户" }}
              </Label>
              <select
                :id="`account-${index}`"
                v-model="item.accountId"
                class="mt-1.5 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
              >
                <option value="">选择账户</option>
                <option v-for="entry in accountEntries" :key="entry.id" :value="entry.id">
                  {{ entry.name }} ({{ entry.kind === 'asset' ? '资产' : '负债' }})
                </option>
              </select>
            </div>

            <div>
              <Label :for="`note-${index}`">备注</Label>
              <Input :id="`note-${index}`" v-model="item.note" placeholder="选填" class="mt-1.5" />
            </div>
          </div>
        </div>

        <!-- 添加事项按钮 -->
        <button
          @click="addMatter"
          class="w-full py-3 border border-dashed border-border rounded-lg text-sm font-medium text-muted-foreground hover:border-primary hover:text-primary transition-colors flex items-center justify-center gap-2"
        >
          <Plus class="w-5 h-5" />
          添加事项
        </button>
      </div>

      <SheetFooter>
        <SheetClose as-child>
          <Button variant="outline" :disabled="loading">取消</Button>
        </SheetClose>
        <Button @click="handleSubmit" :disabled="loading">
          {{ loading ? "创建中..." : "创建" }}
        </Button>
      </SheetFooter>
    </SheetContent>
  </Sheet>
</template>
