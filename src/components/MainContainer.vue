<script setup lang="ts">
import LyricsAwayModal from "@/components/Lyrics/LyricsAwayModal.vue";
</script>

<template>
  <div class="relative min-h-0 min-w-0 flex-1">
    <main class="h-full overflow-y-auto">
      <RouterView v-slot="{ Component }">
        <KeepAlive include="AlbumsView,ArtistsView">
          <component :is="Component" />
        </KeepAlive>
      </RouterView>
    </main>
    <LyricsAwayModal />
  </div>
</template>
