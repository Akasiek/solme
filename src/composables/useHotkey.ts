import { onKeyStroke } from "@vueuse/core";

type HotkeyMatcher = string | ((event: KeyboardEvent) => boolean);

export const useHotkey = (matcher: HotkeyMatcher, callback: () => void) => {
  let shouldPreventKeyUp = false;

  onKeyStroke(
    matcher,
    (event) => {
      if (isUserInputActive()) {
        return;
      }

      shouldPreventKeyUp = true;
      preventDefaultAndStopPropagation(event);
      callback();
    },
    {
      dedupe: true,
      passive: false,
    },
  );

  onKeyStroke(
    matcher,
    (event) => {
      if (!shouldPreventKeyUp) {
        return;
      }

      shouldPreventKeyUp = false;
      preventDefaultAndStopPropagation(event);
    },
    {
      eventName: "keyup",
      passive: false,
    },
  );
};

export const preventDefaultAndStopPropagation = (event: KeyboardEvent): void => {
  event.preventDefault();
  event.stopPropagation();
};

const isUserInputActive = (): boolean => {
  const activeElement = document.activeElement;

  return activeElement
    ? activeElement.tagName === "TEXTAREA" ||
        (activeElement.tagName === "INPUT" && activeElement.getAttribute("type") !== "range") ||
        (activeElement as HTMLElement).isContentEditable
    : false;
};
