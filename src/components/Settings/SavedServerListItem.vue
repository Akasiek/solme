<script setup lang="ts">
import Button from "@/components/Button.vue";
import LibrarySyncControl from "@/components/Settings/LibrarySyncControl.vue";
import ServerAuthorizationForm from "@/components/Settings/ServerAuthorizationForm.vue";
import { SavedServerProfile } from "@/types";

interface ServerAuthorizationPayload {
  profileId?: string;
  url: string;
  secondaryUrl?: string;
  username: string;
  password: string;
}

defineProps<{
  profile: SavedServerProfile;
  isActiveAction: boolean;
  isEditing: boolean;
  isSubmitting: boolean;
}>();

const emit = defineEmits<{
  connect: [];
  edit: [];
  remove: [];
  save: [payload: ServerAuthorizationPayload];
  cancelEdit: [];
}>();
</script>

<template>
  <li class="space-y-3 rounded border border-zinc-800 p-4">
    <div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
      <div class="min-w-0">
        <div class="flex items-center gap-2">
          <p class="truncate font-medium">{{ profile.username }}</p>
          <span v-if="profile.isCurrent" class="rounded bg-zinc-800 px-2 py-0.5 text-xs text-zinc-300"> Current </span>
        </div>
        <p class="truncate font-sans text-sm text-zinc-400">{{ profile.url }}</p>
        <p v-if="profile.secondaryUrl" class="truncate font-sans text-sm text-zinc-500">
          Fallback: {{ profile.secondaryUrl }}
        </p>
      </div>

      <div class="flex gap-2">
        <LibrarySyncControl v-if="profile.isCurrent" :profile-id="profile.id" />
        <Button v-else type="button" :disabled="isActiveAction" @click="emit('connect')">Connect</Button>
        <button
          type="button"
          class="rounded px-3 py-2 text-sm font-medium text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100 disabled:cursor-not-allowed disabled:opacity-50"
          :disabled="isActiveAction"
          @click="emit('edit')"
        >
          Edit
        </button>
        <button
          type="button"
          class="rounded px-3 py-2 text-sm font-medium text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100 disabled:cursor-not-allowed disabled:opacity-50"
          :disabled="isActiveAction"
          @click="emit('remove')"
        >
          Remove
        </button>
      </div>
    </div>

    <ServerAuthorizationForm
      v-if="isEditing"
      :profile="profile"
      :is-submitting="isSubmitting"
      @submit="emit('save', $event)"
      @cancel="emit('cancelEdit')"
    />
  </li>
</template>
