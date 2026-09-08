<script setup lang="ts">
import { Search, SlidersHorizontal } from "@lucide/vue";
import FormSelect from "@/components/FormSelect.vue";
import TextInput from "@/components/TextInput.vue";
import type { ArtistPageSort } from "@/types";

const query = defineModel<string>("query", { required: true });
const sort = defineModel<ArtistPageSort>("sort", { required: true });
</script>

<template>
  <aside class="space-y-6 self-start rounded-lg border border-zinc-800 bg-zinc-900/40 p-4">
    <TextInput id="artist-filter" v-model="query" field-label="Filter artists" type="search" placeholder="Artist name">
      <template #leading><Search class="size-4" /></template>
    </TextInput>
    <FormSelect v-model="sort" label="Sort by">
      <template #icon><SlidersHorizontal class="size-4" /></template>
      <option value="name">Name</option>
      <option value="most-albums">Most albums</option>
      <option value="fewest-albums">Fewest albums</option>
    </FormSelect>
  </aside>
</template>
