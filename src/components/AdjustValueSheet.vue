<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import Sheet from "@/components/ui/Sheet.vue";
import Input from "@/components/ui/Input.vue";
import Label from "@/components/ui/Label.vue";
import Button from "@/components/ui/Button.vue";
import { useEntryStore } from "@/stores/entries";
import type { Entry } from "@/api/entries";
import { TrendingUp, TrendingDown, Clock } from "lucide-vue-next";

const props = defineProps<{
  open?: boolean;
  entry?: Entry;
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
  success: [];
}>();

const entryStore = useEntryStore();

const newValue = ref(0);
const reason = ref("");
const loading = ref(false);

watch(() => props.entry, (entry) => {
  if (entry) {
    newValue.value = entry.value;
    reason.value = "";
    if (props.open && !entryStore.adjustments[entry.id]) {
      entryStore.fetchAdjustments(entry.id);
    }
  }
});

watch(() => props.open, (val) => {
  if (val && props.entry) {
    newValue.value = props.entry.value;
    reason.value = "";
    if (!entryStore.adjustments[props.entry.id]) {
      entryStore.fetchAdjustments(props.entry.id);
    }
  }
});

async function handleAdjust() {
  if (!props.entry) return;
  if (newValue.value === props.entry.value) {
    alert("新价值与当前价值相同");
    return;
  }
  loading.value = true;
  try {
    await entryStore.adjustValue(props.entry.id, newValue.value, reason.value.trim() || undefined);
    emit("update:open", false);
    emit("success");
  } catch (e: any) {
    alert(e?.toString() || "调整失败");
  } finally {
    loading.value = false;
  }
}

function fmt(v: number) {
  return v.toLocaleString("zh-CN", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

function formatDate(iso: string) {
  return new Date(iso).toLocaleString("zh-CN", { 
    year: "numeric", 
    month: "2-digit", 
    day: "2-digit", 
    hour: "2-digit", 
    minute: "2-digit" 
  });
}
</script>

<template>
  <Sheet :open="open" @update:open="emit('update:open', $event)" :title="`调整价值 · ${entry?.name}`">
    <div class="space-y-6">
      <!-- 当前价值 -->
      <div class="rounded-xl border border-border bg-muted/30 p-4">
        <p class="text-xs text-muted-foreground mb-1">当前价值</p>
        <p class="text-2xl font-bold">¥ {{ entry ? fmt(entry.value) : "0.00" }}</p>
      </div>

      <!-- 新价值 -->
      <div class="space-y-2">
        <Label for="new-value">新价值</Label>
        <Input id="new-value" v-model.number="newValue" type="number" step="0.01" />
        <p v-if="entry && newValue !== entry.value" class="text-xs flex items-center gap-1"
          :class="newValue > entry.value ? 'text-success' : 'text-destructive'"
        >
          <TrendingUp v-if="newValue > entry.value" class="w-3.5 h-3.5" />
          <TrendingDown v-else class="w-3.5 h-3.5" />
          {{ newValue > entry.value ? '+' : '' }}{{ fmt(newValue - (entry?.value || 0)) }}
        </p>
      </div>

      <!-- 原因 -->
      <div class="space-y-2">
        <Label for="reason">调整原因（可选）</Label>
        <textarea
          id="reason"
          v-model="reason"
          rows="3"
          placeholder="如：盘点差异、估值修正等"
          class="w-full rounded-lg border border-input bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring resize-none"
        />
      </div>

      <!-- 历史记录 -->
      <div v-if="entry && entryStore.adjustments[entry.id]?.length" class="space-y-2">
        <p class="text-sm font-medium flex items-center gap-2">
          <Clock class="w-4 h-4" />
          调整历史
        </p>
        <div class="space-y-2 max-h-64 overflow-y-auto">
          <div
            v-for="adj in entryStore.adjustments[entry.id]"
            :key="adj.id"
            class="text-xs border border-border rounded-lg p-3 space-y-1"
          >
            <div class="flex items-center justify-between">
              <span class="text-muted-foreground">{{ formatDate(adj.adjustedAt) }}</span>
              <span
                :class="adj.newValue > adj.oldValue ? 'text-success' : 'text-destructive'"
                class="font-medium"
              >
                {{ adj.newValue > adj.oldValue ? '+' : '' }}{{ fmt(adj.newValue - adj.oldValue) }}
              </span>
            </div>
            <div class="flex items-center gap-2 text-muted-foreground">
              <span>¥ {{ fmt(adj.oldValue) }}</span>
              <span>→</span>
              <span>¥ {{ fmt(adj.newValue) }}</span>
            </div>
            <p v-if="adj.reason" class="text-foreground">{{ adj.reason }}</p>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex gap-2">
        <Button variant="outline" class="flex-1" @click="emit('update:open', false)">
          取消
        </Button>
        <Button class="flex-1" :disabled="loading || (entry && newValue === entry.value)" @click="handleAdjust">
          {{ loading ? "调整中..." : "确认调整" }}
        </Button>
      </div>
    </template>
  </Sheet>
</template>
