import { computed, onScopeDispose, ref } from "vue";
import type { PlayerStatus } from "@/types.ts";

export function createPlayerPlayback() {
  const status = ref<PlayerStatus | null>(null);
  const playbackPositionSeconds = ref(0);
  const currentSong = computed(() => status.value?.currentSong ?? null);
  let lastPositionUpdate = performance.now();

  const updateStatus = (playerStatus: PlayerStatus) => {
    status.value = playerStatus;
    playbackPositionSeconds.value = playerStatus.positionSeconds;
    lastPositionUpdate = performance.now();
  };

  const progressTimer = window.setInterval(() => {
    const now = performance.now();
    const elapsedSeconds = (now - lastPositionUpdate) / 1000;
    lastPositionUpdate = now;

    if (status.value?.state !== "playing") {
      return;
    }

    playbackPositionSeconds.value = Math.min(
      status.value.durationSeconds,
      playbackPositionSeconds.value + elapsedSeconds,
    );
  }, 250);

  onScopeDispose(() => window.clearInterval(progressTimer));

  return {
    status,
    playbackPositionSeconds,
    currentSong,
    updateStatus,
  };
}
