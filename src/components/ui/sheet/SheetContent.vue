<script setup lang="ts">
import { inject, type Ref } from "vue";

defineProps<{
  side?: "left" | "right" | "top" | "bottom";
  class?: string;
}>();

const isOpen = inject<Ref<boolean>>("sheetOpen");
const setOpen = inject<(value: boolean) => void>("setSheetOpen");

function close() {
  setOpen?.(false);
}
</script>

<template>
  <Teleport to="body">
    <!-- 遮罩层 -->
    <Transition
      enter-active-class="transition-opacity duration-200"
      enter-from-class="opacity-0"
      leave-active-class="transition-opacity duration-200"
      leave-to-class="opacity-0"
    >
      <div
        v-if="isOpen"
        class="fixed inset-0 z-50 bg-black/50"
        @click="close"
      />
    </Transition>

    <!-- Sheet内容 -->
    <Transition
      enter-active-class="transition-transform duration-300"
      :enter-from-class="side === 'left' ? '-translate-x-full' : side === 'top' ? '-translate-y-full' : side === 'bottom' ? 'translate-y-full' : 'translate-x-full'"
      enter-to-class="translate-x-0 translate-y-0"
      leave-active-class="transition-transform duration-300"
      :leave-to-class="side === 'left' ? '-translate-x-full' : side === 'top' ? '-translate-y-full' : side === 'bottom' ? 'translate-y-full' : 'translate-x-full'"
    >
      <div
        v-if="isOpen"
        :class="[
          'fixed z-50 bg-background border shadow-2xl flex flex-col',
          side === 'right' ? 'right-0 top-0 bottom-0 border-l' : '',
          side === 'left' ? 'left-0 top-0 bottom-0 border-r' : '',
          side === 'top' ? 'top-0 left-0 right-0 border-b' : '',
          side === 'bottom' ? 'bottom-0 left-0 right-0 border-t' : '',
          $props.class
        ]"
      >
        <slot />
      </div>
    </Transition>
  </Teleport>
</template>
