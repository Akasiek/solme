import { useMagicKeys } from "@vueuse/core";

export const overrideKeyAction = (key: string, callback: () => void) => {
  let shouldPreventKeyUp = false;

  useMagicKeys({
    passive: false,
    onEventFired(e) {
      if (e.key !== key) {
        return;
      }

      if (e.type === "keyup" && shouldPreventKeyUp) {
        shouldPreventKeyUp = false;
        preventDefaultAndStopPropagation(e);
        return;
      }

      if (e.type === "keydown" && !e.repeat && !isUserInputActive()) {
        shouldPreventKeyUp = true;
        preventDefaultAndStopPropagation(e);
        callback();
      }
    },
  });
};

// Prevent the first key being sent to input
export const preventDefaultAndStopPropagation = (e: KeyboardEvent): void => {
  e.preventDefault();
  e.stopPropagation();
  e.stopImmediatePropagation();
};

// Check if the user is currently focused on an input, textarea
export const isUserInputActive = (): boolean => {
  const activeElement = document.activeElement;

  return activeElement
    ? activeElement.tagName === "TEXTAREA" ||
        (activeElement.tagName === "INPUT" && activeElement.getAttribute("type") !== "range") ||
        (activeElement as HTMLElement).isContentEditable
    : false;
};
