import { onMounted, ref } from "vue";

export function useAsyncData<T>(loader: () => Promise<T>, initialValue: T) {
  const data = ref<T>(initialValue);
  const isLoading = ref(true);
  const hasLoaded = ref(false);
  const error = ref<string | null>(null);
  let latestRequestId = 0;

  async function reload() {
    const requestId = ++latestRequestId;

    isLoading.value = true;
    error.value = null;

    let outcome: { status: "success"; data: T } | { status: "error"; message: string };

    try {
      outcome = { status: "success", data: await loader() };
    } catch (cause) {
      outcome = {
        status: "error",
        message: cause instanceof Error ? cause.message : "Unexpected error.",
      };
    }

    if (requestId !== latestRequestId) {
      return;
    }

    if (outcome.status === "success") {
      data.value = outcome.data;
    } else {
      error.value = outcome.message;
    }

    isLoading.value = false;
    hasLoaded.value = true;
  }

  onMounted(() => {
    void reload();
  });

  return {
    data,
    isLoading,
    hasLoaded,
    error,
    reload,
  };
}
