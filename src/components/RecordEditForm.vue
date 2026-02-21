<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { recordApi, eventApi, type FlowRecord, type Event, type UpdateRecordParams, type RecordType } from "@/api/records";
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
  record: FlowRecord | null;
  events: Event[];
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
  success: [];
}>();

const entryStore = useEntryStore();
const categoryStore = useCategoryStore();

const type = ref<RecordType>("expense");
const name = ref("");
const amount = ref("");
const happenedAt = ref("");
const categoryId = ref("");
const eventId = ref("");
const fromAccountId = ref("");
const toAccountId = ref("");
const note = ref("");
const loading = ref(false);
const errorMsg = ref("");

// 内联新建事件
const showNewEvent = ref(false);
const newEventName = ref("");
const creatingEvent = ref(false);
const localEvents = ref<Event[]>([]);

watch(() => props.events, (v) => { localEvents.value = [...v]; }, { immediate: true });

async function createEventInline() {
  if (!newEventName.value.trim() || creatingEvent.value) return;
  creatingEvent.value = true;
  try {
    const ev = await eventApi.create({ bookId: props.bookId, name: newEventName.value.trim() });
    localEvents.value.push(ev);
    eventId.value = ev.id;
    newEventName.value = "";
    showNewEvent.value = false;
  } finally {
    creatingEvent.value = false;
  }
}

// 同步 record 数据到表单
watch(() => props.open, async (val) => {
  if (!val || !props.record) return;
  errorMsg.value = "";
  showNewEvent.value = false;
  const r = props.record;
  type.value = r.type;
  name.value = r.name ?? "";
  amount.value = String(r.amount);
  happenedAt.value = r.happenedAt.split("T")[0];
  categoryId.value = r.categoryId ?? "";
  eventId.value = r.eventId ?? "";
  fromAccountId.value = r.fromAccountId ?? "";
  toAccountId.value = r.toAccountId ?? "";
  note.value = r.note ?? "";

  if (entryStore.entries.length === 0) {
    await entryStore.fetchEntries(props.bookId);
  }
  await Promise.all([
    categoryStore.fetchCategories("income"),
    categoryStore.fetchCategories("expense"),
  ]);
});

// 切换类型时重置
watch(type, () => {
  categoryId.value = "";
  fromAccountId.value = "";
  toAccountId.value = "";
});

const accounts = computed(() =>
  entryStore.entries.filter((e) => e.isAccount && !e.closedAt)
);
const categoryDomain = computed(() =>
  type.value === "income" ? ("income" as const) : ("expense" as const)
);
const showCategory = computed(() => type.value !== "transfer");
const showFromAccount = computed(() => type.value === "expense" || type.value === "transfer");
const showToAccount = computed(() => type.value === "income" || type.value === "transfer");

async function handleSubmit() {
  const amt = parseFloat(amount.value);
  if (!amount.value || isNaN(amt) || amt <= 0) {
    errorMsg.value = "请输入有效金额";
    return;
  }
  if (!props.record) return;

  const params: UpdateRecordParams = {
    type: type.value,
    name: name.value.trim() || null,
    amount: amt,
    happenedAt: happenedAt.value,
    categoryId: categoryId.value || null,
    eventId: eventId.value || null,
    fromAccountId: fromAccountId.value || null,
    toAccountId: toAccountId.value || null,
    note: note.value || null,
  };

  loading.value = true;
  errorMsg.value = "";
  try {
    await recordApi.update(props.record.id, params);
    emit("success");
    emit("update:open", false);
  } catch (e: any) {
    errorMsg.value = e?.message ?? "保存失败，请重试";
  } finally {
    loading.value = false;
  }
}

const typeOptions: { value: RecordType; label: string; color: string }[] = [
  { value: "expense", label: "支出", color: "text-rose-500" },
  { value: "income", label: "收入", color: "text-emerald-500" },
  { value: "transfer", label: "不计收支", color: "text-muted-foreground" },
];

const selectClass = "flex h-11 w-full rounded-xl border border-input bg-background px-4 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring";
</script>

