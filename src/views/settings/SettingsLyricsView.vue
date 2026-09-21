<script setup lang="ts">
import { onMounted } from "vue";
import { storeToRefs } from "pinia";

import Button from "@/components/Button.vue";
import SwitchField from "@/components/SwitchField.vue";
import { useSettingsStore } from "@/stores/settings";

const settingsStore = useSettingsStore();
const { settings, isLoaded, isLoading, isSaving, error } = storeToRefs(settingsStore);

onMounted(() => {
  void settingsStore.load();
});
</script>

<template>
  <section class="space-y-6">
    <div class="space-y-1">
      <h2 class="font-serif text-2xl font-semibold text-zinc-100">Lyrics</h2>
      <p class="font-sans text-sm text-zinc-400">Control how lyrics are displayed while listening.</p>
    </div>

    <SwitchField
      :model-value="settings.awayLyricsEnabled"
      label="Show Away Lyrics"
      description="Show synchronized lyrics after the application window has been unfocused for 3 seconds."
      :disabled="!isLoaded || isLoading || isSaving"
      @update:model-value="settingsStore.updateSetting('awayLyricsEnabled', $event)"
    />

    <div
      v-if="error"
      role="alert"
      class="flex items-center justify-between gap-4 rounded border border-red-500/30 bg-red-500/10 p-4 font-sans text-sm text-red-100"
    >
      <span>Lyrics settings could not be {{ isLoaded ? "saved" : "loaded" }}. {{ error }}</span>
      <Button v-if="!isLoaded" type="button" variant="outline" :disabled="isLoading" @click="settingsStore.retryLoad">
        {{ isLoading ? "Retrying..." : "Retry" }}
      </Button>
    </div>
  </section>
</template>
