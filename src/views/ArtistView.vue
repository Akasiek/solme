<script setup lang="ts">
import { useAsyncData } from "@/composables/useAsyncData.ts";
import { invoke } from "@tauri-apps/api/core";
import { watch } from "vue";
import AsyncViewState from "@/components/AsyncViewState.vue";
import MissingCoverImage from "@/components/Album/MissingCoverImage.vue";
import { artworkSource } from "@/utils/artwork";
import type { CachedArtistDetails } from "@/types";

const { artistId } = defineProps<{ artistId: string }>();

const {
  data: artistDetails,
  isLoading,
  error: loadError,
  reload,
} = useAsyncData(
  () =>
    invoke<CachedArtistDetails | null>("get_cached_artist", {
      artistId: artistId,
    }),
  null,
);

watch(
  () => artistId,
  () => {
    void reload();
  },
);
</script>

<template>
  <AsyncViewState :is-loading="isLoading" :error="loadError">
    <template v-if="artistDetails">
      <section class="container mx-auto grid items-center gap-8 p-6 md:grid-cols-[14rem_minmax(0,1fr)]">
        <div class="aspect-square overflow-hidden rounded-lg border border-zinc-800 bg-zinc-900">
          <img
            v-if="artistDetails.artist.artworkPath"
            :src="artworkSource(artistDetails.artist.artworkPath)"
            :alt="`${artistDetails.artist.name} artwork`"
            class="h-full w-full object-cover object-center"
          />
          <MissingCoverImage v-else />
        </div>

        <div class="min-w-0 space-y-3">
          <p class="font-sans text-sm font-semibold tracking-wide text-accent uppercase">Artist</p>
          <h1 class="truncate text-4xl leading-tight font-bold text-white md:text-5xl">
            {{ artistDetails.artist.name }}
          </h1>
          <p class="font-sans text-base text-zinc-300">{{ artistDetails.artist.albumCount }} albums</p>
        </div>
      </section>
    </template>
    <p v-else class="p-6 text-zinc-400">Artist not found.</p>
  </AsyncViewState>
</template>
