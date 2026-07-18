<script setup lang="ts">
import { House, PanelLeftClose, PanelLeftOpen, Search, Settings } from "@lucide/vue";
import { useRouter } from "vue-router";

import solmeLogo from "@/assets/solme-logo-dark.svg";
import { useAsideMenuSize } from "@/composables/useAsideMenuSize";

const router = useRouter();
const { asideWidth, isCollapsed, isResizing, resetWidth, startResize, toggleCollapsed } = useAsideMenuSize();

const isActiveRoute = (route: string) => {
  if (route === "/") {
    return router.currentRoute.value.path === route;
  }

  return router.currentRoute.value.path.startsWith(route);
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
            class="flex cursor-pointer items-center rounded px-2.5 py-2.5 text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100"
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
            class="group flex items-center rounded px-4 py-2 font-medium text-zinc-100 hover:bg-zinc-800"
            :class="{
              'bg-zinc-800': isActiveRoute(item.route),
              'justify-center': isCollapsed,
              'gap-3': !isCollapsed,
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
