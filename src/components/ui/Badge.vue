<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  variant?: 'default' | 'primary' | 'success' | 'warning' | 'destructive' | 'info';
  size?: 'sm' | 'md' | 'lg';
}>(), {
  variant: 'default',
  size: 'md',
});

const badgeClass = computed(() => {
  const baseClass = 'inline-flex items-center justify-center font-medium rounded-full transition-smooth';
  
  const variantClasses = {
    default: 'bg-muted text-muted-foreground',
    primary: 'bg-primary/10 text-primary',
    success: 'bg-success/10 text-success',
    warning: 'bg-warning/10 text-warning',
    destructive: 'bg-destructive/10 text-destructive',
    info: 'bg-info/10 text-info',
  };
  
  const sizeClasses = {
    sm: 'px-2 py-0.5 text-xs',
    md: 'px-2.5 py-1 text-sm',
    lg: 'px-3 py-1.5 text-base',
  };
  
  return `${baseClass} ${variantClasses[props.variant]} ${sizeClasses[props.size]}`;
});
</script>

<template>
  <span :class="badgeClass">
    <slot />
  </span>
</template>
