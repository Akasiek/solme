<script setup lang="ts">
import ModalContainer from "@/components/ModalContainer/ModalContainer.vue";
import { useHotkey } from "@/composables/useHotkey.ts";
import TextInput from "@/components/TextInput.vue";
import { ref } from "vue";

const show = defineModel<boolean>("show", { default: false });
const query = ref("");

useHotkey(
  (event) => event.key.toLowerCase() === "s" && event.shiftKey,
  () => (show.value = true),
);
</script>

<template>
  <ModalContainer :show="show" label="Search" @close="show = false">
    <div class="w-xl min-w-sm space-y-5 rounded bg-zinc-900 p-4 lg:p-8">
      <h2 class="font-serif text-xl font-semibold text-zinc-100">Search</h2>
      <TextInput v-model="query" type="search" placeholder="Search..." aria-label="Search" />
    </div>
  </ModalContainer>
</template>
