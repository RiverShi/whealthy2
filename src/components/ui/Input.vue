<script setup lang="ts">
import { cn } from "@/lib/utils";
import { type HTMLAttributes, type Component } from "vue";

const props = defineProps<{
  class?: HTMLAttributes["class"];
  modelValue?: string | number;
  type?: "text" | "number" | "email" | "password";
  placeholder?: string;
  disabled?: boolean;
  icon?: Component;
  iconRight?: Component;
  error?: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string | number];
}>();
</script>

<template>
  <div class="relative">
    <div class="relative">
      <component 
        v-if="icon" 
        :is="icon" 
        class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground pointer-events-none" 
      />
      <input
        :type="type || 'text'"
        :value="modelValue"
        :placeholder="placeholder"
        :disabled="disabled"
        @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
        :class="cn(
          'flex h-11 w-full rounded-xl border border-input bg-background px-4 py-2 text-sm transition-smooth file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:border-transparent disabled:cursor-not-allowed disabled:opacity-50',
          icon && 'pl-10',
          iconRight && 'pr-10',
          error && 'border-destructive focus-visible:ring-destructive',
          props.class
        )"
      />
      <component 
        v-if="iconRight" 
        :is="iconRight" 
        class="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground pointer-events-none" 
      />
    </div>
    <p v-if="error" class="text-xs text-destructive mt-1.5 ml-1">{{ error }}</p>
  </div>
</template>
