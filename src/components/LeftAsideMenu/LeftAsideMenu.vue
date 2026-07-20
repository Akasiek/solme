<script setup lang="ts">
import { useAsideMenuSize } from "@/composables/useAsideMenuSize";

import LeftAsideMenuHeader from "./LeftAsideMenuHeader.vue";
import LeftAsideMenuNavigation from "./LeftAsideMenuNavigation.vue";

const { asideWidth, isCollapsed, isResizing, resetWidth, startResize, toggleCollapsed } = useAsideMenuSize();
</script>

<template>
  <aside
    class="relative h-full shrink-0 overflow-hidden border-r border-zinc-800"
    :class="{ 'transition-[width] duration-200 ease-out': !isResizing }"
    :style="{ width: `${asideWidth}px` }"
  >
    <div class="flex h-full flex-col px-4 py-6">
      <LeftAsideMenuHeader :is-collapsed="isCollapsed" @toggle="toggleCollapsed" />
      <LeftAsideMenuNavigation :is-collapsed="isCollapsed" />
    </div>
    <div
      v-if="!isCollapsed"
      class="absolute top-0 right-0 h-full w-1 cursor-col-resize transition-colors hover:bg-accent"
      title="Resize menu"
      @dblclick="resetWidth"
      @pointerdown="startResize"
    />
  </aside>
</template>
