<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  variant?: 'default' | 'gradient' | 'glass' | 'bordered';
  hover?: boolean;
  padding?: 'none' | 'sm' | 'md' | 'lg';
}>(), {
  variant: 'default',
  hover: false,
  padding: 'md',
});

const cardClass = computed(() => {
  const baseClass = 'rounded-2xl transition-smooth';
  
  const variantClasses = {
    default: 'bg-card border border-border shadow-smooth',
    gradient: 'bg-gradient-to-br from-card via-card to-accent/5 border border-border shadow-smooth',
    glass: 'glass shadow-smooth',
    bordered: 'bg-card border-2 border-border',
  };
  
  const paddingClasses = {
    none: '',
    sm: 'p-4',
    md: 'p-6',
    lg: 'p-8',
  };
  
  const hoverClass = props.hover ? 'hover:shadow-smooth-lg hover:scale-[1.02] cursor-pointer' : '';
  
  return `${baseClass} ${variantClasses[props.variant]} ${paddingClasses[props.padding]} ${hoverClass}`;
});
</script>

<template>
  <div :class="cardClass">
    <slot />
  </div>
</template>
