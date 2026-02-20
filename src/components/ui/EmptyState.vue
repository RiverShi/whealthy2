<script setup lang="ts">
import { type Component } from 'vue';

defineProps<{
  icon?: Component;
  title: string;
  description?: string;
  actionLabel?: string;
}>();

const emit = defineEmits<{
  action: [];
}>();
</script>

<template>
  <div class="flex flex-col items-center justify-center py-16 px-4 animate-in">
    <div v-if="icon" class="relative mb-6">
      <div class="w-20 h-20 rounded-2xl bg-gradient-to-br from-primary/20 to-primary/10 flex items-center justify-center">
        <component :is="icon" class="w-10 h-10 text-primary" />
      </div>
      <div class="absolute -top-2 -right-2 w-6 h-6 rounded-full bg-primary/20 flex items-center justify-center">
        <div class="w-2 h-2 rounded-full bg-primary"></div>
      </div>
    </div>
    
    <h3 class="text-xl font-semibold mb-2">{{ title }}</h3>
    <p v-if="description" class="text-sm text-muted-foreground mb-6 max-w-md text-center">
      {{ description }}
    </p>
    
    <button 
      v-if="actionLabel"
      @click="emit('action')"
      class="group flex items-center gap-2 px-6 py-3 bg-primary text-primary-foreground rounded-xl text-sm font-medium hover:shadow-lg hover:scale-105 transition-smooth shadow-md"
    >
      <slot name="action-icon">
        <span>+</span>
      </slot>
      {{ actionLabel }}
    </button>
    
    <slot name="actions" />
  </div>
</template>
