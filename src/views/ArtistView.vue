<script setup lang="ts">
import { useAsyncData } from "@/composables/useAsyncData.ts";
import { invoke } from "@tauri-apps/api/core";
import { watch } from "vue";
import AsyncViewState from "@/components/AsyncViewState.vue";
import ArtistAlbums from "@/components/Artist/ArtistAlbums";
import ArtistHero from "@/components/Artist/ArtistHero";
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
      <ArtistHero :artist-details="artistDetails" />
      <ArtistAlbums :albums="artistDetails.albums" />
    </template>
    <p v-else class="p-6 text-zinc-400">Artist not found.</p>
  </AsyncViewState>
</template>
