<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { recordApi, eventApi, type CreateRecordParams, type RecordType, type Event } from "@/api/records";
import { useEntryStore } from "@/stores/entries";
import { useCategoryStore } from "@/stores/categories";
import Sheet from "@/components/ui/Sheet.vue";
import Button from "@/components/ui/Button.vue";
import Input from "@/components/ui/Input.vue";
import Label from "@/components/ui/Label.vue";
import CategoryPicker from "@/components/ui/CategoryPicker.vue";

const props = defineProps<{
  open: boolean;
  bookId: string;
  presetEventId?: string;
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
  success: [];
}>();

const entryStore = useEntryStore();
const categoryStore = useCategoryStore();

// ── 表单状态 ──────────────────────────────────────────────────────────────────
const type = ref<RecordType>("expense");
const name = ref("");
const amount = ref<string>("");
const happenedAt = ref(new Date().toISOString().slice(0, 10));
const categoryId = ref("");
const eventId = ref("");
const fromAccountId = ref("");
const toAccountId = ref("");
const events = ref<Event[]>([]);
const note = ref("");
const loading = ref(false);
const errorMsg = ref("");

// 内联新建事件
const showNewEvent = ref(false);
const newEventName = ref("");
const creatingEvent = ref(false);

async function createEventInline() {
  if (!newEventName.value.trim() || creatingEvent.value) return;
  creatingEvent.value = true;
  try {
    const ev = await eventApi.create({ bookId: props.bookId, name: newEventName.value.trim() });
    events.value.push(ev);
    eventId.value = ev.id;
    newEventName.value = "";
    showNewEvent.value = false;
  } catch (e) {
    // 静默失败
  } finally {
    creatingEvent.value = false;
  }
}

// ── 计算辅助 ──────────────────────────────────────────────────────────────────
// 只取 isAccount=true 的条目作为账户
const accounts = computed(() =>
  entryStore.entries.filter((e) => e.isAccount && !e.closedAt)
);

const categoryDomain = computed(() =>
  type.value === "income" ? ("income" as const) : ("expense" as const)
);

const showCategory = computed(() => type.value === "income" || type.value === "expense");
const showFromAccount = computed(() => ["expense", "transfer", "outflow"].includes(type.value));
const showToAccount = computed(() => ["income", "transfer", "inflow"].includes(type.value));

// 切换类型时重置无关字段
watch(type, () => {
  categoryId.value = "";
  fromAccountId.value = "";
  toAccountId.value = "";
});

// 打开时加载所需数据
watch(() => props.open, async (val) => {
  if (!val) return;
  errorMsg.value = "";
  // 预填事件
  eventId.value = props.presetEventId ?? "";
  if (entryStore.entries.length === 0) {
    await entryStore.fetchEntries(props.bookId);
  }
  await Promise.all([
    categoryStore.fetchCategories("income"),
    categoryStore.fetchCategories("expense"),
    eventApi.list(props.bookId).then((res) => (events.value = res)),
  ]);
});

// ── 提交 ──────────────────────────────────────────────────────────────────────
async function handleSubmit() {
  const amt = parseFloat(amount.value);
  if (!amount.value || isNaN(amt) || amt <= 0) {
    errorMsg.value = "请输入有效金额";
    return;
  }
  if (!happenedAt.value) {
    errorMsg.value = "请选择日期";
    return;
  }

  const params: CreateRecordParams = {
    bookId: props.bookId,
    type: type.value,
    name: name.value.trim() || undefined,
    eventId: eventId.value || undefined,
    amount: amt,
    happenedAt: happenedAt.value,
    categoryId: categoryId.value || undefined,
    fromAccountId: fromAccountId.value || undefined,
    toAccountId: toAccountId.value || undefined,
    note: note.value || undefined,
  };

  loading.value = true;
  errorMsg.value = "";
  try {
    await recordApi.create(params);
    // 重置表单
    name.value = "";
    amount.value = "";
    note.value = "";
    categoryId.value = "";
    eventId.value = "";
    fromAccountId.value = "";
    toAccountId.value = "";
    happenedAt.value = new Date().toISOString().slice(0, 10);
    showNewEvent.value = false;
    newEventName.value = "";
    emit("success");
    emit("update:open", false);
  } catch (e: any) {
    errorMsg.value = e?.message ?? "创建失败，请重试";
  } finally {
    loading.value = false;
  }
}

const typeOptions: { value: RecordType; label: string; color: string }[] = [
  { value: "expense", label: "支出", color: "text-rose-500" },
  { value: "income", label: "收入", color: "text-emerald-500" },
  { value: "inflow", label: "流入", color: "text-blue-500" },
  { value: "outflow", label: "流出", color: "text-orange-500" },
  { value: "transfer", label: "划转", color: "text-muted-foreground" },
];
</script>

