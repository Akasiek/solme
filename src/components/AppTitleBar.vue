<script setup lang="ts">
import { Minus, Square, X } from "@lucide/vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import logoSource from "@/assets/solme-logo-dark.svg";

const appWindow = getCurrentWindow();

function minimizeWindow() {
  void appWindow.minimize();
}

function toggleMaximizeWindow() {
  void appWindow.toggleMaximize();
}

function closeWindow() {
  void appWindow.close();
}

function startDraggingWindow(event: PointerEvent) {
  if (event.button !== 0) {
    return;
  }

  void appWindow.startDragging();
}
</script>

<template>
  <header
    data-tauri-drag-region
    class="flex h-8 shrink-0 items-center justify-between border-b border-zinc-800 bg-zinc-950 text-zinc-300 select-none"
    @dblclick="toggleMaximizeWindow"
    @pointerdown.self="startDraggingWindow"
  >
    <div
      data-tauri-drag-region
      class="flex min-w-0 flex-1 items-center gap-2 self-stretch px-3"
      @pointerdown="startDraggingWindow"
    >
      <img class="pointer-events-none size-4 shrink-0" :src="logoSource" alt="" aria-hidden="true" draggable="false" />
      <span class="truncate text-sm font-medium">Solm<span class="text-accent">ë</span></span>
    </div>
    <div class="flex h-full items-stretch">
      <button
        type="button"
        class="inline-flex w-11 items-center justify-center text-zinc-400 transition-colors hover:bg-zinc-900 hover:text-zinc-100"
        aria-label="Minimize window"
        @click="minimizeWindow"
      >
        <Minus class="size-4" aria-hidden="true" />
      </button>
      <button
        type="button"
        class="inline-flex w-11 items-center justify-center text-zinc-400 transition-colors hover:bg-zinc-900 hover:text-zinc-100"
        aria-label="Maximize window"
        @click="toggleMaximizeWindow"
      >
        <Square class="size-3.5" aria-hidden="true" />
      </button>
      <button
        type="button"
        class="inline-flex w-11 items-center justify-center text-zinc-400 transition-colors hover:bg-red-500 hover:text-white"
        aria-label="Close window"
        @click="closeWindow"
      >
        <X class="size-4" aria-hidden="true" />
      </button>
    </div>
  </header>
</template>
