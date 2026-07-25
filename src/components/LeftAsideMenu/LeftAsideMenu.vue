<script setup lang="ts">
import { useAsideMenuSize } from "@/composables/useAsideMenuSize";

import LeftAsideMenuHeader from "./LeftAsideMenuHeader.vue";
import LeftAsideMenuHistoryNavigation from "./LeftAsideMenuHistoryNavigation.vue";
import LeftAsideMenuNavigation from "./LeftAsideMenuNavigation.vue";
import LeftAsideMenuToggleButton from "@/components/LeftAsideMenu/LeftAsideMenuToggleButton.vue";

const { asideWidth, isCollapsed, isResizing, resetWidth, startResize, toggleCollapsed } =
  useAsideMenuSize("left-aside-menu-width");
</script>

<template>
  <aside
    class="relative h-full shrink-0 overflow-hidden border-r border-zinc-800"
    :class="{ 'transition-[width] duration-200 ease-out': !isResizing }"
    :style="{ width: `${asideWidth}px` }"
  >
    <div class="flex h-full flex-col px-4 py-6">
      <LeftAsideMenuHeader :is-collapsed="isCollapsed" />
      <LeftAsideMenuHistoryNavigation :is-collapsed="isCollapsed" />
      <LeftAsideMenuNavigation :is-collapsed="isCollapsed" />
      <div class="mt-auto border-t border-zinc-800 pt-4 pb-4">
        <LeftAsideMenuToggleButton :is-collapsed="isCollapsed" @toggle="toggleCollapsed" />
      </div>
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
