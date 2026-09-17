<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import LyricsDisplayLayout from "@/components/Lyrics/LyricsDisplayLayout.vue";
import type { SongLyrics } from "@/types.ts";
import { computed, nextTick, ref, useTemplateRef, watch } from "vue";

const props = withDefaults(
  defineProps<{
    lyrics: SongLyrics;
    playbackPositionSeconds: number;
    allowSeek?: boolean;
  }>(),
  {
    allowSeek: true,
  },
);
const currentLyricsIndex = ref(-1);
const lyricsContainer = useTemplateRef<HTMLDivElement>("lyrics-container");

const timedLyricsLines = computed(() =>
  props.lyrics.line
    .map((line, index) => ({ index, start: line.start }))
    .filter((line): line is { index: number; start: number } => line.start !== undefined)
    .map((line) => ({
      index: line.index,
      startSeconds: (line.start + props.lyrics.offset) / 1000,
    })),
);

watch(
  [timedLyricsLines, () => props.playbackPositionSeconds],
  () => {
    const nextLineIndex = timedLyricsLines.value.findIndex((line) => line.startSeconds > props.playbackPositionSeconds);

    if (nextLineIndex === 0 || timedLyricsLines.value.length === 0) {
      currentLyricsIndex.value = -1;
      return;
    }

    const currentLinePosition = nextLineIndex === -1 ? timedLyricsLines.value.length - 1 : nextLineIndex - 1;
    const currentTimedLine = timedLyricsLines.value[currentLinePosition];
    currentLyricsIndex.value = currentTimedLine?.index ?? 0;
  },
  { immediate: true },
);

const scrollCurrentLineIntoView = async (index: number) => {
  await nextTick();
  const lineElement = lyricsContainer.value?.querySelector(`[data-line-index="${index}"]`);
  if (lineElement) {
    lineElement.scrollIntoView({ behavior: "smooth", block: "center", inline: "nearest" });
  }
};

const seekToLine = async (startMilliseconds?: number) => {
  if (!props.allowSeek || startMilliseconds === undefined) {
    return;
  }

  const positionSeconds = Math.max(0, (startMilliseconds + props.lyrics.offset) / 1000);

  try {
    await invoke("player_seek", { positionSeconds });
  } catch (error) {
    console.error("Failed to seek player:", error);
  }
};

watch([currentLyricsIndex], ([index]) => scrollCurrentLineIntoView(index), { immediate: true });
</script>

<template>
  <div ref="lyrics-container" class="w-full">
    <LyricsDisplayLayout class="pb-64">
      <button
        v-for="[index, line] in props.lyrics.line.entries()"
        :key="index"
        type="button"
        :data-line-index="index"
        :disabled="!props.allowSeek || line.start === undefined"
        class="lyrics-line"
        :class="{
          'current-line': index === currentLyricsIndex,
          'past-line': index < currentLyricsIndex,
        }"
        @click="seekToLine(line.start)"
      >
        <template v-if="line.value !== ''">
          {{ line.value }}
        </template>
        <span v-else>...</span>
      </button>
    </LyricsDisplayLayout>
  </div>
</template>

<style scoped>
@reference "@/style/glob.css";

.lyrics-line {
  @apply relative w-full cursor-pointer pl-4 text-left disabled:cursor-default;
  opacity: 0.68;
  transform-origin: left center;
  transition:
    color 500ms ease,
    opacity 500ms ease,
    filter 650ms ease,
    transform 650ms cubic-bezier(0.16, 1, 0.3, 1);
}

.lyrics-line::before {
  @apply absolute bg-accent;
  content: "";
  inset-block: 0.35rem;
  left: 0;
  width: 0.25rem;
  border-radius: 0 0.25rem 0.25rem 0;
  opacity: 0;
  transform: scaleY(0) translateX(-1rem);
  transition:
    opacity 250ms ease,
    transform 600ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.lyrics-line.current-line {
  @apply font-bold text-zinc-50;
  font-size: clamp(1.75rem, 1rem + 2cqw, 3rem);
  opacity: 1;
  filter: blur(0);
  transform: translateX(0.75rem) scale(1.035);
}

.lyrics-line.current-line::before {
  opacity: 1;
  transform: scaleY(1) translateY(0.1rem);
}

.lyrics-line.past-line {
  @apply text-zinc-700;
  transform: translateX(-0.4rem) scale(0.97);
}

@media (prefers-reduced-motion: reduce) {
  .lyrics-line,
  .lyrics-line::before {
    transition: none;
  }
}
</style>
