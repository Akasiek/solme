<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

import SavedServerList from "@/components/Settings/SavedServerList.vue";
import ServerAuthorizationForm from "@/components/Settings/ServerAuthorizationForm.vue";
import SettingsAccountHeader from "@/components/Settings/SettingsAccountHeader.vue";
import { showToast } from "@/composables/useToast";
import { ServerInfo } from "@/types";

interface ServerAuthorizationPayload {
  profileId?: string;
  url: string;
  secondaryUrl?: string;
  username: string;
  password: string;
}

const savedServerList = ref<InstanceType<typeof SavedServerList> | null>(null);
const serverInfo = ref<ServerInfo | null>(null);
const hasProfiles = ref(false);
const isProfilesLoading = ref(true);
const isSubmitting = ref(false);
const isAddingServer = ref(false);
const error = ref<string | null>(null);

const updateProfilesState = (nextHasProfiles: boolean) => {
  hasProfiles.value = nextHasProfiles;
  isProfilesLoading.value = false;

  if (!nextHasProfiles) {
    isAddingServer.value = false;
  }
};

const saveServerAuthorization = async (payload: ServerAuthorizationPayload) => {
  isSubmitting.value = true;
  error.value = null;

  try {
    serverInfo.value = await invoke<ServerInfo>("connect_music_server", {
      config: {
        profileId: payload.profileId,
        serverType: "navidrome",
        url: payload.url,
        secondaryUrl: payload.secondaryUrl,
        username: payload.username,
        password: payload.password,
        saveCredentials: true,
      },
    });
    isAddingServer.value = false;
    showToast(payload.profileId ? "Server authorization updated." : "Server connected.");
    await savedServerList.value?.refreshProfiles();
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : "Failed to save server authorization.";
  } finally {
    isSubmitting.value = false;
  }
};

const addServer = () => {
  isAddingServer.value = true;
  error.value = null;
};
</script>

<template>
  <section class="space-y-6">
    <SettingsAccountHeader :server-info="serverInfo" :can-add-server="hasProfiles" @add-server="addServer" />

    <div
      v-if="!isProfilesLoading && hasProfiles && !serverInfo"
      class="rounded border border-amber-500/30 bg-amber-500/10 p-4 text-sm text-amber-100"
    >
      No connection to the configured music server. Check the server URL, network connection, or add a secondary URL
      for fallback access.
    </div>

    <ServerAuthorizationForm
      v-if="!isProfilesLoading && (!hasProfiles || isAddingServer)"
      :is-submitting="isSubmitting"
      :can-cancel="hasProfiles"
      @submit="saveServerAuthorization"
      @cancel="isAddingServer = false"
    />

    <SavedServerList
      ref="savedServerList"
      @profiles-loaded="updateProfilesState"
      @connection-changed="serverInfo = $event"
      @editing-started="isAddingServer = false"
    />

    <p v-if="error" class="text-sm text-red-300">{{ error }}</p>
  </section>
</template>
