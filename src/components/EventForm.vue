<script setup lang="ts">
import { ref, watch } from "vue";
import { eventApi, type Event, type CreateEventParams, type UpdateEventParams } from "@/api/records";
import Sheet from "@/components/ui/Sheet.vue";
import Button from "@/components/ui/Button.vue";
import Input from "@/components/ui/Input.vue";
import Label from "@/components/ui/Label.vue";

const props = defineProps<{
  open: boolean;
  bookId: string;
  event: Event | null; // null = 新建，非null = 编辑
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
  success: [];
}>();

const name = ref("");
const description = ref("");
const loading = ref(false);
const errorMsg = ref("");

// 打开时同步数据
watch(() => props.open, (val) => {
  if (!val) return;
  errorMsg.value = "";
  if (props.event) {
    name.value = props.event.name;
    description.value = props.event.description ?? "";
  } else {
    name.value = "";
    description.value = "";
  }
});

async function handleSubmit() {
  if (!name.value.trim()) {
    errorMsg.value = "请输入事件名称";
    return;
  }
  loading.value = true;
  errorMsg.value = "";
  try {
    if (props.event) {
      const params: UpdateEventParams = {
        name: name.value.trim(),
        description: description.value.trim() || null,
      };
      await eventApi.update(props.event.id, params);
    } else {
      const params: CreateEventParams = {
        bookId: props.bookId,
        name: name.value.trim(),
        description: description.value.trim() || undefined,
      };
      await eventApi.create(params);
    }
    emit("success");
    emit("update:open", false);
  } catch (e: any) {
    errorMsg.value = e?.message ?? "操作失败，请重试";
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <Sheet
    :open="open"
    :title="event ? '编辑事件' : '新增事件'"
    @update:open="emit('update:open', $event)"
  >
    <div class="space-y-5">
      <div>
        <Label for="ev-name" class="mb-1.5 block">事件名称</Label>
        <Input
          id="ev-name"
          v-model="name"
          placeholder="如：旅行支出、婚礼筹备、年终采购…"
          @keyup.enter="handleSubmit"
        />
      </div>
      <div>
        <Label for="ev-desc" class="mb-1.5 block">
          描述
          <span class="text-muted-foreground font-normal text-xs ml-1">可选</span>
        </Label>
        <Input
          id="ev-desc"
          v-model="description"
          placeholder="简短说明这个事件的背景"
        />
      </div>
      <p v-if="errorMsg" class="text-sm text-destructive">{{ errorMsg }}</p>
    </div>

    <template #footer>
      <div class="flex gap-3">
        <Button variant="outline" class="flex-1" @click="emit('update:open', false)">取消</Button>
        <Button class="flex-1" :loading="loading" @click="handleSubmit">
          {{ event ? '保存' : '创建' }}
        </Button>
      </div>
    </template>
  </Sheet>
</template>