<template>
  <Sheet :open="open" title="编辑流水" @update:open="emit('update:open', $event)">
    <div v-if="record" class="space-y-5">
      <!-- 类型 -->
      <div>
        <Label class="mb-2 block">类型</Label>
        <div class="flex gap-2">
          <button
            v-for="opt in typeOptions"
            :key="opt.value"
            @click="type = opt.value"
            class="flex-1 py-2.5 rounded-xl border text-sm font-medium transition-all"
            :class="type === opt.value
              ? 'border-primary bg-primary/10 ' + opt.color
              : 'border-border text-muted-foreground hover:border-primary/40'"
          >{{ opt.label }}</button>
        </div>
      </div>

      <!-- 关联事件 -->
      <div>
        <div class="flex items-center justify-between mb-1.5">
          <Label for="edit-event">
            关联事件
            <span class="text-muted-foreground font-normal text-xs ml-1">可选</span>
          </Label>
          <button
            type="button"
            @click="showNewEvent = !showNewEvent; newEventName = ''"
            class="text-xs text-primary hover:underline"
          >{{ showNewEvent ? '取消' : '＋ 新建事件' }}</button>
        </div>
        <div v-if="showNewEvent" class="flex gap-2 mb-2">
          <input
            v-model="newEventName"
            placeholder="事件名称"
            autofocus
            @keyup.enter="createEventInline"
            class="flex-1 h-10 rounded-xl border border-input bg-background px-3 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          />
          <button
            type="button"
            :disabled="!newEventName.trim() || creatingEvent"
            @click="createEventInline"
            class="px-4 h-10 rounded-xl bg-primary text-primary-foreground text-sm font-medium disabled:opacity-40 shrink-0"
          >{{ creatingEvent ? '创建中…' : '创建' }}</button>
        </div>
        <select id="edit-event" v-model="eventId" :class="selectClass">
          <option value="">— 不关联事件 —</option>
          <option v-for="ev in localEvents" :key="ev.id" :value="ev.id">{{ ev.name }}</option>
        </select>
      </div>

      <!-- 名称 -->
      <div>
        <Label for="edit-name" class="mb-1.5 block">
          名称
          <span class="text-muted-foreground font-normal text-xs ml-1">可选，留空自动用分类名</span>
        </Label>
        <Input id="edit-name" v-model="name" placeholder="如：周末聚餐、房租…" />
      </div>

      <!-- 金额 -->
      <div>
        <Label for="edit-amt" class="mb-1.5 block">金额</Label>
        <Input id="edit-amt" v-model="amount" type="number" placeholder="0.00" />
      </div>

      <!-- 日期 -->
      <div>
        <Label for="edit-date" class="mb-1.5 block">日期</Label>
        <input id="edit-date" v-model="happenedAt" type="date" :class="selectClass" />
      </div>

      <!-- 分类 -->
      <div v-if="showCategory">
        <Label class="mb-1.5 block">分类</Label>
        <CategoryPicker
          v-model="categoryId"
          :domain="categoryDomain"
        />
      </div>

      <!-- 转出账户 -->
      <div v-if="showFromAccount">
        <Label for="edit-from" class="mb-1.5 block">
          {{ type === 'transfer' ? '转出账户' : '付款账户' }}
        </Label>
        <select id="edit-from" v-model="fromAccountId" :class="selectClass">
          <option value="">— 不选账户 —</option>
          <option v-for="a in accounts" :key="a.id" :value="a.id">
            {{ a.name }}（¥{{ a.value.toFixed(2) }}）
          </option>
        </select>
      </div>

      <!-- 转入账户 -->
      <div v-if="showToAccount">
        <Label for="edit-to" class="mb-1.5 block">
          {{ type === 'transfer' ? '转入账户' : '收款账户' }}
        </Label>
        <select id="edit-to" v-model="toAccountId" :class="selectClass">
          <option value="">— 不选账户 —</option>
          <option v-for="a in accounts" :key="a.id" :value="a.id">
            {{ a.name }}（¥{{ a.value.toFixed(2) }}）
          </option>
        </select>
      </div>

      <!-- 备注 -->
      <div>
        <Label for="edit-note" class="mb-1.5 block">备注</Label>
        <Input id="edit-note" v-model="note" placeholder="添加备注（可选）" />
      </div>

      <p v-if="errorMsg" class="text-sm text-destructive">{{ errorMsg }}</p>
    </div>

    <template #footer>
      <div class="flex gap-3">
        <Button variant="outline" class="flex-1" @click="emit('update:open', false)">取消</Button>
        <Button class="flex-1" :loading="loading" @click="handleSubmit">保存</Button>
      </div>
    </template>
  </Sheet>
</template>
