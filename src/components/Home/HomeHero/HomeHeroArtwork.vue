<script setup lang="ts">
import { computed } from "vue";
import MissingCoverImage from "@/components/Album/MissingCoverImage.vue";
import { artworkSource } from "@/utils/artwork";
import type { CachedAlbum } from "@/types";

const props = defineProps<{
  albums: CachedAlbum[];
}>();

const coverAlbums = computed(() => props.albums.slice(0, 5));
const featuredAlbum = computed(() => coverAlbums.value[0]);
</script>

<template>
  <div class="relative hidden min-h-52 @min-[36rem]:block @min-[48rem]:min-h-60 @min-[64rem]:min-h-72">
    <div
      class="absolute inset-x-4 top-10 bottom-0 rounded-lg border border-zinc-800 bg-zinc-950/80 shadow-2xl shadow-black/40 @min-[64rem]:inset-x-5 @min-[64rem]:top-12"
    />
    <RouterLink
      v-if="featuredAlbum"
      :to="{ name: 'album', params: { albumId: featuredAlbum.remoteId } }"
      class="group absolute top-0 left-0 w-32 transition-colors duration-300 ease-in-out @min-[36rem]:w-40 @min-[48rem]:w-48 @min-[64rem]:w-64"
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
      class="absolute top-0 left-0 aspect-square w-32 overflow-hidden rounded border-2 border-zinc-800 bg-zinc-950 @min-[36rem]:w-40 @min-[48rem]:w-48 @min-[64rem]:w-64"
    >
      <MissingCoverImage />
    </div>

    <div
      v-if="coverAlbums.length > 0"
      class="absolute right-0 bottom-3 grid w-32 grid-cols-2 gap-2 @min-[36rem]:w-40 @min-[48rem]:w-48 @min-[64rem]:w-56 @min-[64rem]:gap-2.5"
    >
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
</template>
