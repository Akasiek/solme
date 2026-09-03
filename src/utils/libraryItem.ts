import type { AnnotatableLibraryItem, LibraryItemKind } from "@/types";

export const libraryItemKind = (item: AnnotatableLibraryItem): LibraryItemKind => {
  if ("title" in item) return "song";
  if ("songCount" in item) return "album";
  return "artist";
};

export const libraryItemName = (item: AnnotatableLibraryItem) => ("title" in item ? item.title : item.name);
