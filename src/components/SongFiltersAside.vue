<script setup lang="ts">
import { Search, SlidersHorizontal } from "@lucide/vue";
import CatalogFilters from "@/components/CatalogFilters.vue";
import FormSelect from "@/components/FormSelect.vue";
import TextInput from "@/components/TextInput.vue";
import type { CatalogFilter, SongPageSort } from "@/types";

defineProps<{ genres: string[] }>();
const query = defineModel<string>("query", { required: true });
const filters = defineModel<CatalogFilter>("filters", { required: true });
const sort = defineModel<SongPageSort>("sort", { required: true });
const sortOptions = [
  { value: "title", label: "Title" },
  { value: "artist", label: "Artist" },
  { value: "album", label: "Album" },
  { value: "newest", label: "Newest release" },
  { value: "oldest", label: "Oldest release" },
] as const;
</script>

<template>
  <aside class="space-y-6 self-start rounded-lg border border-zinc-800 bg-zinc-900/40 p-4">
    <TextInput
      id="song-filter"
      v-model="query"
      field-label="Filter songs"
      label-font="serif"
      type="search"
      placeholder="Song, artist or album"
    >
      <template #label-icon><Search class="mt-0.5 size-4 shrink-0" aria-hidden="true" /></template>
    </TextInput>
    <FormSelect v-model="sort" label="Sort by" :options="sortOptions">
      <template #icon><SlidersHorizontal class="mt-0.5 size-4 shrink-0" aria-hidden="true" /></template>
    </FormSelect>
    <CatalogFilters v-model="filters" :genres="genres" :show-playback="false" />
  </aside>
</template>
