import { defineStore } from "pinia";
import { ref } from "vue";

export const useLayoutStore = defineStore("layout", () => {
  const isRightAsideCollapsed = ref(false);

  const toggleRightAside = () => {
    isRightAsideCollapsed.value = !isRightAsideCollapsed.value;
  };

  return {
    isRightAsideCollapsed,
    toggleRightAside,
  };
});
