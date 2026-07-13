<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted, ref } from "vue";
import { CachedAlbumDetails, CachedSong } from "@/types.ts";
import { usePlayerStatusStore } from "@/stores/playerStatus.ts";
import AlbumTracklistDiscHeader from "./AlbumTracklistDiscHeader.vue";
import AlbumTracklistEmptyState from "./AlbumTracklistEmptyState.vue";
import AlbumTracklistHeader from "./AlbumTracklistHeader.vue";
import AlbumTracklistRow from "./AlbumTracklistRow.vue";

const { albumDetails } = defineProps<{
  albumDetails: CachedAlbumDetails;
}>();

const showDiscNumbers = computed(() => albumDetails.discCount > 1);

const trackNumber = (song: CachedSong, index: number) => song.trackNumber ?? index + 1;

const shouldShowDiscHeader = (song: CachedSong, index: number) => {
  if (!showDiscNumbers.value) {
    return false;
  }

  const previous = albumDetails.songs[index - 1];
  return !previous || (previous.discNumber ?? 1) !== (song.discNumber ?? 1);
};

const shouldShowArtist = (song: CachedSong) => song.artistName !== albumDetails.album.artistName;

const discNumber = (song: CachedSong) => song.discNumber ?? 1;

const playingSongId = ref<string>();
const playerStatusStore = usePlayerStatusStore();

const currentAlbumSongId = computed(() => {
  const currentSong = playerStatusStore.currentSong;
  if (currentSong?.albumId !== albumDetails.album.remoteId) {
    return undefined;
  }

  return currentSong.remoteId;
});

const isCurrentSong = (song: CachedSong) => currentAlbumSongId.value === song.remoteId;

const playTrack = async (song: CachedSong) => {
  playingSongId.value = song.remoteId;
  try {
    await invoke("player_play_album", {
      albumId: albumDetails.album.remoteId,
      startSongId: song.remoteId,
    });
  } catch (error) {
    console.error("Failed to play album track", error);
  } finally {
    playingSongId.value = undefined;
  }
};

onMounted(async () => {
  await playerStatusStore.startListening();
});
</script>

<template>
  <section class="px-6 pb-12">
    <div
      class="container mx-auto rounded-lg border border-zinc-800 bg-zinc-950/70 p-6 shadow-2xl shadow-black/20 lg:p-8"
    >
      <div class="mb-5 flex items-end justify-between gap-4">
        <div class="min-w-0">
          <h2 class="font-serif text-2xl leading-tight font-bold text-white">Tracks</h2>
          <p class="mt-1 font-sans text-sm text-zinc-500">{{ albumDetails.album.songCount }} tracks</p>
        </div>
      </div>

      <div>
        <AlbumTracklistHeader />

        <div v-if="albumDetails.songs.length">
          <template v-for="(song, index) in albumDetails.songs" :key="song.remoteId">
            <AlbumTracklistDiscHeader v-if="shouldShowDiscHeader(song, index)" :disc-number="discNumber(song)" />

            <AlbumTracklistRow
              :song="song"
              :track-number="trackNumber(song, index)"
              :show-artist="shouldShowArtist(song)"
              :is-current="isCurrentSong(song)"
              :is-loading="playingSongId === song.remoteId"
              @play="playTrack"
            />
          </template>
        </div>

        <AlbumTracklistEmptyState v-else />
      </div>
    </div>
  </section>
</template>
