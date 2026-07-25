<script setup lang="ts">
import { storeToRefs } from "pinia";
import AppTitleBar from "@/components/AppTitleBar.vue";
import LeftAsideMenu from "@/components/LeftAsideMenu";
import RightAsideMenu from "@/components/RightAsideMenu";
import PlayerBar from "@/components/PlayerBar";
import SearchModal from "@/components/Search/SearchModal";
import ServerConnectionOverlay from "@/components/ServerConnectionOverlay.vue";
import ToastHost from "@/components/ToastHost.vue";
import { useLayoutStore } from "@/stores/layout";

const layoutStore = useLayoutStore();
const { isSearchModalOpen } = storeToRefs(layoutStore);
</script>

<template>
  <div class="flex h-dvh flex-col bg-zinc-950 font-serif text-zinc-100">
    <SearchModal v-model:show="isSearchModalOpen" />
    <AppTitleBar />
    <div class="flex min-h-0 flex-1">
      <LeftAsideMenu />
      <main class="min-w-0 flex-1 overflow-y-auto">
        <RouterView />
      </main>
      <RightAsideMenu />
    </div>
    <PlayerBar />
    <ServerConnectionOverlay />
    <ToastHost />
  </div>
</template>
