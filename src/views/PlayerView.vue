<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { onMounted, onUnmounted, ref } from "vue";

import { formatTime } from "../format";
import type { PlayerStatus } from "../types";

const status = ref<PlayerStatus | null>(null);
const message = ref("");
let timer: ReturnType<typeof setInterval> | undefined;

async function refresh() {
  try {
    status.value = await invoke<PlayerStatus>("get_player_status");
  } catch (error) {
    message.value = String(error);
  }
}

async function command(name: "pause" | "resume" | "stop" | "next" | "previous") {
  try {
    await invoke(name);
    await refresh();
  } catch (error) {
    message.value = String(error);
  }
}

async function seek(event: Event) {
  try {
    await invoke("seek", {
      positionSeconds: Number((event.target as HTMLInputElement).value),
    });
    await refresh();
  } catch (error) {
    message.value = String(error);
  }
}

async function setVolume(event: Event) {
  try {
    await invoke("set_volume", {
      volume: Number((event.target as HTMLInputElement).value),
    });
    await refresh();
  } catch (error) {
    message.value = String(error);
  }
}

onMounted(async () => {
  await refresh();
  timer = setInterval(refresh, 500);
});

onUnmounted(() => {
  if (timer) {
    clearInterval(timer);
  }
});
</script>

<template>
  <h2>Player</h2>
  <p>{{ message }}</p>

  <template v-if="status">
    <p v-if="status.currentSong">
      {{ status.currentSong.artistName }} - {{ status.currentSong.title }} ({{
        status.queuePosition
      }}/{{ status.queueLength }})
    </p>
    <p v-else>No track selected</p>

    <p>
      State: {{ status.state }}
      <button type="button" @click="command('previous')">Previous</button>
      <button v-if="status.state === 'paused'" type="button" @click="command('resume')">
        Resume
      </button>
      <button v-else type="button" @click="command('pause')">Pause</button>
      <button type="button" @click="command('stop')">Stop</button>
      <button type="button" @click="command('next')">Next</button>
    </p>

    <p>
      <label>
        Position:
        <input
          type="range"
          min="0"
          :max="Math.max(status.durationSeconds, 0)"
          :value="status.positionSeconds"
          @change="seek"
        />
        {{ formatTime(status.positionSeconds) }} / {{ formatTime(status.durationSeconds) }}
      </label>
    </p>

    <p>
      <label>
        Volume:
        <input type="range" min="0" max="100" :value="status.volume" @input="setVolume" />
        {{ Math.round(status.volume) }}
      </label>
    </p>
  </template>
</template>
