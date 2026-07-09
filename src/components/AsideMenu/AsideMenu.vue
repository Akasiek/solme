<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { House, PanelLeftClose, PanelLeftOpen, Search, Settings } from "@lucide/vue";
import { useRouter } from "vue-router";

import solmeLogo from "@/assets/solme-logo-dark.svg";

const router = useRouter();
const defaultWidth = 224;
const minWidth = 208;
const maxWidth = 360;
const collapsedWidth = 72;
const collapseThreshold = 136;
const storageKey = "solme-aside-menu";

const width = ref(defaultWidth);
const isCollapsed = ref(false);
const isResizing = ref(false);
const asideWidth = computed(() => (isCollapsed.value ? collapsedWidth : width.value));

let startX = 0;
let startWidth = defaultWidth;
let resizeHandle: HTMLElement | null = null;
let activePointerId: number | null = null;

const isActiveRoute = (route: string) => {
  if (route === "/") {
    return router.currentRoute.value.path === route;
  }

  return router.currentRoute.value.path.startsWith(route);
};

const clampWidth = (value: number) => {
  return Math.min(maxWidth, Math.max(minWidth, value));
};

const clampResizeWidth = (value: number) => {
  return Math.min(maxWidth, Math.max(collapsedWidth, value));
};

const saveState = () => {
  localStorage.setItem(
    storageKey,
    JSON.stringify({
      width: width.value,
      isCollapsed: isCollapsed.value,
    }),
  );
};

const toggleCollapsed = () => {
  isCollapsed.value = !isCollapsed.value;
  saveState();
};

const startResize = (event: PointerEvent) => {
  if (isCollapsed.value) {
    return;
  }

  event.preventDefault();
  startX = event.clientX;
  startWidth = width.value;
  isResizing.value = true;
  activePointerId = event.pointerId;
  resizeHandle = event.currentTarget as HTMLElement;
  resizeHandle.setPointerCapture(activePointerId);

  document.body.classList.add("cursor-col-resize", "select-none");
  window.addEventListener("pointermove", resize);
  window.addEventListener("pointerup", stopResize);
  window.addEventListener("pointercancel", stopResize);
};

const resize = (event: PointerEvent) => {
  if (!isResizing.value) {
    return;
  }

  width.value = clampResizeWidth(startWidth + event.clientX - startX);
};

const stopResize = () => {
  if (!isResizing.value) {
    return;
  }

  if (resizeHandle && activePointerId !== null) {
    resizeHandle.releasePointerCapture(activePointerId);
  }

  isResizing.value = false;
  resizeHandle = null;
  activePointerId = null;
  document.body.classList.remove("cursor-col-resize", "select-none");
  window.removeEventListener("pointermove", resize);
  window.removeEventListener("pointerup", stopResize);
  window.removeEventListener("pointercancel", stopResize);

  if (width.value <= collapseThreshold) {
    isCollapsed.value = true;
    width.value = startWidth;
  } else {
    width.value = clampWidth(width.value);
  }

  saveState();
};

const resetWidth = () => {
  width.value = defaultWidth;
  saveState();
};

const items = [
  {
    name: "Home",
    icon: House,
    route: "/",
    animation: "group-hover:-translate-y-0.25 group-hover:scale-110 group-hover:-rotate-3",
  },
  {
    name: "Search",
    icon: Search,
    route: "/search",
    animation: "group-hover:rotate-12 group-hover:scale-110",
  },
  {
    name: "Settings",
    icon: Settings,
    route: "/settings",
    animation: "group-hover:rotate-45 group-hover:scale-110",
  },
];

onMounted(() => {
  const savedState = localStorage.getItem(storageKey);

  if (!savedState) {
    return;
  }

  try {
    const parsedState = JSON.parse(savedState) as { width?: number; isCollapsed?: boolean };
    if (typeof parsedState.width === "number") {
      width.value = clampWidth(parsedState.width);
    }
    isCollapsed.value = parsedState.isCollapsed === true;
  } catch {
    localStorage.removeItem(storageKey);
  }
});

onUnmounted(() => {
  stopResize();
});
</script>

<template>
  <aside
    class="relative h-full shrink-0 overflow-hidden border-r border-zinc-800"
    :class="{ 'transition-[width] duration-200 ease-out': !isResizing }"
    :style="{ width: `${asideWidth}px` }"
  >
    <div class="flex h-full flex-col px-4 py-6">
      <div>
        <div
          class="mb-4 flex items-center gap-2"
          :class="{ 'justify-center': isCollapsed, 'justify-between': !isCollapsed }"
        >
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
            class="grid size-8 shrink-0 cursor-pointer place-items-center rounded text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100"
            :title="isCollapsed ? 'Expand menu' : 'Collapse menu'"
            @click="toggleCollapsed"
          >
            <component :is="isCollapsed ? PanelLeftOpen : PanelLeftClose" class="size-5" />
          </button>
        </div>

        <nav class="flex flex-col gap-1">
          <RouterLink
            v-for="item in items"
            :key="item.name"
            :to="item.route"
            :title="item.name"
            class="group flex items-center rounded py-2 font-medium text-zinc-100 hover:bg-zinc-800"
            :class="{
              'bg-zinc-800': isActiveRoute(item.route),
              'justify-center px-4': isCollapsed,
              'gap-3 px-4': !isCollapsed,
            }"
          >
            <component
              :is="item.icon"
              class="mt-0.5 size-5 shrink-0 transition-transform duration-200 ease-out"
              :class="item.animation"
            />
            <span
              class="overflow-hidden whitespace-nowrap transition-[max-width,opacity] duration-150"
              :class="isCollapsed ? 'hidden' : 'max-w-32 opacity-100'"
            >
              {{ item.name }}
            </span>
          </RouterLink>
        </nav>
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
