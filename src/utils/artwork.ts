import { convertFileSrc } from "@tauri-apps/api/core";

export function artworkSource(path?: string): string | undefined {
  return path ? convertFileSrc(path) : undefined;
}
