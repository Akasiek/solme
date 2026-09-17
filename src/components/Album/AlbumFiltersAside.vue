<script setup lang="ts">
import { Check, Disc3, Search, SlidersHorizontal } from "@lucide/vue";
import FormSelect from "@/components/FormSelect.vue";
import CatalogFilters from "@/components/CatalogFilters.vue";
import TextInput from "@/components/TextInput.vue";
import type { AlbumPageSort, CatalogFilter } from "@/types";

defineProps<{ albumTypes: string[]; genres: string[] }>();
const query = defineModel<string>("query", { required: true });
const selectedAlbumTypes = defineModel<string[]>("selectedAlbumTypes", { required: true });
const filters = defineModel<CatalogFilter>("filters", { required: true });
const sort = defineModel<AlbumPageSort>("sort", { required: true });

const sortOptions = [
  { value: "artist", label: "Artist" },
  { value: "title", label: "Album title" },
  { value: "newest", label: "Newest release" },
  { value: "oldest", label: "Oldest release" },
  { value: "recently-added", label: "Recently added" },
  { value: "recently-played", label: "Recently played" },
  { value: "most-played", label: "Most played" },
] as const;
</script>

<template>
  <aside class="space-y-6 self-start rounded-lg border border-zinc-800 bg-zinc-900/40 p-4">
    <TextInput
      id="album-filter"
      v-model="query"
      field-label="Filter albums"
      label-font="serif"
      type="search"
      placeholder="Album or artist"
    >
      <template #label-icon><Search class="mt-0.5 size-4 shrink-0" aria-hidden="true" /></template>
    </TextInput>
    <FormSelect v-model="sort" label="Sort by" :options="sortOptions">
      <template #icon><SlidersHorizontal class="mt-0.5 size-4 shrink-0" aria-hidden="true" /></template>
    </FormSelect>
    <fieldset>
      <legend class="mb-2 font-serif text-sm font-bold text-zinc-300">
        <span class="flex items-center gap-1.5">
          <Disc3 class="mt-0.5 size-4 shrink-0" aria-hidden="true" />Album type
        </span>
      </legend>
      <div class="max-h-44 space-y-1 overflow-y-auto pr-1">
        <label
          v-for="type in albumTypes"
          :key="type"
          class="flex cursor-pointer items-center gap-2.5 rounded px-2 py-1.5 text-sm text-zinc-300 transition-colors hover:bg-zinc-800 hover:text-white"
        >
          <span class="relative size-4 shrink-0">
            <input
              v-model="selectedAlbumTypes"
              type="checkbox"
              :value="type"
              class="peer size-4 cursor-pointer appearance-none rounded border border-zinc-600 bg-zinc-950 transition-colors checked:border-accent checked:bg-accent focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
            />
            <Check
              aria-hidden="true"
              class="pointer-events-none absolute inset-0 size-4 stroke-3 text-white opacity-0 transition-opacity peer-checked:opacity-100"
            />
          </span>
          <span class="truncate font-sans capitalize" :title="type">{{ type }}</span>
        </label>
      </div>
    </fieldset>
    <CatalogFilters v-model="filters" :genres="genres" />
  </aside>
</template>
