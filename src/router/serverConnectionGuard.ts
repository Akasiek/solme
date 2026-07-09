import { invoke } from "@tauri-apps/api/core";
import type { Router } from "vue-router";

import { useServerConnectionStatus } from "@/composables/useServerConnectionStatus";
import { showToast } from "@/composables/useToast";
import type { SavedServerProfile } from "@/types";

const serverAuthorizationRoute = "settings-account";
const failedMessageDelayMs = 1_200;

const delay = (ms: number) => new Promise((resolve) => window.setTimeout(resolve, ms));

export function installServerConnectionGuard(router: Router) {
  router.beforeEach(async (to) => {
    if (!to.meta.requiresServer) {
      return true;
    }

    if (await ensureServerConnection()) {
      return true;
    }

    showToast("No connection to the music server. Check the saved server settings.");
    return { name: serverAuthorizationRoute, replace: true };
  });
}

async function ensureServerConnection() {
  const { connectPrimary, clearConnectionStatus } = useServerConnectionStatus();

  try {
    await invoke("ping_music_server");
    clearConnectionStatus();
    return true;
  } catch {
    const profile = await savedServerProfile();

    try {
      connectPrimary(profile?.url, profile?.secondaryUrl);
      await connectSavedEndpoint(profile?.id ?? null, "primary");
      clearConnectionStatus();
      return true;
    } catch (primaryCause) {
      if (profile?.secondaryUrl) {
        return connectSecondaryEndpoint(profile);
      }

      await failConnectionWithDelay(primaryCause);
      return false;
    }
  }
}

async function connectSecondaryEndpoint(profile: SavedServerProfile) {
  const { connectSecondary, clearConnectionStatus } = useServerConnectionStatus();

  try {
    connectSecondary();
    await connectSavedEndpoint(profile.id, "secondary");
    clearConnectionStatus();
    return true;
  } catch (secondaryCause) {
    await failConnectionWithDelay(secondaryCause);
    return false;
  }
}

async function savedServerProfile() {
  try {
    return await invoke<SavedServerProfile | null>("get_saved_server_profile");
  } catch {
    return null;
  }
}

async function connectSavedEndpoint(profileId: string | null, endpoint: "primary" | "secondary") {
  await invoke("connect_saved_music_server_endpoint", {
    profileId,
    endpoint,
  });
}

async function failConnectionWithDelay(cause: unknown) {
  const { failConnection, clearConnectionStatus } = useServerConnectionStatus();
  const message = cause instanceof Error ? cause.message : "Check the saved server settings.";

  failConnection(message);
  await delay(failedMessageDelayMs);
  clearConnectionStatus();
}
