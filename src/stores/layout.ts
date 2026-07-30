import { defineStore } from "pinia";
import { ref } from "vue";

export const useLayoutStore = defineStore("layout", () => {
  const isLeftAsideCollapsed = ref(false);
  const isRightAsideCollapsed = ref(false);
  const isBigArtworkShown = ref(false);
  const isSearchModalOpen = ref(false);

  const toggleLeftAside = () => {
    isLeftAsideCollapsed.value = !isLeftAsideCollapsed.value;
  };

  const toggleRightAside = () => {
    isRightAsideCollapsed.value = !isRightAsideCollapsed.value;
  };

  const toggleBigArtwork = () => {
    isBigArtworkShown.value = !isBigArtworkShown.value;
  };

  const openSearchModal = () => {
    isSearchModalOpen.value = true;
  };

  const closeSearchModal = () => {
    isSearchModalOpen.value = false;
  };

  return {
    isLeftAsideCollapsed,
    isRightAsideCollapsed,
    isBigArtworkShown,
    isSearchModalOpen,
    toggleLeftAside,
    toggleRightAside,
    toggleBigArtwork,
    openSearchModal,
    closeSearchModal,
  };
});
