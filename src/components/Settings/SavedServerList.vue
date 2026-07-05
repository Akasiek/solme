<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

import SavedServerListItem from "@/components/Settings/SavedServerListItem.vue";
import { showToast } from "@/composables/useToast";
import { SavedServerProfile, ServerInfo } from "@/types";

interface ServerAuthorizationPayload {
  profileId?: string;
  url: string;
  username: string;
  password: string;
}

const emit = defineEmits<{
  profilesLoaded: [hasProfiles: boolean];
  connectionChanged: [serverInfo: ServerInfo | null];
  editingStarted: [];
}>();

const profiles = ref<SavedServerProfile[]>([]);
const isLoading = ref(false);
const isSubmitting = ref(false);
const activeActionProfileId = ref<string | null>(null);
const editedProfileId = ref<string | null>(null);
const error = ref<string | null>(null);

const refreshConnection = async () => {
  try {
    const serverInfo = await invoke<ServerInfo>("ping_music_server");
    emit("connectionChanged", serverInfo);
  } catch {
    emit("connectionChanged", null);
  }
};

const refreshProfiles = async () => {
  isLoading.value = true;
  error.value = null;

  try {
    profiles.value = await invoke<SavedServerProfile[]>("get_saved_server_profiles");
    emit("profilesLoaded", profiles.value.length > 0);
    await refreshConnection();
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : "Failed to load saved servers.";
    emit("profilesLoaded", false);
  } finally {
    isLoading.value = false;
  }
};

const saveServerAuthorization = async (payload: ServerAuthorizationPayload) => {
  isSubmitting.value = true;
  activeActionProfileId.value = payload.profileId ?? null;
  error.value = null;

  try {
    const serverInfo = await invoke<ServerInfo>("connect_music_server", {
      config: {
        profileId: payload.profileId,
        serverType: "navidrome",
        url: payload.url,
        username: payload.username,
        password: payload.password,
        saveCredentials: true,
      },
    });
    emit("connectionChanged", serverInfo);
    editedProfileId.value = null;
    showToast("Server authorization updated.");
    await refreshProfiles();
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : "Failed to save server authorization.";
  } finally {
    isSubmitting.value = false;
    activeActionProfileId.value = null;
  }
};

const connectSavedServer = async (profile: SavedServerProfile) => {
  activeActionProfileId.value = profile.id;
  error.value = null;

  try {
    const serverInfo = await invoke<ServerInfo>("connect_saved_music_server", {
      profileId: profile.id,
    });
    emit("connectionChanged", serverInfo);
    showToast("Server connected.");
    await refreshProfiles();
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : "Failed to connect saved server.";
  } finally {
    activeActionProfileId.value = null;
  }
};

const forgetServer = async (profile: SavedServerProfile) => {
  activeActionProfileId.value = profile.id;
  error.value = null;

  try {
    await invoke("forget_saved_server_profile", {
      profileId: profile.id,
    });
    if (profile.isCurrent) {
      emit("connectionChanged", null);
    }
    if (editedProfileId.value === profile.id) {
      editedProfileId.value = null;
    }
    showToast("Server removed.");
    await refreshProfiles();
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : "Failed to remove saved server.";
  } finally {
    activeActionProfileId.value = null;
  }
};

const editServer = (profile: SavedServerProfile) => {
  editedProfileId.value = profile.id;
  error.value = null;
  emit("editingStarted");
};

onMounted(refreshProfiles);

defineExpose({
  refreshProfiles,
});
</script>

<template>
  <div class="space-y-3">
    <h3 class="font-serif text-xl font-bold">Saved servers</h3>

    <p v-if="isLoading" class="text-sm text-zinc-400">Loading servers...</p>
    <p v-else-if="profiles.length === 0" class="text-sm text-zinc-400">No saved servers.</p>

    <ul v-else class="space-y-2">
      <SavedServerListItem
        v-for="profile in profiles"
        :key="profile.id"
        :profile="profile"
        :is-active-action="activeActionProfileId === profile.id"
        :is-editing="editedProfileId === profile.id"
        :is-submitting="isSubmitting && activeActionProfileId === profile.id"
        @connect="connectSavedServer(profile)"
        @edit="editServer(profile)"
        @remove="forgetServer(profile)"
        @save="saveServerAuthorization"
        @cancel-edit="editedProfileId = null"
      />
    </ul>

    <p v-if="error" class="text-sm text-red-300">{{ error }}</p>
  </div>
</template>
