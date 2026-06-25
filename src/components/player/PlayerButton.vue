<script setup lang="ts">
defineProps<{
  hasLight?: boolean;
  label: string;
}>();
</script>

<template>
  <button
    class="cd-button relative isolate flex h-15 w-24 items-center justify-center overflow-hidden rounded-sm bg-repeat p-0 text-zinc-900"
    type="button"
    :aria-label="label"
  >
    <span v-if="hasLight" class="status-light relative z-1 block h-2 w-4 rounded-xs bg-green-300" />
    <span v-else class="control-symbol relative z-1 grid place-items-center" aria-hidden="true">
      <slot />
    </span>
  </button>
</template>

<style scoped>
.cd-button {
  border: 0;
  background-image: url("/assets/images/texture_bg_2.webp");
  background-size: 200%;
  transition:
    transform 100ms,
    box-shadow 150ms,
    filter 150ms;
  box-shadow:
    0 16px 12px rgba(0, 0, 0, 0.3),
    0 0 4px 2px rgba(255, 255, 255, 0.4),
    0 0 2px 3px rgba(0, 0, 0, 1),
    inset 0 0 1px 2px rgba(0, 0, 0, 0.3);
}

.cd-button::before {
  content: "";
  position: absolute;
  z-index: 2;
  top: 0;
  inset-inline: 0;
  height: 3px;
  pointer-events: none;
  background: linear-gradient(
    to bottom,
    rgba(255, 255, 255, 1) 0%,
    rgba(255, 255, 255, 0.6) 70%,
    rgba(255, 255, 255, 0.4) 90%,
    rgba(0, 0, 0, 0) 100%
  );
}

.cd-button::after {
  content: "";
  position: absolute;
  z-index: 2;
  bottom: 0;
  inset-inline: 0;
  height: 3px;
  pointer-events: none;
  background: linear-gradient(
    to top,
    rgba(0, 0, 0, 0.6) 0%,
    rgba(0, 0, 0, 0.4) 70%,
    rgba(0, 0, 0, 0.2) 90%,
    rgba(0, 0, 0, 0) 100%
  );
}

.cd-button:active,
.cd-button:disabled {
  transform: translateY(2px) scale(0.995);
  filter: brightness(0.96);
  box-shadow:
    0 10px 12px rgba(0, 0, 0, 0.3),
    0 0 4px 2px rgba(255, 255, 255, 0.4),
    0 0 2px 3px rgba(0, 0, 0, 1),
    inset 0 2px 3px rgba(0, 0, 0, 0.38);
}

.cd-button:focus-visible {
  outline: 2px solid rgb(40 40 38);
  outline-offset: 4px;
}

.cd-button:disabled {
  cursor: default;
  filter: brightness(0.9);
}

.cd-button:disabled .control-symbol {
  transform: translateY(1px);
  opacity: 0.34;
  filter: drop-shadow(0 1px 0 rgba(255, 255, 255, 0.42)) drop-shadow(0 -1px 0 rgba(0, 0, 0, 0.25));
}

.control-symbol {
  opacity: 0.82;
  filter: drop-shadow(0 1px 0 rgba(255, 255, 255, 0.72)) drop-shadow(0 -1px 0 rgba(0, 0, 0, 0.52))
    drop-shadow(0 0 0.35px rgba(0, 0, 0, 0.9));
  transition:
    transform 100ms,
    opacity 100ms,
    filter 100ms;
}

.control-symbol :deep(svg) {
  display: block;
  width: auto;
  height: 1.5rem;
}

.cd-button:active .control-symbol {
  transform: translateY(1px);
  opacity: 0.74;
  filter: drop-shadow(0 0.5px 0 rgba(255, 255, 255, 0.5))
    drop-shadow(0 -0.5px 0 rgba(0, 0, 0, 0.42));
}

.cd-button:active .status-light {
  filter: brightness(0.9);
}

.status-light {
  box-shadow:
    0 0 1px rgba(0, 0, 0, 0.7),
    inset 0 0 2px 2px rgba(60, 119, 18, 0.9);
}
</style>
