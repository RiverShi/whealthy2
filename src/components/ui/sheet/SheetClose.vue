<script setup lang="ts">
import { inject } from "vue";

defineProps<{
  asChild?: boolean;
}>();

const setOpen = inject<(value: boolean) => void>("setSheetOpen");

function handleClick(e: Event) {
  setOpen?.(false);
}
</script>

<template>
  <div v-if="asChild" @click="handleClick">
    <slot />
  </div>
  <button v-else @click="handleClick">
    <slot />
  </button>
</template>
