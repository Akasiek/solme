<script setup lang="ts">
import { House, Search, Settings } from "@lucide/vue";
import { useRouter } from "vue-router";

defineProps<{
  isCollapsed: boolean;
}>();

const router = useRouter();

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

const isActiveRoute = (route: string) => {
  if (route === "/") {
    return router.currentRoute.value.path === route;
  }

  return router.currentRoute.value.path.startsWith(route);
};
</script>

<template>
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
</template>
