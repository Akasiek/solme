import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { ref } from "vue";

import type { AnnotatableLibraryItem, LibraryItemAnnotation, LibraryItemKind } from "@/types";
import { libraryItemKind, libraryItemName } from "@/utils/libraryItem";
import { useToastStore } from "./toast";

const annotationKey = (itemKind: LibraryItemKind, itemId: string) => `${itemKind}:${itemId}`;

const errorMessage = (cause: unknown) =>
  typeof cause === "string" ? cause : cause instanceof Error ? cause.message : "Unexpected error.";

const mutationCommand: Record<keyof LibraryItemAnnotation, string> = {
  favorite: "set_library_item_favorite",
  rating: "set_library_item_rating",
};

export const useLibraryAnnotationsStore = defineStore("library-annotations", () => {
  const annotations = ref<Record<string, LibraryItemAnnotation>>({});
  const mutationQueues = new Map<string, Promise<void>>();
  const toastStore = useToastStore();

  const seedAnnotation = (item: AnnotatableLibraryItem) => {
    const key = annotationKey(libraryItemKind(item), item.remoteId);
    if (!(key in annotations.value)) {
      annotations.value[key] = { favorite: item.favorite, rating: item.rating };
    }
  };

  const annotationFor = (item: AnnotatableLibraryItem) => {
    seedAnnotation(item);
    const key = annotationKey(libraryItemKind(item), item.remoteId);
    return annotations.value[key];
  };

  const ratingFor = (item: AnnotatableLibraryItem) => annotationFor(item).rating;

  const favoriteFor = (item: AnnotatableLibraryItem) => annotationFor(item).favorite;

  const updateAnnotation = <Field extends keyof LibraryItemAnnotation>(
    item: AnnotatableLibraryItem,
    field: Field,
    nextValue: LibraryItemAnnotation[Field],
  ) => {
    const itemKind = libraryItemKind(item);
    const itemId = item.remoteId;
    const itemName = libraryItemName(item);
    const key = annotationKey(itemKind, itemId);
    const queueKey = `${key}:${field}`;
    const annotation = annotationFor(item);
    const previousValue = annotation[field];

    annotations.value[key] = { ...annotation, [field]: nextValue };

    const mutation = (mutationQueues.get(queueKey) ?? Promise.resolve()).then(async () => {
      try {
        await invoke<LibraryItemAnnotation>(mutationCommand[field], {
          itemKind,
          itemId,
          [field]: nextValue,
        });
      } catch (cause) {
        const currentAnnotation = annotations.value[key];
        if (currentAnnotation[field] !== nextValue) return;

        annotations.value[key] = { ...currentAnnotation, [field]: previousValue };
        toastStore.show(`Could not update ${itemName}: ${errorMessage(cause)}`);
      }
    });

    mutationQueues.set(queueKey, mutation);
    void mutation.finally(() => {
      if (mutationQueues.get(queueKey) === mutation) mutationQueues.delete(queueKey);
    });
  };

  const setRating = (item: AnnotatableLibraryItem, value: number) => {
    const currentRating = ratingFor(item);
    updateAnnotation(item, "rating", currentRating === value ? null : value);
  };

  const toggleFavorite = (item: AnnotatableLibraryItem) => {
    updateAnnotation(item, "favorite", !favoriteFor(item));
  };

  return {
    seedAnnotation,
    ratingFor,
    favoriteFor,
    setRating,
    toggleFavorite,
  };
});
