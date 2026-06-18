<script setup lang="ts">
defineProps<{
  hasLight?: boolean;
  label: string;
}>();
</script>

<template>
  <button class="cd-button" type="button" :aria-label="label">
    <span v-if="hasLight" class="status-light" />
    <span v-else class="control-symbol" aria-hidden="true">
      <slot />
    </span>
  </button>
</template>

<style scoped>
.cd-button {
  position: relative;
  isolation: isolate;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 3.75rem;
  width: 5.85rem;
  padding: 0;
  border: 0;
  border-radius: 3px;
  color: rgb(28 30 30);
  background-image: url("/assets/images/texture_bg_2.webp");
  background-repeat: repeat;
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
  inset: 0;
  pointer-events: none;
  background: linear-gradient(
    to bottom,
    rgba(255, 255, 255, 0.9) 0%,
    rgba(255, 255, 255, 0.8) 4%,
    rgba(255, 255, 255, 0.12) 6%,
    rgba(0, 0, 0, 0) 15%,
    rgba(0, 0, 0, 0) 100%
  );
}

.cd-button::after {
  content: "";
  position: absolute;
  z-index: 2;
  inset: 0;
  pointer-events: none;
  background: linear-gradient(
    to top,
    rgba(0, 0, 0, 0.8) 0%,
    rgba(0, 0, 0, 0.6) 2%,
    rgba(0, 0, 0, 0.2) 4%,
    rgba(0, 0, 0, 0) 8%,
    rgba(0, 0, 0, 0) 100%
  );
}

.cd-button:active {
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

.control-symbol {
  position: relative;
  z-index: 1;
  display: grid;
  place-items: center;
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
  position: relative;
  z-index: 1;
  display: block;
  width: 16px;
  height: 8px;
  border-radius: 1px;
  background: rgb(175 245 141);
  box-shadow:
    0 0 1px rgba(0, 0, 0, 0.7),
    inset 0 0 2px 2px rgba(60, 119, 18, 0.9);
}
</style>
