<script setup lang="ts">
import { ref, reactive, watch } from "vue";
import Sheet from "@/components/ui/Sheet.vue";
import Input from "@/components/ui/Input.vue";
import Label from "@/components/ui/Label.vue";
import Switch from "@/components/ui/Switch.vue";
import Button from "@/components/ui/Button.vue";
import { useEntryStore } from "@/stores/entries";
import { useCategoryStore } from "@/stores/categories";
import CategoryPicker from "@/components/ui/CategoryPicker.vue";
import type { Entry, EntryKind, CreateEntryParams } from "@/api/entries";

const props = defineProps<{
  open?: boolean;
  bookId: string;
  entry?: Entry;
  defaultKind?: EntryKind;
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
  success: [];
}>();

const entryStore = useEntryStore();
const categoryStore = useCategoryStore();

const form = reactive<CreateEntryParams>({
  bookId: props.bookId,
  name: "",
  kind: props.defaultKind || "asset",
  isAccount: false,
  value: 0,
  valuationType: "manual",
  categoryL1Id: undefined,
  categoryL2Id: undefined,
  tagIds: [],
  extra: undefined,
});

const loading = ref(false);

// 分类选择回调（同时设置 L1 和 L2）
function handleCategorySelect(payload: { l1Id: string; l2Id: string }) {
  form.categoryL1Id = payload.l1Id || undefined;
  form.categoryL2Id = payload.l2Id || undefined;
}

watch(() => props.open, (val) => {
  if (val) {
    if (props.entry) {
      Object.assign(form, {
        bookId: props.entry.bookId,
        name: props.entry.name,
        kind: props.entry.kind,
        isAccount: props.entry.isAccount,
        value: props.entry.value,
        valuationType: props.entry.valuationType,
        categoryL1Id: props.entry.categoryL1Id,
        categoryL2Id: props.entry.categoryL2Id,
        tagIds: props.entry.tagIds,
        extra: props.entry.extra,
      });
    } else {
      form.name = "";
      form.kind = props.defaultKind || "asset";
      form.isAccount = false;
      form.value = 0;
      form.valuationType = "manual";
      form.categoryL1Id = undefined;
      form.categoryL2Id = undefined;
      form.tagIds = [];
      form.extra = undefined;
    }
    // 加载分类
    const domain = form.kind === "asset" ? "asset" : "liability";
    const hasCats = categoryStore.categories.some(c => c.domain === domain);
    if (!hasCats) {
      categoryStore.fetchCategories(domain);
    }
  }
});

async function handleSubmit() {
  if (!form.name.trim()) {
    alert("请输入名称");
    return;
  }
  loading.value = true;
  try {
    if (props.entry) {
      await entryStore.updateEntry(props.entry.id, form);
    } else {
      await entryStore.createEntry(form);
    }
    emit("update:open", false);
    emit("success");
  } catch (e: any) {
    alert(e?.toString() || "操作失败");
  } finally {
    loading.value = false;
  }
}

function handleKindChange() {
  form.categoryL1Id = undefined;
  form.categoryL2Id = undefined;
}
</script>

<template>
  <Sheet :open="open" @update:open="emit('update:open', $event)" :title="entry ? '编辑条目' : '新建条目'">
    <div class="space-y-5">
      <!-- 名称 -->
      <div class="space-y-2">
        <Label for="name">名称</Label>
        <Input id="name" v-model="form.name" placeholder="如：招商银行储蓄卡" />
      </div>

      <!-- 类型 -->
      <div class="space-y-2">
        <Label>类型</Label>
        <div class="flex gap-2">
          <button
            @click="form.kind = 'asset'; handleKindChange()"
            :class="form.kind === 'asset' ? 'bg-success/10 text-success border-success' : 'bg-background border-border'"
            class="flex-1 py-2 px-4 rounded-lg border text-sm font-medium transition-colors"
          >
            资产
          </button>
          <button
            @click="form.kind = 'liability'; handleKindChange()"
            :class="form.kind === 'liability' ? 'bg-destructive/10 text-destructive border-destructive' : 'bg-background border-border'"
            class="flex-1 py-2 px-4 rounded-lg border text-sm font-medium transition-colors"
          >
            负债
          </button>
        </div>
      </div>

      <!-- 价值 -->
      <div class="space-y-2">
        <Label for="value">当前价值</Label>
        <Input id="value" v-model.number="form.value" type="number" step="0.01" placeholder="0.00" />
      </div>

      <!-- 估值方式 -->
      <div class="space-y-2">
        <Label>估值方式</Label>
        <select
          v-model="form.valuationType"
          class="w-full h-10 rounded-lg border border-input bg-background px-3 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
        >
          <option value="fixed">固定值</option>
          <option value="manual">手动估值</option>
        </select>
      </div>

      <!-- 是否账户 -->
      <div class="flex items-center justify-between">
        <div>
          <Label>作为记账账户</Label>
          <p class="text-xs text-muted-foreground mt-1">启用后可在记账时关联此条目</p>
        </div>
        <Switch v-model="form.isAccount" />
      </div>

      <!-- 分类 -->
      <div class="space-y-2">
        <Label>分类</Label>
        <CategoryPicker
          :model-value="form.categoryL2Id ?? form.categoryL1Id ?? ''"
          :domain="form.kind === 'asset' ? 'asset' : 'liability'"
          @select="handleCategorySelect"
        />
      </div>
    </div>

    <template #footer>
      <div class="flex gap-2">
        <Button variant="outline" class="flex-1" @click="emit('update:open', false)">
          取消
        </Button>
        <Button class="flex-1" :disabled="loading" @click="handleSubmit">
          {{ loading ? "保存中..." : "保存" }}
        </Button>
      </div>
    </template>
  </Sheet>
</template>
