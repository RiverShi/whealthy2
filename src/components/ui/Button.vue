<script setup lang="ts">
import { cn } from "@/lib/utils";
import { type HTMLAttributes, type Component, computed } from "vue";

const props = defineProps<{
  variant?: "default" | "primary" | "success" | "destructive" | "outline" | "secondary" | "ghost" | "link";
  size?: "default" | "sm" | "lg" | "xl" | "icon";
  class?: HTMLAttributes["class"];
  loading?: boolean;
  icon?: Component;
  iconRight?: Component;
}>();

const buttonClass = computed(() => {
  const base = "inline-flex items-center justify-center gap-2 rounded-xl text-sm font-medium transition-smooth focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50";
  
  const variants = {
    default: "bg-card border border-border text-foreground hover:bg-accent hover:border-primary/30",
    primary: "bg-primary text-primary-foreground shadow-md hover:shadow-lg hover:scale-105",
    success: "bg-success text-success-foreground shadow-md hover:shadow-lg hover:scale-105",
    destructive: "bg-destructive text-destructive-foreground shadow-md hover:shadow-lg hover:scale-105",
    outline: "border-2 border-border bg-transparent hover:bg-accent hover:text-accent-foreground",
    secondary: "bg-secondary text-secondary-foreground hover:bg-secondary/80",
    ghost: "hover:bg-accent hover:text-accent-foreground",
    link: "text-primary underline-offset-4 hover:underline",
  };
  
  const sizes = {
    default: "h-10 px-4 py-2",
    sm: "h-8 px-3 text-xs",
    lg: "h-12 px-6 text-base",
    xl: "h-14 px-8 text-lg",
    icon: "h-10 w-10",
  };
  
  return cn(
    base,
    variants[props.variant || "default"],
    sizes[props.size || "default"],
    props.class
  );
});
</script>

<template>
  <button :class="buttonClass" :disabled="loading">
    <component v-if="icon && !loading" :is="icon" class="w-4 h-4 shrink-0" />
    <div v-if="loading" class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
    <slot />
    <component v-if="iconRight && !loading" :is="iconRight" class="w-4 h-4 shrink-0" />
  </button>
</template>
