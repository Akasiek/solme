<script setup lang="ts">
import ModalContainer from "@/components/ModalContainer/ModalContainer.vue";
import SearchSection from "@/components/Search/SearchSection";
import { useHotkey } from "@/composables/useHotkey.ts";

const show = defineModel<boolean>("show", { default: false });

const closeModal = () => {
  show.value = false;
};

useHotkey(
  (event) => event.key.toLowerCase() === "s" && event.shiftKey,
  () => (show.value = true),
);
</script>

<template>
  <ModalContainer :show="show" label="Search" @close="closeModal">
    <div
      class="flex max-h-[calc(100dvh-4rem)] max-w-[min(80rem,calc(100vw-4rem))] min-w-xl flex-col overflow-hidden rounded border border-zinc-800 bg-zinc-900 text-zinc-100 shadow-2xl shadow-black/40"
    >
      <SearchSection variant="modal" :is-open="show" @navigate="closeModal" />
    </div>
  </ModalContainer>
</template>
