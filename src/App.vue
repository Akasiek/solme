<script setup lang="ts">
import AppTitleBar from "@/components/AppTitleBar.vue";
import LeftAsideMenu from "@/components/LeftAsideMenu";
import PlayerBar from "@/components/PlayerBar";
import SearchModal from "@/components/Search/SearchModal";
import ServerConnectionOverlay from "@/components/ServerConnectionOverlay.vue";
import ToastHost from "@/components/ToastHost.vue";
import { useSearchModal } from "@/composables/useSearchModal";

const { isSearchModalOpen } = useSearchModal();
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
    </div>
    <PlayerBar />
    <ServerConnectionOverlay />
    <ToastHost />
  </div>
</template>
