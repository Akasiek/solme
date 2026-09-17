<template>
  <div class="@container w-full">
    <div class="mx-auto flex w-full max-w-[clamp(36rem,60cqw,72rem)] flex-col gap-[clamp(0.75rem,1.5cqw,1.5rem)] pr-6">
      <slot />
    </div>
  </div>
</template>

<style scoped>
@reference "@/style/glob.css";

:deep(.lyrics-line) {
  @apply text-zinc-300;
  font-size: clamp(1.125rem, 0.75rem + 1cqw, 1.5rem);
}
</style>
