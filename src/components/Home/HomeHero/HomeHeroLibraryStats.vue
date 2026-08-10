<script setup lang="ts">
import { computed } from "vue";
import { Library, Music, Tags, Users } from "@lucide/vue";
import type { LibrarySummary } from "@/types";

const props = defineProps<{
  summary: LibrarySummary;
  isLoading: boolean;
}>();

const countFormatter = new Intl.NumberFormat();
const sections = computed(() => [
  { label: "Albums", count: props.summary.albumCount, icon: Library },
  { label: "Artists", count: props.summary.artistCount, icon: Users },
  { label: "Songs", count: props.summary.songCount, icon: Music },
  { label: "Genres", count: props.summary.genreCount, icon: Tags },
]);
const countLabel = (value: number) => (props.isLoading ? "..." : countFormatter.format(value));
</script>

<template>
  <div class="grid max-w-2xl grid-cols-1 gap-2.5 @min-[20rem]:grid-cols-2 @min-[32rem]:grid-cols-4">
    <button
      v-for="section in sections"
      :key="section.label"
      type="button"
      disabled
      class="min-w-0 rounded-md border border-zinc-800 px-2.5 py-2.5 text-left disabled:cursor-not-allowed disabled:opacity-80 @min-[32rem]:px-3 @min-[32rem]:py-3"
    >
      <span class="flex items-center gap-1.5 text-zinc-400 @min-[32rem]:gap-2">
        <component :is="section.icon" class="size-3.5 shrink-0 @min-[32rem]:size-4" aria-hidden="true" />
        <span class="truncate font-sans text-xxs font-semibold uppercase @min-[32rem]:text-xs">
          {{ section.label }}
        </span>
      </span>
      <span
        class="mt-1.5 block truncate text-xl leading-none font-bold text-white @min-[32rem]:mt-2 @min-[32rem]:text-2xl"
      >
        {{ countLabel(section.count) }}
      </span>
    </button>
  </div>
</template>
