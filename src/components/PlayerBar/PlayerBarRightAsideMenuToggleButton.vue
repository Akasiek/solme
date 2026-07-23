<script setup lang="ts">
import { PanelRightClose, PanelRightOpen } from "@lucide/vue";
import { useLayoutStore } from "@/stores/layout.ts";

const layoutStore = useLayoutStore();
</script>

<template>
  <button
    type="button"
    class="flex cursor-pointer items-center rounded p-2.5 text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100"
    :title="layoutStore.isRightAsideCollapsed ? 'Show queue' : 'Hide queue'"
    :aria-label="layoutStore.isRightAsideCollapsed ? 'Show queue' : 'Hide queue'"
    @click="layoutStore.toggleRightAside"
  >
    <component :is="layoutStore.isRightAsideCollapsed ? PanelRightOpen : PanelRightClose" class="size-5" />
  </button>
</template>
