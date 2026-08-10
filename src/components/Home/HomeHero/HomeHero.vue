<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { Library, Search } from "@lucide/vue";
import Button from "@/components/Button.vue";
import HomeHeroArtwork from "@/components/Home/HomeHero/HomeHeroArtwork.vue";
import HomeHeroLibraryStats from "@/components/Home/HomeHero/HomeHeroLibraryStats.vue";
import { useAsyncData } from "@/composables/useAsyncData";
import { useLayoutStore } from "@/stores/layout";
import type { HomeAlbumSections, LibrarySummary } from "@/types";

defineProps<{
  albumSections: HomeAlbumSections;
}>();

const emptySummary: LibrarySummary = {
  artistCount: 0,
  albumCount: 0,
  songCount: 0,
  genreCount: 0,
};
const { data: summary, isLoading: isSummaryLoading } = useAsyncData(
  () => invoke<LibrarySummary>("get_library_summary"),
  emptySummary,
);
const heroDescriptions = [
  "Put something on, follow a thread, and let the next record find you.",
  "Start anywhere in the shelf and see where the mood takes you.",
  "A quiet place to rediscover what you already saved for later.",
  "Pick a cover, press play, and let the library open up from there.",
  "Old favorites, recent finds, and the next album you forgot you loved.",
];
const heroDescription = heroDescriptions[Math.floor(Math.random() * heroDescriptions.length)];
const { openSearchModal } = useLayoutStore();
</script>

<template>
  <section class="@container overflow-hidden rounded-lg border border-zinc-800 bg-zinc-900">
    <div
      class="grid items-center gap-4 p-5 @min-[36rem]:grid-cols-[minmax(0,2fr)_minmax(10rem,1fr)] @min-[36rem]:gap-6 @min-[36rem]:p-6 @min-[48rem]:grid-cols-[minmax(0,2fr)_minmax(14rem,1fr)] @min-[64rem]:min-h-72 @min-[64rem]:grid-cols-[minmax(0,2fr)_minmax(18rem,1fr)] @min-[64rem]:p-7"
    >
      <div class="@container max-w-3xl space-y-4 @min-[36rem]:space-y-5">
        <div
          class="flex items-center gap-1.5 font-sans text-xxs font-semibold tracking-wide text-accent uppercase @min-[28rem]:text-xs @min-[36rem]:gap-2 @min-[36rem]:text-sm"
        >
          <Library class="size-3.5 @min-[36rem]:size-4" aria-hidden="true" />
          Library
        </div>

        <div class="space-y-2 @min-[36rem]:space-y-3">
          <h1 class="text-3xl leading-tight font-bold text-white @min-[28rem]:text-4xl @min-[36rem]:text-5xl">Home</h1>
          <p class="max-w-2xl font-sans text-sm leading-6 text-zinc-300 @min-[36rem]:text-base @min-[36rem]:leading-7">
            {{ heroDescription }}
          </p>
        </div>

        <div class="text-sm @min-[36rem]:text-base">
          <Button type="button" @click="openSearchModal">
            <Search class="size-3.5 @min-[36rem]:size-4" aria-hidden="true" />
            Search library
          </Button>
        </div>

        <HomeHeroLibraryStats :summary="summary" :is-loading="isSummaryLoading" />
      </div>

      <HomeHeroArtwork :albums="albumSections.heroRandomAlbums" />
    </div>
  </section>
</template>
