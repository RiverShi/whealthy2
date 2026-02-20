<script setup lang="ts">
import { type Component } from 'vue';

defineProps<{
  title: string;
  value: string | number;
  description?: string;
  icon?: Component;
  variant?: 'default' | 'primary' | 'success' | 'warning' | 'destructive';
  trend?: 'up' | 'down' | 'neutral';
  trendValue?: string;
}>();

const variantColors = {
  default: {
    iconBg: 'bg-muted',
    iconColor: 'text-muted-foreground',
    valueColor: 'text-foreground',
  },
  primary: {
    iconBg: 'bg-primary/10',
    iconColor: 'text-primary',
    valueColor: 'text-primary',
  },
  success: {
    iconBg: 'bg-success/10',
    iconColor: 'text-success',
    valueColor: 'text-success',
  },
  warning: {
    iconBg: 'bg-warning/10',
    iconColor: 'text-warning',
    valueColor: 'text-warning',
  },
  destructive: {
    iconBg: 'bg-destructive/10',
    iconColor: 'text-destructive',
    valueColor: 'text-destructive',
  },
};
</script>

<template>
  <div class="relative overflow-hidden rounded-2xl border border-border bg-card p-6 shadow-smooth transition-smooth hover:shadow-smooth-lg">
    <div class="absolute top-0 right-0 w-32 h-32 rounded-full -mr-16 -mt-16 opacity-50"
         :class="variant ? `bg-${variant === 'default' ? 'muted' : variant}/5` : 'bg-muted/5'"></div>
    
    <div class="relative">
      <div class="flex items-center justify-between mb-3">
        <p class="text-sm font-medium text-muted-foreground">{{ title }}</p>
        <div v-if="icon" 
             class="w-10 h-10 rounded-xl flex items-center justify-center"
             :class="variantColors[variant || 'default'].iconBg">
          <component :is="icon" class="w-5 h-5" :class="variantColors[variant || 'default'].iconColor" />
        </div>
      </div>
      
      <p class="text-3xl font-bold mb-1" :class="variantColors[variant || 'default'].valueColor">
        {{ value }}
      </p>
      
      <div v-if="description || trendValue" class="flex items-center gap-2">
        <p v-if="description" class="text-xs text-muted-foreground">{{ description }}</p>
        <span v-if="trendValue" class="text-xs font-medium"
              :class="{
                'text-success': trend === 'up',
                'text-destructive': trend === 'down',
                'text-muted-foreground': trend === 'neutral'
              }">
          {{ trendValue }}
        </span>
      </div>
    </div>
  </div>
</template>
