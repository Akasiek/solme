<script setup lang="ts">
import { Play } from "@lucide/vue";
import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";

import MissingCoverImage from "@/components/Album/MissingCoverImage.vue";
import type { CachedSong } from "@/types";
import { artworkSource } from "@/utils/artwork";
import { formatTime } from "@/utils/format";

const props = defineProps<{
  song: CachedSong;
}>();

defineEmits<{
  select: [];
}>();

const isPlaying = ref(false);

const playTrack = async () => {
  isPlaying.value = true;
  try {
    await invoke("player_play_album", {
      albumId: props.song.albumId,
      startSongId: props.song.remoteId,
    });
  } catch (error) {
    console.error("Failed to play search result", error);
  } finally {
    isPlaying.value = false;
  }
};
</script>

<template>
  <div
    class="group grid grid-cols-[minmax(0,1fr)_2.5rem] items-center gap-2 rounded-md px-2 py-2 transition-colors focus-within:bg-zinc-800 hover:bg-zinc-800"
  >
    <RouterLink
      :to="{ name: 'album', params: { albumId: song.albumId } }"
      class="grid min-w-0 grid-cols-[3.5rem_minmax(0,1fr)_4rem] items-center gap-3 focus-visible:outline-none"
      :title="`Open ${song.albumName}`"
      @click="$emit('select')"
    >
      <div class="size-14 overflow-hidden rounded border border-zinc-700 bg-zinc-950">
        <img
          v-if="song.artworkPath"
          :src="artworkSource(song.artworkPath)"
          :alt="`${song.albumName} artwork`"
          class="h-full w-full object-cover object-center"
        />
        <MissingCoverImage v-else />
      </div>

      <span class="min-w-0">
        <span class="block truncate font-serif text-sm font-semibold text-zinc-100 group-hover:underline">
          {{ song.title }}
        </span>
        <span class="mt-1 block truncate font-sans text-xs text-zinc-400">
          {{ song.artistName }} · {{ song.albumName }}
        </span>
      </span>

      <span class="text-right font-sans text-sm text-zinc-500 tabular-nums">
        {{ formatTime(song.durationSeconds) }}
      </span>
    </RouterLink>

    <button
      type="button"
      class="grid size-9 place-items-center rounded-full bg-accent text-white transition-opacity hover:opacity-80 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-wait disabled:opacity-50"
      :disabled="isPlaying"
      :title="`Play ${song.title}`"
      @click="playTrack"
    >
      <Play class="size-4 fill-current" aria-hidden="true" />
    </button>
  </div>
</template>
