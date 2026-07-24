<script setup lang="ts">
import solmeLogo from "@/assets/solme-logo-dark.svg";

defineProps<{
  isCollapsed: boolean;
}>();
</script>

<template>
  <div class="mb-4 flex items-center gap-2" :class="{ hidden: isCollapsed }">
    <div
      class="flex min-w-0 items-center gap-2 overflow-hidden transition-[max-width,opacity] duration-150"
      :class="isCollapsed ? 'hidden' : 'max-w-32 opacity-100'"
    >
      <img :src="solmeLogo" alt="" class="size-8 shrink-0" />
      <h1 class="overflow-hidden font-serif text-2xl font-bold whitespace-nowrap text-white">
        Solm<span class="text-accent">ë</span>
      </h1>
    </div>
  </div>
</template>
