import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { ref } from "vue";

import type { AppSettings } from "@/types";
import { useToastStore } from "@/stores/toast";

const defaultSettings = (): AppSettings => ({
  awayLyricsEnabled: true,
});

const errorMessage = (cause: unknown) =>
  typeof cause === "string" ? cause : cause instanceof Error ? cause.message : "Unexpected error.";

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<AppSettings>(defaultSettings());
  const isLoaded = ref(false);
  const isLoading = ref(false);
  const isSaving = ref(false);
  const error = ref<string | null>(null);
  const toastStore = useToastStore();
  let loadPromise: Promise<void> | null = null;

  const load = () => {
    if (isLoaded.value) {
      return Promise.resolve();
    }

    if (loadPromise) {
      return loadPromise;
    }

    loadPromise = (async () => {
      isLoading.value = true;
      error.value = null;

      try {
        settings.value = await invoke<AppSettings>("get_app_settings");
        isLoaded.value = true;
      } catch (cause) {
        error.value = errorMessage(cause);
      } finally {
        isLoading.value = false;
        loadPromise = null;
      }
    })();

    return loadPromise;
  };

  const retryLoad = () => {
    isLoaded.value = false;
    return load();
  };

  const updateSetting = async <Key extends keyof AppSettings>(key: Key, value: AppSettings[Key]) => {
    if (isSaving.value || settings.value[key] === value) {
      return;
    }

    const previousSettings = settings.value;
    settings.value = { ...settings.value, [key]: value };
    isSaving.value = true;
    error.value = null;

    try {
      settings.value = await invoke<AppSettings>("update_app_setting", {
        update: { key, value },
      });
      isLoaded.value = true;
    } catch (cause) {
      settings.value = previousSettings;
      error.value = errorMessage(cause);
      toastStore.show("Could not save setting.");
    } finally {
      isSaving.value = false;
    }
  };

  return {
    settings,
    isLoaded,
    isLoading,
    isSaving,
    error,
    load,
    retryLoad,
    updateSetting,
  };
});
