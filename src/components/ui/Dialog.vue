<script setup lang="ts">
defineProps<{
  open: boolean;
  title?: string;
  description?: string;
  maxWidth?: 'sm' | 'md' | 'lg' | 'xl' | '2xl';
}>();

const emit = defineEmits<{
  'update:open': [value: boolean];
}>();

const maxWidthClasses = {
  sm: 'max-w-sm',
  md: 'max-w-md',
  lg: 'max-w-lg',
  xl: 'max-w-xl',
  '2xl': 'max-w-2xl',
};

function close() {
  emit('update:open', false);
}
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition-smooth"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-smooth"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div 
        v-if="open"
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4"
        @click.self="close"
      >
        <div 
          class="bg-card rounded-2xl w-full shadow-2xl border border-border animate-in"
          :class="maxWidthClasses[maxWidth || 'md']"
        >
          <div v-if="title || $slots.header" class="px-6 pt-6 pb-4">
            <slot name="header">
              <h2 v-if="title" class="text-xl font-bold mb-1">{{ title }}</h2>
              <p v-if="description" class="text-sm text-muted-foreground">{{ description }}</p>
            </slot>
          </div>
          
          <div class="px-6 pb-6">
            <slot />
          </div>
          
          <div v-if="$slots.footer" class="px-6 pb-6 pt-2 border-t border-border">
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
