<script setup lang="ts">
import { Search, SlidersHorizontal } from "@lucide/vue";
import FormSelect from "@/components/FormSelect.vue";
import CatalogFilters from "@/components/CatalogFilters.vue";
import TextInput from "@/components/TextInput.vue";
import type { ArtistPageSort, CatalogFilter } from "@/types";

defineProps<{ genres: string[] }>();

const query = defineModel<string>("query", { required: true });
const filters = defineModel<CatalogFilter>("filters", { required: true });
const sort = defineModel<ArtistPageSort>("sort", { required: true });

const sortOptions = [
  { value: "name", label: "Name" },
  { value: "most-albums", label: "Most albums" },
  { value: "fewest-albums", label: "Fewest albums" },
  { value: "recently-played", label: "Recently played" },
  { value: "most-played", label: "Most played" },
  { value: "recently-added", label: "Recently added" },
] as const;
</script>

<template>
  <aside class="space-y-6 self-start rounded-lg border border-zinc-800 bg-zinc-900/40 p-4">
    <TextInput
      id="artist-filter"
      v-model="query"
      field-label="Filter artists"
      label-font="serif"
      type="search"
      placeholder="Artist name"
    >
      <template #label-icon><Search class="mt-0.5 size-4 shrink-0" aria-hidden="true" /></template>
    </TextInput>
    <FormSelect v-model="sort" label="Sort by" :options="sortOptions">
      <template #icon><SlidersHorizontal class="mt-0.5 size-4 shrink-0" aria-hidden="true" /></template>
    </FormSelect>
    <CatalogFilters v-model="filters" :genres="genres" />
  </aside>
</template>
