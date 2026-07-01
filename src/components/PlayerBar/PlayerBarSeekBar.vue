<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { PlayerStatus } from "@/types.ts";

const { playerStatus } = defineProps<{
  playerStatus: PlayerStatus;
}>();

const currentPosition = ref(Math.round(playerStatus.positionSeconds));
const isSeeking = ref(false);
let progressTimer: number | undefined;

const duration = computed(() => Math.max(0, Math.round(playerStatus.durationSeconds)));
const progressPercent = computed(() => {
  if (duration.value === 0) {
    return 0;
  }

  return Math.min(100, Math.max(0, (currentPosition.value / duration.value) * 100));
});
const seekTrackStyle = computed(() => ({
  background: `linear-gradient(to right, var(--color-zinc-200) 0%, var(--color-zinc-200) ${progressPercent.value}%, var(--color-zinc-600) ${progressPercent.value}%, var(--color-zinc-600) 100%)`,
}));

watch(
  () => [playerStatus.positionSeconds, playerStatus.durationSeconds],
  () => {
    if (!isSeeking.value) {
      currentPosition.value = Math.round(playerStatus.positionSeconds);
    }
  },
);

onMounted(() => {
  progressTimer = window.setInterval(() => {
    if (playerStatus.state !== "playing" || isSeeking.value || duration.value === 0) {
      return;
    }

    currentPosition.value = Math.min(duration.value, currentPosition.value + 1);
  }, 1000);
});

onUnmounted(() => {
  if (progressTimer !== undefined) {
    window.clearInterval(progressTimer);
  }
});

const onSeekInput = (event: Event) => {
  isSeeking.value = true;
  currentPosition.value = parseInt((event.target as HTMLInputElement).value);
};

const onSeekChange = async (event: Event) => {
  const positionSeconds = parseInt((event.target as HTMLInputElement).value);
  currentPosition.value = positionSeconds;

  try {
    await invoke("player_seek", { positionSeconds });
  } catch (error) {
    console.error("Failed to seek player:", error);
    currentPosition.value = Math.round(playerStatus.positionSeconds);
  } finally {
    isSeeking.value = false;
  }
};

const formatTime = (seconds: number) => {
  const safeSeconds = Math.max(0, Math.round(seconds));
  const minutes = Math.floor(safeSeconds / 60);
  const remainingSeconds = safeSeconds % 60;

  return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
};
</script>

<template>
  <div class="seek-slider-wrapper">
    <span class="seek-slider-time">{{ formatTime(currentPosition) }}</span>
    <input
      type="range"
      min="0"
      :max="duration"
      :value="currentPosition"
      class="seek-slider"
      :style="seekTrackStyle"
      :disabled="duration === 0"
      @input="onSeekInput"
      @change="onSeekChange"
    />
    <span class="seek-slider-time">{{ formatTime(duration) }}</span>
  </div>
</template>

<style scoped>
@reference "@/style/glob.css";

.seek-slider-wrapper {
  @apply flex h-4 w-full max-w-md min-w-72 items-center gap-3;
}

.seek-slider-time {
  @apply w-10 text-center text-xs leading-none font-medium text-zinc-300 tabular-nums;
}

.seek-slider {
  @apply h-1.5 w-full cursor-pointer appearance-none rounded-full disabled:cursor-not-allowed disabled:opacity-50;
}

.seek-slider::-webkit-slider-thumb {
  @apply size-4 appearance-none rounded-full bg-accent shadow-[0_0_12px_#00000060];
}

.seek-slider::-moz-range-thumb {
  @apply size-4 rounded-full border-0 bg-accent shadow-[0_0_12px_#00000060];
}
</style>
