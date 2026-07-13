<script setup lang="ts">
import AsideMenu from "@/components/AsideMenu";
import PlayerBar from "@/components/PlayerBar";
import ServerConnectionOverlay from "@/components/ServerConnectionOverlay.vue";
import ToastHost from "@/components/ToastHost.vue";
</script>

<template>
  <div class="flex h-dvh flex-col bg-zinc-950 font-serif text-zinc-100">
    <div class="flex min-h-0 flex-1">
      <AsideMenu />
      <main class="min-w-0 flex-1 overflow-y-auto">
        <RouterView />
      </main>
    </div>
    <PlayerBar />
    <ServerConnectionOverlay />
    <ToastHost />
  </div>
</template>
