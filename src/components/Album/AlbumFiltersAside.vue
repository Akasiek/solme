<script setup lang="ts">
import { Check, Search, SlidersHorizontal } from "@lucide/vue";
import FormSelect from "@/components/FormSelect.vue";
import TextInput from "@/components/TextInput.vue";
import type { AlbumPageSort } from "@/types";

defineProps<{ albumTypes: string[] }>();
const query = defineModel<string>("query", { required: true });
const selectedAlbumTypes = defineModel<string[]>("selectedAlbumTypes", { required: true });
const sort = defineModel<AlbumPageSort>("sort", { required: true });
</script>

<template>
  <aside class="space-y-6 self-start rounded-lg border border-zinc-800 bg-zinc-900/40 p-4">
    <TextInput
      id="album-filter"
      v-model="query"
      field-label="Filter albums"
      type="search"
      placeholder="Album or artist"
    >
      <template #leading><Search class="size-4" /></template>
    </TextInput>
    <fieldset>
      <legend class="mb-2 font-sans text-sm font-bold text-zinc-200">Album type</legend>
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
    <FormSelect v-model="sort" label="Sort by">
      <template #icon><SlidersHorizontal class="size-4" /></template>
      <option value="artist">Artist</option>
      <option value="title">Album title</option>
      <option value="newest">Newest release</option>
      <option value="oldest">Oldest release</option>
      <option value="recently-added">Recently added</option>
    </FormSelect>
  </aside>
</template>
