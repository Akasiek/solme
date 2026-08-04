<script setup lang="ts">
import { PlayerStatus } from "@/types.ts";
import { LoaderCircle, Play, Pause, SkipBack, SkipForward } from "@lucide/vue";
import { invoke } from "@tauri-apps/api/core";
import { computed } from "vue";

const { playerStatus } = defineProps<{
  playerStatus: PlayerStatus;
}>();

const onPlayPause = (playerState: PlayerStatus["state"]) => {
  if (playerState === "loading") {
    return;
  }
  const cmd = playerState === "playing" ? "player_pause" : "player_resume";
  invoke(cmd).catch((error) => {
    console.error(`Failed to ${playerState === "playing" ? "pause" : "resume"} player:`, error);
  });
};

const canGoBack = computed(() => {
  return (
    playerStatus.state !== "loading" &&
    playerStatus.queuePosition !== undefined &&
    (playerStatus.queuePosition > 1 || playerStatus.positionSeconds > 5)
  );
});

const canGoNext = computed(
  () =>
    playerStatus.state !== "loading" &&
    playerStatus.queuePosition !== undefined &&
    playerStatus.queuePosition < playerStatus.queueLength,
);
</script>

<template>
  <div class="playback-control-container">
    <button @click="invoke('player_previous')" :disabled="!canGoBack">
      <SkipBack class="size-3.5" />
    </button>
    <button @click="onPlayPause(playerStatus.state)" :disabled="playerStatus.state === 'loading'">
      <LoaderCircle v-if="playerStatus.state === 'loading'" class="size-5 animate-spin fill-transparent!" />
      <component v-else :is="playerStatus.state === 'playing' ? Pause : Play" class="size-5" />
    </button>
    <button @click="invoke('player_next')" :disabled="!canGoNext">
      <SkipForward class="size-3.5" />
    </button>
  </div>
</template>

<style scoped>
@reference "@/style/glob.css";

.playback-control-container {
  @apply flex items-center justify-center gap-4;
}

.playback-control-container > button {
  @apply cursor-pointer rounded-full bg-accent p-2.5 text-zinc-100 transition-colors duration-300 ease-in-out;
}

.playback-control-container > button:not(:disabled):hover {
  @apply bg-accent/90;
}

.playback-control-container > button:not(:disabled):active {
  @apply scale-95 bg-accent/80;
}

.playback-control-container > button:disabled {
  @apply cursor-not-allowed bg-zinc-700;
}

.playback-control-container > button > svg {
  @apply fill-white;
}
</style>
