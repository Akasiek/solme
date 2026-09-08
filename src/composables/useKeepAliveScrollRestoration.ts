import { nextTick, onActivated, ref } from "vue";
import { onBeforeRouteLeave } from "vue-router";

export function useKeepAliveScrollRestoration(selector = "main") {
  const scrollTop = ref(0);

  onBeforeRouteLeave(() => {
    scrollTop.value = document.querySelector<HTMLElement>(selector)?.scrollTop ?? 0;
  });

  onActivated(async () => {
    await nextTick();
    window.requestAnimationFrame(() => {
      document.querySelector<HTMLElement>(selector)?.scrollTo({ top: scrollTop.value });
    });
  });
}
