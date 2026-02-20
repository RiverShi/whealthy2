<script setup lang="ts">
import { provide, ref, watch } from "vue";

const props = defineProps<{
  open?: boolean;
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
}>();

const isOpen = ref(props.open ?? false);

watch(() => props.open, (val) => {
  isOpen.value = val ?? false;
});

function setOpen(value: boolean) {
  isOpen.value = value;
  emit("update:open", value);
}

provide("sheetOpen", isOpen);
provide("setSheetOpen", setOpen);
</script>

<template>
  <slot />
</template>
