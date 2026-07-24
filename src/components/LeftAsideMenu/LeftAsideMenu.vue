<script setup lang="ts">
import { PanelLeftClose, PanelLeftOpen } from "@lucide/vue";

import { useAsideMenuSize } from "@/composables/useAsideMenuSize";

import LeftAsideMenuHeader from "./LeftAsideMenuHeader.vue";
import LeftAsideMenuHistoryNavigation from "./LeftAsideMenuHistoryNavigation.vue";
import LeftAsideMenuNavigation from "./LeftAsideMenuNavigation.vue";

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
      <div class="mt-auto border-t border-zinc-800 pt-4">
        <button
          type="button"
          class="flex w-full cursor-pointer items-center rounded px-4 py-2 text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-zinc-100"
          :class="isCollapsed ? 'justify-center' : 'gap-3'"
          :title="isCollapsed ? 'Expand menu' : 'Collapse menu'"
          @click="toggleCollapsed"
        >
          <component :is="isCollapsed ? PanelLeftOpen : PanelLeftClose" class="size-5 shrink-0" />
          <span v-if="!isCollapsed" class="text-sm font-medium whitespace-nowrap">Collapse menu</span>
        </button>
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
