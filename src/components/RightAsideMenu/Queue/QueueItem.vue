<script setup lang="ts">
import { ref } from "vue";
import SongContextMenu from "@/components/SongContextMenu/SongContextMenu.vue";
import { GripVertical, Ellipsis } from "@lucide/vue";
import { invoke } from "@tauri-apps/api/core";
import MissingCoverImage from "@/components/Album/MissingCoverImage.vue";
import ArtworkImage from "@/components/ArtworkImage.vue";
import type { CachedSong } from "@/types";
import { formatTime } from "@/utils/format";

const props = defineProps<{
  song: CachedSong;
  entryId: string;
  index: number;
  songCount: number;
  isCurrent: boolean;
  isDragged: boolean;
  dropBefore: boolean;
  dropAfter: boolean;
  reorderDisabled: boolean;
}>();

const contextMenu = ref<{ x: number; y: number }>();

const emit = defineEmits<{
  reorderStart: [event: PointerEvent, index: number];
  reorderCancel: [];
  reorderKeyboard: [from: number, to: number];
}>();

const moveWithKeyboard = (event: KeyboardEvent) => {
  if (!event.altKey || !["ArrowUp", "ArrowDown"].includes(event.key)) return;
  event.preventDefault();
  const to = props.index + (event.key === "ArrowUp" ? -1 : 1);
  if (to >= 0 && to < props.songCount) emit("reorderKeyboard", props.index, to);
};
</script>

<template>
  <div
    :data-queue-index="index"
    @contextmenu.prevent="contextMenu = { x: $event.clientX, y: $event.clientY }"
    class="relative flex w-full min-w-0 items-center rounded-md font-sans transition-colors"
    :class="[
      isCurrent
        ? 'bg-zinc-800/70 before:absolute before:top-2 before:bottom-2 before:left-0 before:w-1 before:rounded-r before:bg-accent'
        : 'hover:bg-zinc-900',
      isDragged ? 'opacity-40' : '',
    ]"
  >
    <span v-if="dropBefore" class="pointer-events-none absolute inset-x-1 top-0 z-10 h-0.5 rounded bg-accent" />

    <button
      type="button"
      :disabled="reorderDisabled"
      :aria-label="`Reorder ${song.title}, position ${index + 1} of ${songCount}`"
      aria-keyshortcuts="Alt+ArrowUp Alt+ArrowDown"
      title="Drag to reorder, or use Alt + Arrow Up / Down"
      class="ml-1 grid w-7 shrink-0 cursor-grab touch-none place-items-center self-stretch rounded text-zinc-500 select-none hover:text-zinc-200 focus-visible:ring-2 focus-visible:ring-accent focus-visible:outline-none active:cursor-grabbing disabled:cursor-default disabled:opacity-30"
      @pointerdown="emit('reorderStart', $event, index)"
      @lostpointercapture="emit('reorderCancel')"
      @keydown="moveWithKeyboard"
      @click.stop
    >
      <GripVertical class="size-4" aria-hidden="true" />
    </button>

    <button
      type="button"
      @click="invoke('player_skip_to_queue_position', { position: index })"
      :aria-current="isCurrent ? 'true' : undefined"
      class="grid min-w-0 flex-1 grid-cols-[2.75rem_minmax(0,1fr)_3rem] items-center rounded-md py-2 pr-2 text-left transition-colors focus-visible:bg-zinc-800 focus-visible:outline-none"
    >
      <span class="size-9 overflow-hidden rounded bg-zinc-900">
        <ArtworkImage
          v-if="song.artworkPath"
          :path="song.artworkPath"
          :alt="`${song.albumName} artwork`"
          class="size-full object-cover object-center"
        />
        <MissingCoverImage v-else />
      </span>
      <span class="min-w-0">
        <span
          class="block truncate font-serif text-sm font-medium"
          :class="isCurrent ? 'text-accent' : 'text-zinc-100'"
        >
          {{ song.title }}
        </span>
        <span class="mt-0.5 block truncate text-xs text-zinc-500">{{ song.artistName }}</span>
      </span>
      <span class="text-right text-xs text-zinc-500 tabular-nums">{{ formatTime(song.durationSeconds) }}</span>
    </button>
    <button
      type="button"
      :aria-label="`${song.title} actions`"
      class="mr-1 grid size-7 shrink-0 place-items-center rounded text-zinc-500 hover:bg-zinc-800 hover:text-zinc-100 focus-visible:ring-2 focus-visible:ring-accent focus-visible:outline-none"
      @click="contextMenu = { x: $event.clientX, y: $event.clientY }"
    >
      <Ellipsis class="size-4" aria-hidden="true" />
    </button>
    <SongContextMenu
      variant="queue"
      :song="song"
      :entry-id="entryId"
      :open="Boolean(contextMenu)"
      :x="contextMenu?.x ?? 0"
      :y="contextMenu?.y ?? 0"
      :disabled="reorderDisabled"
      @close="contextMenu = undefined"
    />
    <span v-if="dropAfter" class="pointer-events-none absolute inset-x-1 bottom-0 z-10 h-0.5 rounded bg-accent" />
  </div>
</template>
