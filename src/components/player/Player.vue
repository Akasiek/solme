<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted, onUnmounted, ref } from "vue";

import PlayerButton from "@/components/player/PlayerButton.vue";
import PlayerDisplay from "@/components/player/PlayerDisplay.vue";
import PlayerSeekVolume from "@/components/player/PlayerSeekVolume.vue";
import PlayIcon from "@/components/icons/PlayIcon.vue";
import StopIcon from "@/components/icons/StopIcon.vue";
import PrevIcon from "@/components/icons/PrevIcon.vue";
import NextIcon from "@/components/icons/NextIcon.vue";
import PauseIcon from "@/components/icons/PauseIcon.vue";
import type { PlayerStatus } from "@/types";
import { formatTime } from "@/utils/format";

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

async function seek(position: number) {
  try {
    await invoke("seek", { positionSeconds: position });
    await refreshPlayerStatus();
  } catch (error) {
    errorMessage.value = String(error);
  }
}

async function setVolume(volume: number) {
  try {
    await invoke("set_volume", { volume });
  } catch (error) {
    errorMessage.value = String(error);
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
    <section
      class="player-console relative isolate w-full max-w-2xl overflow-hidden rounded-sm bg-repeat p-3"
    >
      <PlayerDisplay
        :track="displayTrack"
        :elapsed="displayElapsed"
        :title="displayTitle"
        :artist="displayArtist"
        :album="displayAlbum"
        :state="displayState"
        embedded
      />

      <PlayerSeekVolume
        :position="status?.positionSeconds"
        :duration="status?.durationSeconds"
        :volume="status?.volume"
        :disabled="!hasTrack"
        embedded
        @seek="seek"
        @volume="setVolume"
      />
    </section>

    <div class="flex max-w-full gap-3">
      <div class="flex flex-col items-center gap-2">
        <span class="button-label font-display-condensed text-xs tracking-widest uppercase">
          Stop
        </span>
        <PlayerButton
          label="Stop"
          :disabled="controlState === 'stopped'"
          @click="runCommand('stop')"
        >
          <StopIcon />
        </PlayerButton>
      </div>
      <div class="flex flex-col items-center gap-2">
        <span class="button-label font-display-condensed text-xs tracking-widest uppercase">
          Prev
        </span>
        <PlayerButton
          label="Previous track"
          :disabled="!canPrevious"
          @click="runCommand('previous')"
        >
          <PrevIcon />
        </PlayerButton>
      </div>
      <div class="flex flex-col items-center gap-2">
        <span class="button-label font-display-condensed text-xs tracking-widest uppercase">
          Play
        </span>
        <PlayerButton
          label="Play"
          :disabled="controlState === 'playing' || !hasTrack"
          @click="runCommand('resume')"
        >
          <PlayIcon />
        </PlayerButton>
      </div>
      <div class="flex flex-col items-center gap-2">
        <span class="button-label font-display-condensed text-xs tracking-widest uppercase">
          Next
        </span>
        <PlayerButton label="Next track" :disabled="!canNext" @click="runCommand('next')">
          <NextIcon />
        </PlayerButton>
      </div>
      <div class="flex flex-col items-center gap-2">
        <span class="button-label font-display-condensed text-xs tracking-widest uppercase">
          Pause
        </span>
        <PlayerButton
          label="Pause"
          :disabled="controlState !== 'playing'"
          @click="runCommand('pause')"
        >
          <PauseIcon />
        </PlayerButton>
      </div>
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
.player-console {
  background-image: url("/assets/images/texture_bg_2.webp");
  background-size: 200%;
  box-shadow:
    0 7px 9px rgba(0, 0, 0, 0.28),
    0 0 2px 1px rgba(255, 255, 255, 0.35),
    0 0 1px 2px rgba(0, 0, 0, 0.75),
    inset 0 0 1px 1px rgba(0, 0, 0, 0.25);
}

.player-console::before {
  content: "";
  position: absolute;
  z-index: 2;
  top: 0;
  inset-inline: 0;
  height: 3px;
  pointer-events: none;
  background: linear-gradient(
      to bottom,
      rgba(255, 255, 255, 1) 0%,
      rgba(255, 255, 255, 0.6) 70%,
      rgba(255, 255, 255, 0.4) 90%,
      rgba(0, 0, 0, 0) 100%
  );
}

.player-console::after {
  content: "";
  position: absolute;
  z-index: 2;
  bottom: 0;
  inset-inline: 0;
  height: 3px;
  pointer-events: none;
  background: linear-gradient(
      to top,
      rgba(0, 0, 0, 0.6) 0%,
      rgba(0, 0, 0, 0.4) 70%,
      rgba(0, 0, 0, 0.2) 90%,
      rgba(0, 0, 0, 0) 100%
  );
}

.player-console > * {
  position: relative;
  z-index: 1;
}

.button-label {
  color: rgb(38 38 35);
  text-shadow: 0 1px 0 rgba(255, 255, 255, 0.65);
}
</style>
