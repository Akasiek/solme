<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { Disc3, ListEnd, ListStart, Play, Trash2, UserRound } from "@lucide/vue";
import { computed, ref, watch, type Component } from "vue";
import { isNavigationFailure, NavigationFailureType, useRouter, type RouteLocationRaw } from "vue-router";
import ContextMenu from "@/components/ContextMenu";
import { LibraryItemAnnotations } from "@/components/LibraryItemAnnotations";
import type { CachedSong } from "@/types";
import { usePlayerStore } from "@/stores/player";

interface MenuItem {
  id: string;
  label: string;
  icon: Component;
  action: () => Promise<unknown>;
  disabled: boolean;
  destructive?: boolean;
  separatorBefore?: boolean;
}

type MenuVariant = "library" | "queue";

const props = defineProps<{
  song: CachedSong;
  variant: MenuVariant;
  entryId?: string;
  open: boolean;
  x: number;
  y: number;
  disabled?: boolean;
}>();
const emit = defineEmits<{ close: []; navigate: [] }>();
const player = usePlayerStore();
const router = useRouter();
const busy = ref(false);
const error = ref<string | null>(null);

const queueEntryId = () => {
  if (!props.entryId) {
    throw new Error("This song is no longer in the queue");
  }
  return props.entryId;
};

const menuConfig = computed<Record<MenuVariant, MenuItem[]>>(() => {
  const navigationItems = getNavigationItems();

  return {
    library: [
      {
        id: "play",
        label: "Play now",
        icon: Play,
        action: () => invoke("player_play_album", { albumId: props.song.albumId, startSongId: props.song.remoteId }),
        disabled: Boolean(props.disabled),
      },
      {
        id: "queue-next",
        label: "Queue next",
        icon: ListStart,
        action: () => player.queueSong(props.song, true),
        disabled: player.status?.state === "loading",
      },
      {
        id: "queue-last",
        label: "Queue last",
        icon: ListEnd,
        action: () => player.queueSong(props.song, false),
        disabled: player.status?.state === "loading",
      },
      ...navigationItems,
    ],
    queue: [
      {
        id: "play",
        label: "Play now",
        icon: Play,
        action: () => player.playQueueEntry(queueEntryId()),
        disabled: Boolean(props.disabled) || !props.entryId,
      },
      {
        id: "remove",
        label: "Remove from queue",
        icon: Trash2,
        action: () => player.removeQueueEntry(queueEntryId()),
        disabled: Boolean(props.disabled) || !props.entryId,
        destructive: true,
        separatorBefore: true,
      },
      ...navigationItems,
    ],
  };
});

function getNavigationItems() {
  const navigationItems: MenuItem[] = [];
  const artistId = props.song.artistId;

  if (artistId) {
    navigationItems.push({
      id: "go-to-artist",
      label: "Go to artist",
      icon: UserRound,
      action: () => navigate({ name: "artist", params: { artistId } }),
      disabled: false,
      separatorBefore: true,
    });
  }

  navigationItems.push({
    id: "go-to-album",
    label: "Go to album",
    icon: Disc3,
    action: () => navigate({ name: "album", params: { albumId: props.song.albumId } }),
    disabled: false,
    separatorBefore: navigationItems.length === 0,
  });
  return navigationItems;
}

async function navigate(to: RouteLocationRaw) {
  const failure = await router.push(to);
  if (failure && !isNavigationFailure(failure, NavigationFailureType.duplicated)) {
    throw failure;
  }
  emit("navigate");
}

watch(
  () => props.open,
  () => {
    error.value = null;
  },
);

async function run(action: () => Promise<unknown>) {
  if (busy.value) {
    return;
  }
  busy.value = true;
  error.value = null;
  try {
    await action();
    emit("close");
  } catch (cause) {
    error.value = String(cause);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <ContextMenu :open="open" :x="x" :y="y" :aria-label="`${song.title} actions`" @close="emit('close')">
    <div class="w-60" :aria-busy="busy">
      <div class="px-2 py-1.5">
        <p class="truncate text-xs font-medium text-zinc-200" :title="song.title">{{ song.title }}</p>
        <p class="mt-0.5 truncate text-xs text-zinc-500">{{ song.artistName }}</p>
      </div>
      <div role="separator" class="my-1 border-t border-zinc-800" />
      <template v-for="item in menuConfig[variant]" :key="item.id">
        <div v-if="item.separatorBefore" role="separator" class="my-1 border-t border-zinc-800" />
        <button
          type="button"
          role="menuitem"
          :disabled="busy || item.disabled"
          class="flex w-full cursor-pointer items-center gap-2.5 rounded px-2 py-2 text-left text-sm outline-none hover:bg-zinc-800 focus-visible:bg-zinc-800 disabled:cursor-default disabled:opacity-40"
          :class="item.destructive ? 'text-red-300' : 'text-zinc-200'"
          @click="run(item.action)"
        >
          <component :is="item.icon" class="size-4 shrink-0" aria-hidden="true" />
          {{ item.label }}
        </button>
      </template>
      <div role="separator" class="my-1 border-t border-zinc-800" />
      <div class="px-2 py-1.5"><LibraryItemAnnotations :item="song" /></div>
      <p v-if="error" role="alert" class="px-2 py-1.5 text-xs text-red-300">{{ error }}</p>
    </div>
  </ContextMenu>
</template>
