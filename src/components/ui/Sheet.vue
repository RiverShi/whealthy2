<script setup lang="ts">
import { Transition, ref, watch } from "vue";
import { X } from "lucide-vue-next";

const props = defineProps<{
  open?: boolean;
  title?: string;
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
}>();

const localOpen = ref(props.open);
watch(() => props.open, (val) => (localOpen.value = val));

function close() {
  localOpen.value = false;
  emit("update:open", false);
}
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition-opacity duration-200"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-opacity duration-200"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="localOpen"
        class="fixed inset-0 z-50 bg-black/50"
        @click="close"
      />
    </Transition>

    <Transition
      enter-active-class="transition-transform duration-300"
      enter-from-class="translate-x-full"
      enter-to-class="translate-x-0"
      leave-active-class="transition-transform duration-300"
      leave-from-class="translate-x-0"
      leave-to-class="translate-x-full"
    >
      <div
        v-if="localOpen"
        class="fixed right-0 top-0 bottom-0 z-50 w-full max-w-lg bg-card border-l border-border shadow-2xl flex flex-col"
      >
        <!-- Header -->
        <div class="flex items-center justify-between px-6 py-4 border-b border-border">
          <h2 class="text-lg font-semibold">{{ title }}</h2>
          <button
            @click="close"
            class="rounded-lg p-2 hover:bg-accent transition-colors"
          >
            <X class="w-5 h-5" />
          </button>
        </div>

        <!-- Content -->
        <div class="flex-1 overflow-y-auto px-6 py-4">
          <slot />
        </div>

        <!-- Footer -->
        <div v-if="$slots.footer" class="px-6 py-4 border-t border-border">
          <slot name="footer" />
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
