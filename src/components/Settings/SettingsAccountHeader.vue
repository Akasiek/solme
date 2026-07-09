<script setup lang="ts">
import Button from "@/components/Button.vue";
import { ServerInfo } from "@/types";

defineProps<{
  serverInfo: ServerInfo | null;
  canAddServer: boolean;
}>();

const emit = defineEmits<{
  addServer: [];
}>();
</script>

<template>
  <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
    <div class="space-y-1">
      <h2 class="font-serif text-2xl font-bold">Servers</h2>
      <p v-if="serverInfo" class="text-sm text-zinc-400">
        Connected as {{ serverInfo.username }} through {{ serverInfo.serverType }}
      </p>
      <p v-else class="text-sm text-zinc-400">No server is connected.</p>
    </div>

    <Button v-if="canAddServer" type="button" @click="emit('addServer')">Add server</Button>
  </div>
</template>
