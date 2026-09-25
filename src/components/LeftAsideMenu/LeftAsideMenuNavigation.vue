<script setup lang="ts">
import { Disc3, House, Music2, Settings, Tags, UserGroup } from "@lucide/vue";
import { RouterLink, useRouter } from "vue-router";

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
    name: "Albums",
    icon: Disc3,
    route: "/albums",
    animation:
      "group-hover:rotate-390 group-hover:scale-110 duration-600 ease-[cubic-bezier(0.678,-0.202,0.308,1.397)]",
  },
  {
    name: "Artists",
    icon: UserGroup,
    route: "/artists",
    animation: "group-hover:scale-110",
  },
  {
    name: "Songs",
    icon: Music2,
    route: "/songs",
    animation: "song-icon",
  },
  {
    name: "Genres",
    icon: Tags,
    route: "/genres",
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
    <component
      v-for="item in items"
      :key="item.name"
      :is="item.route ? RouterLink : 'button'"
      v-bind="item.route ? { to: item.route } : { type: 'button' }"
      :title="item.name"
      class="group flex cursor-pointer items-center rounded px-3 py-1.5 font-bold text-zinc-100 hover:bg-zinc-800"
      :class="{
        'bg-zinc-800': item.route && isActiveRoute(item.route),
        'justify-center': isCollapsed,
        'gap-3': !isCollapsed,
      }"
    >
      <component
        :is="item.icon"
        class="mt-0.5 size-5 shrink-0 transition-transform delay-100 duration-300 ease-[cubic-bezier(0.678,-0.202,0.308,1.397)] group-hover:delay-0"
        :class="item.animation"
      />
      <span
        class="overflow-hidden whitespace-nowrap transition-[max-width,opacity] duration-150"
        :class="isCollapsed ? 'hidden' : 'max-w-32 opacity-100'"
      >
        {{ item.name }}
      </span>
    </component>
  </nav>
</template>

<style scoped>
.song-icon {
  transform: rotate(12deg);
  transform-origin: 50% 85%;
}

@media (prefers-reduced-motion: no-preference) {
  .group:hover .song-icon,
  .group:focus-visible .song-icon {
    animation: song-shake 900ms ease-in-out both;
  }
}

@keyframes song-shake {
  0%,
  100% {
    transform: rotate(12deg);
  }
  20%,
  60% {
    transform: rotate(2deg);
  }
  40%,
  80% {
    transform: rotate(20deg);
  }
}
</style>
