<script setup lang="ts">
import MissingCoverImage from "@/components/Album/MissingCoverImage.vue";
import type { CachedSong } from "@/types";
import { artworkSource } from "@/utils/artwork";
import { formatTime } from "@/utils/format";
import { invoke } from "@tauri-apps/api/core";

defineProps<{
  song: CachedSong;
  index: number;
  isCurrent: boolean;
}>();
</script>

<template>
  <button
    type="button"
    @click="invoke('player_skip_to_queue_position', { position: index })"
    :data-queue-index="index"
    class="relative grid w-full min-w-0 grid-cols-[2.75rem_minmax(0,1fr)_3rem] items-center gap-3 rounded-md px-2 py-2 text-left font-sans transition-colors focus-visible:bg-zinc-800 focus-visible:outline-none"
    :class="
      isCurrent
        ? 'bg-zinc-800/70 before:absolute before:top-2 before:bottom-2 before:left-0 before:w-1 before:rounded-r before:bg-accent'
        : 'hover:bg-zinc-900'
    "
    :aria-current="isCurrent ? 'true' : undefined"
  >
    <span class="size-11 overflow-hidden rounded border border-zinc-800 bg-zinc-900">
      <img
        v-if="song.artworkPath"
        :src="artworkSource(song.artworkPath)"
        :alt="`${song.albumName} artwork`"
        class="size-full object-cover object-center"
      />
      <MissingCoverImage v-else />
    </span>

    <span class="min-w-0">
      <span class="block truncate text-sm font-medium" :class="isCurrent ? 'text-accent' : 'text-zinc-100'">
        {{ song.title }}
      </span>
      <span class="mt-0.5 block truncate text-xs text-zinc-500">
        {{ song.artistName }}
      </span>
    </span>

    <span class="text-right text-xs text-zinc-500 tabular-nums">
      {{ formatTime(song.durationSeconds) }}
    </span>
  </button>
</template>
