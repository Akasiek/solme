<script setup lang="ts">
import { reactive, watch } from "vue";

import Button from "@/components/Button.vue";
import TextInput from "@/components/TextInput.vue";
import { SavedServerProfile } from "@/types";

const props = defineProps<{
  profile?: SavedServerProfile | null;
  isSubmitting?: boolean;
  canCancel?: boolean;
}>();

const emit = defineEmits<{
  submit: [
    payload: {
      profileId?: string;
      url: string;
      username: string;
      password: string;
    },
  ];
  cancel: [];
}>();

const form = reactive({
  url: "",
  username: "",
  password: "",
});

const resetForm = () => {
  form.url = props.profile?.url ?? "";
  form.username = props.profile?.username ?? "";
  form.password = "";
};

const submit = () => {
  emit("submit", {
    profileId: props.profile?.id,
    url: form.url,
    username: form.username,
    password: form.password,
  });
};

watch(() => props.profile, resetForm, { immediate: true });
</script>

<template>
  <form class="space-y-4 rounded border border-zinc-800 p-4" @submit.prevent="submit">
    <div class="grid gap-4 md:grid-cols-2">
      <TextInput
        v-model="form.url"
        field-label="Server URL"
        required
        type="url"
        placeholder="https://music.example.com"
      />

      <TextInput v-model="form.username" field-label="Username" required autocomplete="username" />

      <TextInput
        v-model="form.password"
        field-label="Password"
        required
        type="password"
        autocomplete="current-password"
        class="md:col-span-2"
      />
    </div>

    <div class="flex gap-2">
      <Button type="submit" :disabled="isSubmitting">
        {{ isSubmitting ? "Connecting..." : profile ? "Save and connect" : "Connect server" }}
      </Button>
      <button
        v-if="profile || canCancel"
        type="button"
        class="rounded px-3 py-2 text-sm font-medium text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100"
        @click="emit('cancel')"
      >
        Cancel
      </button>
    </div>
  </form>
</template>
