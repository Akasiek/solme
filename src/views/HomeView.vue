<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { onMounted, onUnmounted, ref } from "vue";

import type { LibrarySummary, LibrarySyncStatus, SavedServerProfile, ServerInfo } from "@/types";

const serverType = ref("navidrome");
const serverUrl = ref("");
const username = ref("");
const password = ref("");
const saveCredentials = ref(true);
const hasSavedProfile = ref(false);
const message = ref("");
const syncStatus = ref<LibrarySyncStatus | null>(null);
const librarySummary = ref<LibrarySummary | null>(null);
let statusTimer: ReturnType<typeof setInterval> | undefined;

function describeServer(info: ServerInfo) {
  const version = info.serverVersion ? ` ${info.serverVersion}` : "";
  return `${info.serverType}${version}, user ${info.username}, API ${info.apiVersion}`;
}

async function connectMusicServer() {
  try {
    const info = await invoke<ServerInfo>("connect_music_server", {
      config: {
        serverType: serverType.value,
        url: serverUrl.value,
        username: username.value,
        password: password.value,
        saveCredentials: saveCredentials.value,
      },
    });
    password.value = "";
    hasSavedProfile.value = saveCredentials.value;
    message.value = `Connected to ${describeServer(info)}`;
  } catch (error) {
    message.value = String(error);
  }
}

async function pingMusicServer() {
  try {
    const info = await invoke<ServerInfo>("ping_music_server");
    message.value = `Connection OK: ${describeServer(info)}`;
  } catch (error) {
    message.value = String(error);
  }
}

async function connectSavedMusicServer() {
  try {
    const info = await invoke<ServerInfo>("connect_saved_music_server");
    message.value = `Connected to ${describeServer(info)}`;
  } catch (error) {
    message.value = String(error);
  }
}

async function forgetSavedServerProfile() {
  try {
    await invoke("forget_saved_server_profile");
    hasSavedProfile.value = false;
    message.value = "Saved credentials removed";
  } catch (error) {
    message.value = String(error);
  }
}

async function syncLibrary(force: boolean) {
  try {
    await invoke("sync_library", { force });
    await refreshLibraryStatus();
  } catch (error) {
    message.value = String(error);
  }
}

async function refreshLibraryStatus() {
  try {
    const previousPhase = syncStatus.value?.phase;
    syncStatus.value = await invoke<LibrarySyncStatus>("get_library_sync_status");
    if (
      previousPhase !== syncStatus.value.phase ||
      syncStatus.value.phase === "completed" ||
      syncStatus.value.phase === "failed"
    ) {
      await refreshLibrarySummary();
    }
  } catch (error) {
    message.value = String(error);
  }
}

async function refreshLibrarySummary() {
  librarySummary.value = await invoke<LibrarySummary>("get_library_summary");
}

onMounted(async () => {
  try {
    const profile = await invoke<SavedServerProfile | null>("get_saved_server_profile");
    if (profile) {
      serverType.value = profile.serverType;
      serverUrl.value = profile.url;
      username.value = profile.username;
      hasSavedProfile.value = true;
    }
  } catch (error) {
    message.value = String(error);
  }

  await refreshLibrarySummary();
  await refreshLibraryStatus();
  statusTimer = setInterval(refreshLibraryStatus, 1000);
});

onUnmounted(() => {
  if (statusTimer) {
    clearInterval(statusTimer);
  }
});
</script>

<template>
  <h2>Music server</h2>

  <p>
    <label>
      Server type:
      <select v-model="serverType">
        <option value="navidrome">Navidrome</option>
      </select>
    </label>
  </p>
  <p>
    <label>
      Server URL:
      <input v-model="serverUrl" type="url" placeholder="https://music.example.com" />
    </label>
  </p>
  <p>
    <label>
      Username:
      <input v-model="username" type="text" />
    </label>
  </p>
  <p>
    <label>
      Password:
      <input v-model="password" type="password" />
    </label>
  </p>
  <p>
    <label>
      <input v-model="saveCredentials" type="checkbox" />
      Remember credentials
    </label>
  </p>
  <p>
    <button type="button" @click="connectMusicServer">Connect</button>
    <button type="button" @click="pingMusicServer">Ping</button>
  </p>
  <p v-if="hasSavedProfile">
    <button type="button" @click="connectSavedMusicServer">Connect saved profile</button>
    <button type="button" @click="forgetSavedServerProfile">Forget saved profile</button>
  </p>
  <p>{{ message }}</p>

  <h2>Library cache</h2>
  <p>
    <button type="button" @click="syncLibrary(false)">Sync library</button>
    <button type="button" @click="syncLibrary(true)">Force full sync</button>
  </p>
  <p v-if="syncStatus">
    Phase: {{ syncStatus.phase }}; artists: {{ syncStatus.processedArtists }}; albums:
    {{ syncStatus.processedAlbums }}; songs: {{ syncStatus.processedSongs }}; artwork:
    {{ syncStatus.processedArtwork }}/{{ syncStatus.totalArtwork }}
  </p>
  <p v-if="syncStatus?.lastError">{{ syncStatus.lastError }}</p>
  <p v-if="librarySummary">
    Cached artists: {{ librarySummary.artistCount }}, albums: {{ librarySummary.albumCount }},
    songs: {{ librarySummary.songCount }}
  </p>
</template>
