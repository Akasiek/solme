import { invoke } from "@tauri-apps/api/core";
import type { Router } from "vue-router";

import { useToastStore } from "@/stores/toast";
import type { SavedServerProfile } from "@/types";

const serverAuthorizationRoute = "settings-account";

export function installLibraryProfileGuard(router: Router) {
  router.beforeEach(async (to) => {
    if (!to.meta.requiresLibraryProfile) {
      return true;
    }

    try {
      if (await invoke<SavedServerProfile | null>("get_saved_server_profile")) {
        return true;
      }
    } catch (error) {
      console.error("Failed to read the saved library profile:", error);
    }

    useToastStore().show("Add a music server account to create a local library.");
    return { name: serverAuthorizationRoute, replace: true };
  });
}
