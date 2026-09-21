import { invoke } from "@tauri-apps/api/core";
import { computed, ref } from "vue";
import type { SongLyrics } from "@/types.ts";

export function createPlayerLyrics() {
  const lyrics = ref<SongLyrics[]>([]);
  const isLyricsLoading = ref(false);
  const lyricsError = ref<string | null>(null);
  const hasLyrics = computed(() => lyrics.value.length > 0);
  const lyricsCache = new Map<string, SongLyrics[]>();
  const lyricsRequests = new Map<string, Promise<SongLyrics[]>>();
  let lyricsSongId: string | null = null;
  let latestRequestId = 0;

  const fetchLyrics = (songId: string) => {
    const cachedLyrics = lyricsCache.get(songId);
    if (cachedLyrics) {
      return Promise.resolve(cachedLyrics);
    }

    const existingRequest = lyricsRequests.get(songId);
    if (existingRequest) {
      return existingRequest;
    }

    const request = invoke<SongLyrics[]>("get_song_lyrics", { songId })
      .then((songLyrics) => {
        lyricsCache.set(songId, songLyrics);
        return songLyrics;
      })
      .finally(() => {
        lyricsRequests.delete(songId);
      });

    lyricsRequests.set(songId, request);
    return request;
  };

  const prefetchLyrics = async (songId: string | null) => {
    if (!songId || lyricsCache.has(songId)) {
      return;
    }

    try {
      await fetchLyrics(songId);
    } catch {
      // Prefetch failures are retried when the song becomes current.
    }
  };

  const loadLyrics = async (songId: string | null) => {
    const isCurrentSongAlreadyHandled =
      songId === lyricsSongId && (isLyricsLoading.value || (songId !== null && lyricsCache.has(songId)));

    if (isCurrentSongAlreadyHandled) {
      return;
    }

    lyricsSongId = songId;
    const requestId = ++latestRequestId;
    lyrics.value = [];
    lyricsError.value = null;

    if (!songId) {
      isLyricsLoading.value = false;
      return;
    }

    if (lyricsCache.has(songId)) {
      const cachedLyrics = lyricsCache.get(songId) ?? [];
      lyrics.value = cachedLyrics;
      isLyricsLoading.value = false;
      return;
    }

    isLyricsLoading.value = true;
    let outcome: { status: "success"; lyrics: SongLyrics[] } | { status: "error"; message: string };

    try {
      outcome = {
        status: "success",
        lyrics: await fetchLyrics(songId),
      };
    } catch (error) {
      outcome = {
        status: "error",
        message: error instanceof Error ? error.message : String(error),
      };
    }

    if (requestId !== latestRequestId) {
      return;
    }

    if (outcome.status === "success") {
      lyrics.value = outcome.lyrics;
    } else {
      lyricsError.value = outcome.message;
    }

    isLyricsLoading.value = false;
  };

  return {
    lyrics,
    isLyricsLoading,
    lyricsError,
    hasLyrics,
    loadLyrics,
    prefetchLyrics,
  };
}
