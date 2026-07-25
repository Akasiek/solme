<script setup lang="ts">
import { PanelLeftClose, PanelLeftOpen } from "@lucide/vue";

defineProps<{
  isCollapsed: boolean;
}>();

defineEmits<{
  (e: "toggle"): void;
}>();
</script>

<template>
  <button
    type="button"
    class="flex w-full cursor-pointer items-center rounded px-4 py-2 text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-zinc-100"
    :class="isCollapsed ? 'justify-center' : 'gap-3'"
    :title="isCollapsed ? 'Expand menu' : 'Collapse menu'"
    @click="$emit('toggle')"
  >
    <component :is="isCollapsed ? PanelLeftOpen : PanelLeftClose" class="size-5 shrink-0" />
    <span v-if="!isCollapsed" class="text-sm font-medium whitespace-nowrap">Collapse menu</span>
  </button>
</template>
