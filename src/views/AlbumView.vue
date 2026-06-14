<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { onMounted, ref } from "vue";
import { RouterLink, useRouter } from "vue-router";

import { formatTime } from "../format";
import type { CachedAlbum, CachedSong } from "../types";

const props = defineProps<{ albumId: string }>();
const router = useRouter();
const album = ref<CachedAlbum | null>(null);
const songs = ref<CachedSong[]>([]);
const message = ref("");

async function loadAlbum() {
  try {
    [album.value, songs.value] = await Promise.all([
      invoke<CachedAlbum | null>("get_cached_album", { albumId: props.albumId }),
      invoke<CachedSong[]>("get_cached_songs", { albumId: props.albumId }),
    ]);
    if (!album.value) {
      message.value = "Album was not found in the cache";
    }
  } catch (error) {
    message.value = String(error);
  }
}

async function play(startSongId?: string) {
  try {
    await invoke("play_album", { albumId: props.albumId, startSongId });
    await router.push({ name: "player" });
  } catch (error) {
    message.value = String(error);
  }
}

async function queueAlbum() {
  try {
    await invoke("queue_album", { albumId: props.albumId });
    message.value = "Album added to queue";
  } catch (error) {
    message.value = String(error);
  }
}

onMounted(loadAlbum);
</script>

<template>
  <p><RouterLink :to="{ name: 'search' }">Back to search</RouterLink></p>
  <p v-if="message">{{ message }}</p>

  <template v-if="album">
    <h2>{{ album.name }}</h2>
    <p>
      {{ album.artistName }}
      <span v-if="album.year">- {{ album.year }}</span>
    </p>
    <p>{{ album.songCount }} tracks</p>
    <p>
      <button type="button" @click="play()">Play album</button>
      <button type="button" @click="queueAlbum">Add album to queue</button>
    </p>

    <ol>
      <li v-for="song in songs" :key="song.remoteId">
        <button type="button" @click="play(song.remoteId)">
          <span v-if="song.discNumber">Disc {{ song.discNumber }}, </span>
          <span v-if="song.trackNumber">{{ song.trackNumber }}. </span>
          {{ song.title }} ({{ formatTime(song.durationSeconds) }})
        </button>
      </li>
    </ol>
  </template>
</template>
