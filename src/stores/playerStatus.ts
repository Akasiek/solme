import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { PlayerStatus } from "@/types.ts";

export const usePlayerStatusStore = defineStore("playerStatus", () => {
  const status = ref<PlayerStatus | null>(null);
  const isListening = ref(false);
  let startPromise: Promise<void> | null = null;

  const currentSong = computed(() => status.value?.currentSong ?? null);

  const load = async () => {
    status.value = await invoke<PlayerStatus>("get_player_status");
  };

  const startListening = async () => {
    if (isListening.value) {
      return;
    }

    if (startPromise) {
      return startPromise;
    }

    startPromise = (async () => {
      await load();
      await listen<PlayerStatus>("player-status-changed", (event) => {
        status.value = event.payload;
      });
      isListening.value = true;
      startPromise = null;
    })();

    try {
      await startPromise;
    } catch (error) {
      startPromise = null;
      throw error;
    }
  };

  return {
    status,
    currentSong,
    startListening,
  };
});
