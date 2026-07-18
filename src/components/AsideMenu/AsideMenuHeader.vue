<script setup lang="ts">
import { PanelLeftClose, PanelLeftOpen } from "@lucide/vue";

import solmeLogo from "@/assets/solme-logo-dark.svg";

defineProps<{
  isCollapsed: boolean;
}>();

const emit = defineEmits<{
  toggle: [];
}>();
</script>

<template>
  <div class="mb-4 flex items-center gap-2" :class="{ 'justify-center': isCollapsed, 'justify-between': !isCollapsed }">
    <div
      class="flex min-w-0 items-center gap-2 overflow-hidden transition-[max-width,opacity] duration-150"
      :class="isCollapsed ? 'hidden' : 'max-w-32 opacity-100'"
    >
      <img :src="solmeLogo" alt="" class="size-8 shrink-0" />
      <h1 class="overflow-hidden font-serif text-2xl font-bold whitespace-nowrap text-white">
        Solm<span class="text-accent">ë</span>
      </h1>
    </div>
    <button
      type="button"
      class="flex cursor-pointer items-center rounded px-2.5 py-2.5 text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100"
      :title="isCollapsed ? 'Expand menu' : 'Collapse menu'"
      @click="emit('toggle')"
    >
      <component :is="isCollapsed ? PanelLeftOpen : PanelLeftClose" class="size-5" />
    </button>
  </div>
</template>
