<script setup lang="ts">
import { computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useRouter } from "vue-router";
import { Library, Music, Search, Tags, Users } from "@lucide/vue";
import Button from "@/components/Button.vue";
import MissingCoverImage from "@/components/Album/MissingCoverImage.vue";
import { useAsyncData } from "@/composables/useAsyncData";
import { artworkSource } from "@/utils/artwork";
import type { HomeAlbumSections, LibrarySummary } from "@/types";

const props = defineProps<{
  albumSections: HomeAlbumSections;
}>();

const coverAlbums = computed(() => props.albumSections.heroRandomAlbums.slice(0, 5));
const featuredAlbum = computed(() => coverAlbums.value[0]);
const hasLibraryAlbums = computed(() => coverAlbums.value.length > 0);
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
const countFormatter = new Intl.NumberFormat();
const countLabel = (value: number) => (isSummaryLoading.value ? "..." : countFormatter.format(value));
const librarySections = computed(() => [
  { label: "Albums", count: summary.value.albumCount, icon: Library },
  { label: "Artists", count: summary.value.artistCount, icon: Users },
  { label: "Songs", count: summary.value.songCount, icon: Music },
  { label: "Genres", count: summary.value.genreCount, icon: Tags },
]);
const heroDescriptions = [
  "Put something on, follow a thread, and let the next record find you.",
  "Start anywhere in the shelf and see where the mood takes you.",
  "A quiet place to rediscover what you already saved for later.",
  "Pick a cover, press play, and let the library open up from there.",
  "Old favorites, recent finds, and the next album you forgot you loved.",
];
const heroDescription = heroDescriptions[Math.floor(Math.random() * heroDescriptions.length)];
const router = useRouter();
</script>

<template>
  <section class="overflow-hidden rounded-lg border border-zinc-800 bg-zinc-900">
    <div class="grid min-h-72 gap-6 p-5 md:p-6 lg:grid-cols-[minmax(0,1fr)_24rem] lg:items-center lg:p-7">
      <div class="max-w-3xl space-y-5">
        <div class="flex items-center gap-2 font-sans text-sm font-semibold tracking-wide text-accent uppercase">
          <Library class="size-4" aria-hidden="true" />
          Library
        </div>

        <div class="space-y-3">
          <h1 class="text-4xl leading-tight font-bold text-white md:text-5xl">Home</h1>
          <p class="max-w-2xl font-sans text-base leading-7 text-zinc-300">
            {{ heroDescription }}
          </p>
        </div>

        <div>
          <Button type="button" @click="router.push({ name: 'search' })">
            <Search class="size-4" aria-hidden="true" />
            Search library
          </Button>
        </div>

        <div class="grid max-w-2xl grid-cols-2 gap-2.5 sm:grid-cols-4">
          <button
            v-for="section in librarySections"
            :key="section.label"
            type="button"
            disabled
            class="min-w-0 rounded-md border border-zinc-800 px-3 py-3 text-left disabled:cursor-not-allowed disabled:opacity-80"
          >
            <span class="flex items-center gap-2 text-zinc-400">
              <component :is="section.icon" class="size-4 shrink-0" aria-hidden="true" />
              <span class="truncate font-sans text-xs font-semibold uppercase">{{ section.label }}</span>
            </span>
            <span class="mt-2 block truncate text-2xl leading-none font-bold text-white">
              {{ countLabel(section.count) }}
            </span>
          </button>
        </div>
      </div>

      <div class="relative min-h-72">
        <div
          class="absolute inset-x-5 top-12 bottom-0 rounded-lg border border-zinc-800 bg-zinc-950/80 shadow-2xl shadow-black/40"
        />
        <RouterLink
          v-if="featuredAlbum"
          :to="{ name: 'album', params: { albumId: featuredAlbum.remoteId } }"
          class="group absolute top-0 left-0 w-48 transition-colors duration-300 ease-in-out sm:w-56 lg:w-64"
        >
          <div
            class="relative aspect-square w-full overflow-hidden rounded border-2 border-zinc-800 transition delay-100 duration-300 ease-in-out group-hover:-translate-y-1 group-hover:border-zinc-600 hover:delay-0"
          >
            <img
              v-if="featuredAlbum.artworkPath"
              :src="artworkSource(featuredAlbum.artworkPath)"
              :alt="`${featuredAlbum.name} artwork`"
              class="h-full w-full object-cover object-center"
            />
            <MissingCoverImage v-else />
          </div>
        </RouterLink>
        <div
          v-else
          class="absolute top-0 left-0 aspect-square w-48 overflow-hidden rounded border-2 border-zinc-800 bg-zinc-950 sm:w-56 lg:w-64"
        >
          <MissingCoverImage />
        </div>

        <div v-if="hasLibraryAlbums" class="absolute right-0 bottom-3 grid w-56 grid-cols-2 gap-2.5">
          <RouterLink
            v-for="album in coverAlbums.slice(1)"
            :key="album.remoteId"
            :to="{ name: 'album', params: { albumId: album.remoteId } }"
            class="group transition-colors duration-300 ease-in-out"
          >
            <div
              class="relative aspect-square w-full overflow-hidden rounded border-2 border-zinc-800 transition duration-300 ease-in-out group-hover:-translate-y-1 group-hover:border-zinc-600"
            >
              <img
                v-if="album.artworkPath"
                :src="artworkSource(album.artworkPath)"
                :alt="`${album.name} artwork`"
                class="h-full w-full object-cover object-center"
              />
              <MissingCoverImage v-else />
            </div>
          </RouterLink>
        </div>
      </div>
    </div>
  </section>
</template>
