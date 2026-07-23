<script setup lang="ts">
import { storeToRefs } from "pinia";
import Queue from "@/components/RightAsideMenu/Queue";
import { useAsideMenuSize } from "@/composables/useAsideMenuSize.ts";
import { useLayoutStore } from "@/stores/layout";

const layoutStore = useLayoutStore();
const { isRightAsideCollapsed } = storeToRefs(layoutStore);

const { asideWidth, isCollapsed, isResizing, resetWidth, startResize } = useAsideMenuSize("right-aside-menu-width", {
  defaultWidth: 320,
  minWidth: 240,
  maxWidth: 480,
  collapsedWidth: 0,
  collapseThreshold: 120,
  isLeft: false,
  isCollapsed: isRightAsideCollapsed,
});
</script>

<template>
  <aside
    class="relative flex h-full shrink-0 flex-col overflow-hidden border-l border-zinc-800 bg-zinc-950"
    :class="{ 'transition-[width] duration-200 ease-out': !isResizing }"
    :style="{ width: `${asideWidth}px` }"
  >
    <Queue />
    <div
      v-if="!isCollapsed"
      class="absolute top-0 left-0 h-full w-1 cursor-col-resize transition-colors hover:bg-accent"
      title="Resize menu"
      @dblclick="resetWidth"
      @pointerdown="startResize"
    />
  </aside>
</template>
