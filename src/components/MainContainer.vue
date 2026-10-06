<script setup lang="ts">
import LyricsAwayModal from "@/components/Lyrics/LyricsAwayModal.vue";
</script>

<template>
  <div class="relative min-h-0 min-w-0 flex-1">
    <main class="absolute inset-0 overflow-y-auto">
      <RouterView v-slot="{ Component, route }">
        <KeepAlive include="AlbumsView,ArtistsView,ArtistView">
          <component :is="Component" :key="route.name === 'artist' ? route.path : undefined" />
        </KeepAlive>
      </RouterView>
    </main>
    <LyricsAwayModal />
  </div>
</template>
