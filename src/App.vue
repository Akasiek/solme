<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";

const path = ref("");
const volume = ref(70);
const status = ref("");

async function play() {
  try {
    await invoke("play_file", { path: path.value });
    status.value = "Playing";
  } catch (error) {
    status.value = String(error);
  }
}

async function pause() {
  try {
    await invoke("pause");
    status.value = "Paused";
  } catch (error) {
    status.value = String(error);
  }
}

async function resume() {
  try {
    await invoke("resume");
    status.value = "Playing";
  } catch (error) {
    status.value = String(error);
  }
}

async function stop() {
  try {
    await invoke("stop");
    status.value = "Stopped";
  } catch (error) {
    status.value = String(error);
  }
}

async function setVolume() {
  try {
    await invoke("set_volume", { volume: Number(volume.value) });
  } catch (error) {
    status.value = String(error);
  }
}
</script>

<template>
  <main>
    <h1>Solme</h1>

    <p>
      <label>
        Audio file:
        <input v-model="path" type="text" />
      </label>
      <button type="button" @click="play">Play</button>
    </p>

    <p>
      <button type="button" @click="pause">Pause</button>
      <button type="button" @click="resume">Resume</button>
      <button type="button" @click="stop">Stop</button>
    </p>

    <p>
      <label>
        Volume: {{ volume }}
        <input v-model="volume" type="range" min="0" max="100" @input="setVolume" />
      </label>
    </p>

    <p>{{ status }}</p>
  </main>
</template>