<template>
  <Sheet :open="open" title="新增流水" @update:open="emit('update:open', $event)">
    <div class="space-y-5">
      <!-- 类型选择 -->
      <div>
        <Label class="mb-2 block">类型</Label>
        <div class="grid grid-cols-5 gap-1">
          <button
            v-for="opt in typeOptions"
            :key="opt.value"
            @click="type = opt.value"
            class="py-2 rounded-xl border text-xs font-medium transition-all cursor-pointer"
            :class="type === opt.value
              ? 'border-primary bg-primary/10 ' + opt.color
              : 'border-border text-muted-foreground hover:border-primary/40'"
          >
            {{ opt.label }}
          </button>
        </div>
      </div>

      <!-- 关联事件 -->
      <div>
        <div class="flex items-center justify-between mb-1.5">
          <Label for="event-sel">
            关联事件
            <span class="text-muted-foreground font-normal text-xs ml-1">可选</span>
          </Label>
          <button
            type="button"
            @click="showNewEvent = !showNewEvent; newEventName = ''"
            class="text-xs text-primary hover:underline flex items-center gap-0.5"
          >
            {{ showNewEvent ? '取消' : '＋ 新建事件' }}
          </button>
        </div>
        <div v-if="showNewEvent" class="flex gap-2 mb-2">
          <input
            v-model="newEventName"
            placeholder="事件名称，如：旅行、婚礼…"
            autofocus
            @keyup.enter="createEventInline"
            class="flex-1 h-10 rounded-xl border border-input bg-background px-3 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          />
          <button
            type="button"
            :disabled="!newEventName.trim() || creatingEvent"
            @click="createEventInline"
            class="px-4 h-10 rounded-xl bg-primary text-primary-foreground text-sm font-medium disabled:opacity-40 shrink-0"
          >
            {{ creatingEvent ? '创建中…' : '创建' }}
          </button>
        </div>
        <select
          id="event-sel"
          v-model="eventId"
          class="flex h-11 w-full rounded-xl border border-input bg-background px-4 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <option value="">— 不关联事件 —</option>
          <option v-for="ev in events" :key="ev.id" :value="ev.id">
            {{ ev.name }}
          </option>
        </select>
        <p v-if="events.length === 0 && !showNewEvent" class="text-xs text-muted-foreground mt-1.5">
          暂无事件，点击「＋ 新建事件」创建
        </p>
      </div>

      <!-- 名称（可选，不填时用分类名显示） -->
      <div>
        <Label for="rec-name" class="mb-1.5 block">
          名称
          <span class="text-muted-foreground font-normal text-xs ml-1">可选，留空自动用分类名</span>
        </Label>
        <Input
          id="rec-name"
          v-model="name"
          placeholder="如：周末聚餐、房租…"
        />
      </div>

      <!-- 金额 -->
      <div>
        <Label for="amount" class="mb-1.5 block">金额</Label>
        <Input
          id="amount"
          v-model="amount"
          type="number"
          placeholder="0.00"
          :error="errorMsg && !amount ? '请输入金额' : ''"
        />
      </div>

      <!-- 日期 -->
      <div>
        <Label for="date" class="mb-1.5 block">日期</Label>
        <input
          id="date"
          v-model="happenedAt"
          type="date"
          class="flex h-11 w-full rounded-xl border border-input bg-background px-4 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        />
      </div>

      <!-- 分类（收入/支出时显示） -->
      <div v-if="showCategory">
        <Label class="mb-1.5 block">分类</Label>
        <CategoryPicker
          v-model="categoryId"
          :domain="categoryDomain"
        />
      </div>

      <!-- 转出账户（支出/转账时显示） -->
      <div v-if="showFromAccount">
        <Label for="from-account" class="mb-1.5 block">
          {{ type === "transfer" ? "转出账户" : "付款账户" }}
        </Label>
        <select
          id="from-account"
          v-model="fromAccountId"
          class="flex h-11 w-full rounded-xl border border-input bg-background px-4 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <option value="">— 不选账户 —</option>
          <option v-for="a in accounts" :key="a.id" :value="a.id">
            {{ a.name }}（¥{{ a.value.toFixed(2) }}）
          </option>
        </select>
      </div>

      <!-- 转入账户（收入/转账时显示） -->
      <div v-if="showToAccount">
        <Label for="to-account" class="mb-1.5 block">
          {{ type === "transfer" ? "转入账户" : "收款账户" }}
        </Label>
        <select
          id="to-account"
          v-model="toAccountId"
          class="flex h-11 w-full rounded-xl border border-input bg-background px-4 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <option value="">— 不选账户 —</option>
          <option v-for="a in accounts" :key="a.id" :value="a.id">
            {{ a.name }}（¥{{ a.value.toFixed(2) }}）
          </option>
        </select>
      </div>

      <!-- 备注 -->
      <div>
        <Label for="note" class="mb-1.5 block">备注</Label>
        <Input
          id="note"
          v-model="note"
          placeholder="添加备注（可选）"
        />
      </div>


      <!-- 错误提示 -->
      <p v-if="errorMsg" class="text-sm text-destructive">{{ errorMsg }}</p>
    </div>

    <template #footer>
      <div class="flex gap-3">
        <Button
          variant="outline"
          class="flex-1"
          @click="emit('update:open', false)"
        >
          取消
        </Button>
        <Button
          class="flex-1"
          :loading="loading"
          @click="handleSubmit"
        >
          保存
        </Button>
      </div>
    </template>
  </Sheet>
</template>
