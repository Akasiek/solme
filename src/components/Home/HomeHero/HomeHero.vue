<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import { Library, Search } from "@lucide/vue";
import Button from "@/components/Button.vue";
import MissingCoverImage from "@/components/Album/MissingCoverImage.vue";
import { artworkSource } from "@/utils/artwork";
import type { HomeAlbumSections } from "@/types";

const props = defineProps<{
  albumSections: HomeAlbumSections;
}>();

const coverAlbums = computed(() => props.albumSections.heroRandomAlbums.slice(0, 5));
const featuredAlbum = computed(() => coverAlbums.value[0]);
const hasLibraryAlbums = computed(() => coverAlbums.value.length > 0);
const librarySections = [{ label: "Albums" }, { label: "Artists" }, { label: "Genres" }];
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
  <section class="overflow-hidden rounded-lg border border-zinc-800 bg-zinc-950">
    <div class="grid min-h-96 gap-8 p-6 md:p-8 lg:grid-cols-[minmax(0,1fr)_28rem] lg:items-center lg:p-10">
      <div class="max-w-3xl space-y-7">
        <div class="flex items-center gap-2 text-sm font-semibold tracking-wide text-accent uppercase font-sans">
          <Library class="size-4" aria-hidden="true" />
          Library
        </div>

        <div class="space-y-4">
          <h1 class="text-4xl leading-tight font-bold text-white md:text-5xl">Home</h1>
          <p class="max-w-2xl font-sans text-base leading-7 text-zinc-300 md:text-lg">
            {{ heroDescription }}
          </p>
        </div>

        <div class="flex flex-wrap gap-2">
          <Button type="button" @click="router.push({ name: 'search' })">
            <Search class="size-4" aria-hidden="true" />
            Search library
          </Button>
          <Button v-for="section in librarySections" :key="section.label" type="button" variant="outline" disabled>
            {{ section.label }}
          </Button>
        </div>
      </div>

      <div class="relative min-h-80">
        <div
          class="absolute inset-x-6 top-14 bottom-0 rounded-lg border border-zinc-800 bg-zinc-900/80 shadow-2xl shadow-black/40"
        />
        <RouterLink
          v-if="featuredAlbum"
          :to="{ name: 'album', params: { albumId: featuredAlbum.remoteId } }"
          class="group absolute top-0 left-0 w-56 transition-colors duration-300 ease-in-out sm:w-64 lg:w-72"
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
          class="absolute top-0 left-0 aspect-square w-56 overflow-hidden rounded border-2 border-zinc-800 bg-zinc-900 sm:w-64 lg:w-72"
        >
          <MissingCoverImage />
        </div>

        <div v-if="hasLibraryAlbums" class="absolute right-0 bottom-4 grid w-64 grid-cols-2 gap-3">
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
