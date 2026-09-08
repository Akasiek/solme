import { computed, ref, watch, type Ref } from "vue";

export function usePagination(total: Ref<number>, pageSize = 24, itemLabel = "items", page = ref(1)) {
  const pageCount = computed(() => Math.max(1, Math.ceil(total.value / pageSize)));
  const resultRange = computed(() => {
    if (total.value === 0) return `0 ${itemLabel}`;
    const start = (page.value - 1) * pageSize + 1;
    return `${start}–${Math.min(page.value * pageSize, total.value)} of ${total.value} ${itemLabel}`;
  });

  watch(pageCount, (count) => {
    page.value = Math.min(page.value, count);
  });

  function changePage(nextPage: number) {
    page.value = Math.min(pageCount.value, Math.max(1, nextPage));
    document.querySelector("main")?.scrollTo({ top: 0, behavior: "smooth" });
  }

  return { page, pageCount, pageSize, resultRange, changePage };
}
