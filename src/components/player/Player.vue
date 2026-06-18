<script setup lang="ts">
import {invoke} from "@tauri-apps/api/core";
import {computed, onMounted, onUnmounted, ref} from "vue";

import PlayerButton from "@/components/player/PlayerButton.vue";
import PlayerDisplay from "@/components/player/PlayerDisplay.vue";
import PlayIcon from "@/components/icons/PlayIcon.vue";
import StopIcon from "@/components/icons/StopIcon.vue";
import PrevIcon from "@/components/icons/PrevIcon.vue";
import NextIcon from "@/components/icons/NextIcon.vue";
import PauseIcon from "@/components/icons/PauseIcon.vue";
import type {PlayerStatus} from "@/types";
import {formatTime} from "@/utils/format";

type PlayerCommand = "resume" | "pause" | "stop" | "next" | "previous";

const status = ref<PlayerStatus | null>(null);
const errorMessage = ref("");
const pendingCommand = ref<PlayerCommand | null>(null);
const requestedPlaybackState = ref<PlayerStatus["state"] | null>(null);
let refreshTimer: ReturnType<typeof setInterval> | undefined;
let refreshInProgress = false;

const currentSong = computed(() => status.value?.currentSong);
const displayTrack = computed(
    () => currentSong.value?.trackNumber ?? status.value?.queuePosition ?? 0,
);
const displayTitle = computed(() => currentSong.value?.title ?? "NO DISC");
const displayArtist = computed(() => currentSong.value?.artistName ?? "");
const displayAlbum = computed(() => currentSong.value?.albumName ?? "");
const displayState = computed(() => status.value?.state ?? "stopped");
const displayElapsed = computed(() => formatTime(status.value?.positionSeconds ?? 0));
const controlState = computed(() => requestedPlaybackState.value ?? displayState.value);

const hasTrack = computed(() => Boolean(currentSong.value));
const canPrevious = computed(() => hasTrack.value && (status.value?.queuePosition ?? 0) > 1);
const canNext = computed(
    () => hasTrack.value && (status.value?.queuePosition ?? 0) < (status.value?.queueLength ?? 0),
);

async function refreshPlayerStatus() {
  if (refreshInProgress) {
    return;
  }

  refreshInProgress = true;
  try {
    status.value = await invoke<PlayerStatus>("get_player_status");
    if (requestedPlaybackState.value === status.value.state) {
      requestedPlaybackState.value = null;
    }
    errorMessage.value = "";
  } catch (error) {
    errorMessage.value = String(error);
  } finally {
    refreshInProgress = false;
  }
}

async function runCommand(command: PlayerCommand) {
  if (pendingCommand.value) {
    return;
  }

  pendingCommand.value = command;
  errorMessage.value = "";

  if (command === "pause") {
    requestedPlaybackState.value = "paused";
  } else if (command === "resume") {
    requestedPlaybackState.value = "playing";
  } else if (command === "stop") {
    requestedPlaybackState.value = "stopped";
  }

  try {
    await invoke(command);
    await refreshPlayerStatus();
  } catch (error) {
    requestedPlaybackState.value = null;
    errorMessage.value = String(error);
  } finally {
    pendingCommand.value = null;
  }
}

onMounted(async () => {
  await refreshPlayerStatus();
  refreshTimer = setInterval(refreshPlayerStatus, 250);
});

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer);
  }
});
</script>

<template>
  <div class="relative z-10 flex h-full w-full flex-col items-center justify-center gap-5 px-4">
    <PlayerDisplay
        :track="displayTrack"
        :elapsed="displayElapsed"
        :title="displayTitle"
        :artist="displayArtist"
        :album="displayAlbum"
        :state="displayState"
    />

    <div class="flex max-w-full space-x-1">
      <PlayerButton
          label="Stop"
          :disabled="controlState === 'stopped'"
          @click="runCommand('stop')"
      >
        <StopIcon/>
      </PlayerButton>
      <PlayerButton
          label="Previous track"
          :disabled="!canPrevious"
          @click="runCommand('previous')"
      >
        <PrevIcon/>
      </PlayerButton>
      <PlayerButton
          label="Play"
          :disabled="controlState === 'playing' || !hasTrack"
          @click="runCommand('resume')"
      >
        <PlayIcon/>
      </PlayerButton>
      <PlayerButton label="Next track" :disabled="!canNext" @click="runCommand('next')">
        <NextIcon/>
      </PlayerButton>
      <PlayerButton
          label="Pause"
          :disabled="controlState !== 'playing'"
          @click="runCommand('pause')"
      >
        <PauseIcon/>
      </PlayerButton>
    </div>

    <p
        v-if="errorMessage"
        role="alert"
        class="max-w-xl font-display-condensed text-xs text-red-950"
    >
      {{ errorMessage }}
    </p>
  </div>
</template>

<style scoped>

</style>