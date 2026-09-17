<script setup lang="ts">
import { Calendar, Check, Heart, Library, Play, Star, Tag } from "@lucide/vue";
import NumberInput from "@/components/NumberInput.vue";
import type { CatalogFilter } from "@/types";

defineProps<{ genres: string[] }>();

const filters = defineModel<CatalogFilter>({ required: true });

function selectMinimumRating(value: number) {
  filters.value.minimumRating = filters.value.minimumRating === value ? null : value;
  filters.value.unratedOnly = false;
}

function toggleUnrated() {
  filters.value.unratedOnly = !filters.value.unratedOnly;
  if (filters.value.unratedOnly) filters.value.minimumRating = null;
}

function toggleNeverPlayed() {
  filters.value.neverPlayed = !filters.value.neverPlayed;
  if (filters.value.neverPlayed) filters.value.minimumPlayCount = null;
}
</script>

<template>
  <fieldset class="space-y-2">
    <legend class="mb-2 font-serif text-sm font-bold text-zinc-300">
      <span class="flex items-center gap-1.5">
        <Library class="mt-0.5 size-4 shrink-0" aria-hidden="true" />Library status
      </span>
    </legend>
    <button
      type="button"
      class="flex h-8 w-full cursor-pointer items-center gap-2 rounded-md border px-2 font-sans text-sm transition-colors focus:ring-2 focus:ring-zinc-500 focus:outline-none"
      :class="
        filters.favoriteOnly
          ? 'border-accent/60 bg-accent/10 text-accent hover:bg-accent/20'
          : 'border-zinc-700 text-zinc-300 hover:border-zinc-500 hover:text-white'
      "
      :aria-pressed="filters.favoriteOnly"
      @click="filters.favoriteOnly = !filters.favoriteOnly"
    >
      <Heart class="size-3.5" :fill="filters.favoriteOnly ? 'currentColor' : 'none'" aria-hidden="true" />
      Favorites only
    </button>
    <div class="space-y-1.5">
      <span class="font-sans text-xs text-zinc-400">Minimum rating</span>
      <div
        class="flex h-8 items-center gap-0 rounded-md border border-zinc-700 px-1"
        aria-label="Minimum rating filter"
      >
        <button
          v-for="value in 5"
          :key="value"
          type="button"
          class="cursor-pointer rounded p-0.5 transition-colors hover:text-amber-300 focus:ring-2 focus:ring-zinc-500 focus:outline-none"
          :class="value <= (filters.minimumRating ?? 0) ? 'text-amber-400' : 'text-zinc-600'"
          :aria-label="
            filters.minimumRating === value
              ? `Clear ${value}-star minimum rating`
              : `Show items rated ${value} stars or higher`
          "
          :aria-pressed="filters.minimumRating === value"
          @click="selectMinimumRating(value)"
        >
          <Star
            class="size-4"
            :fill="value <= (filters.minimumRating ?? 0) ? 'currentColor' : 'none'"
            aria-hidden="true"
          />
        </button>
      </div>
    </div>
    <button
      type="button"
      class="flex h-8 w-full cursor-pointer items-center gap-2 rounded-md border px-2 font-sans text-sm transition-colors focus:ring-2 focus:ring-zinc-500 focus:outline-none"
      :class="
        filters.unratedOnly ? 'border-amber-400/60 bg-amber-400/10 text-amber-300' : 'border-zinc-700 text-zinc-300'
      "
      :aria-pressed="filters.unratedOnly"
      @click="toggleUnrated"
    >
      <Star class="size-3.5" aria-hidden="true" />
      Unrated only
    </button>
  </fieldset>

  <fieldset class="space-y-2">
    <legend class="font-serif text-sm font-bold text-zinc-300">
      <span class="flex items-center gap-1.5">
        <Calendar class="mt-0.5 size-4 shrink-0" aria-hidden="true" />Release year
      </span>
    </legend>
    <div class="grid grid-cols-2 gap-2">
      <NumberInput v-model="filters.fromYear" label="From" :min="1" placeholder="1960" />
      <NumberInput v-model="filters.toYear" label="To" :min="1" placeholder="2026" />
    </div>
  </fieldset>

  <fieldset v-if="genres.length" class="min-w-0">
    <legend class="mb-2 font-serif text-sm font-bold text-zinc-300">
      <span class="flex items-center gap-1.5"><Tag class="mt-0.5 size-4 shrink-0" aria-hidden="true" />Genre</span>
    </legend>
    <div class="max-h-44 min-w-0 space-y-1 overflow-y-auto pr-1">
      <label
        v-for="genre in genres"
        :key="genre"
        class="flex min-w-0 cursor-pointer items-center gap-2.5 rounded px-2 py-1.5 text-sm text-zinc-300 transition-colors hover:bg-zinc-800 hover:text-white"
      >
        <span class="relative size-4 shrink-0">
          <input
            v-model="filters.genres"
            type="checkbox"
            :value="genre"
            class="peer size-4 cursor-pointer appearance-none rounded border border-zinc-600 bg-zinc-950 transition-colors checked:border-accent checked:bg-accent focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
          />
          <Check
            aria-hidden="true"
            class="pointer-events-none absolute inset-0 size-4 stroke-3 text-white opacity-0 transition-opacity peer-checked:opacity-100"
          />
        </span>
        <span class="min-w-0 flex-1 truncate font-sans" :title="genre">{{ genre }}</span>
      </label>
    </div>
  </fieldset>

  <fieldset class="min-w-0 space-y-2">
    <legend class="font-serif text-sm font-bold text-zinc-300">
      <span class="flex items-center gap-1.5"><Play class="mt-0.5 size-4 shrink-0" aria-hidden="true" />Playback</span>
    </legend>
    <button
      type="button"
      class="flex h-8 w-full cursor-pointer items-center gap-2 rounded-md border px-2 font-sans text-sm transition-colors focus:ring-2 focus:ring-zinc-500 focus:outline-none"
      :class="filters.neverPlayed ? 'border-accent/60 bg-accent/10 text-accent' : 'border-zinc-700 text-zinc-300'"
      :aria-pressed="filters.neverPlayed"
      @click="toggleNeverPlayed"
    >
      Never played
    </button>
    <NumberInput
      v-model="filters.minimumPlayCount"
      label="Minimum play count"
      :min="1"
      :disabled="filters.neverPlayed"
      placeholder="1"
      @update:model-value="filters.neverPlayed = false"
    />
  </fieldset>
</template>
