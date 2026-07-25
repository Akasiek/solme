import { defineStore } from "pinia";
import { reactive } from "vue";

type ServerConnectionPhase = "idle" | "primary" | "secondary" | "failed";

interface ServerConnectionStatus {
  phase: ServerConnectionPhase;
  primaryUrl?: string;
  secondaryUrl?: string;
  error?: string;
}

export const useServerConnectionStore = defineStore("serverConnection", () => {
  const status = reactive<ServerConnectionStatus>({
    phase: "idle",
  });

  const connectPrimary = (primaryUrl?: string, secondaryUrl?: string) => {
    status.phase = "primary";
    status.primaryUrl = primaryUrl;
    status.secondaryUrl = secondaryUrl;
    status.error = undefined;
  };

  const connectSecondary = () => {
    status.phase = "secondary";
    status.error = undefined;
  };

  const failConnection = (error: string) => {
    status.phase = "failed";
    status.error = error;
  };

  const clear = () => {
    status.phase = "idle";
    status.primaryUrl = undefined;
    status.secondaryUrl = undefined;
    status.error = undefined;
  };

  return {
    status,
    connectPrimary,
    connectSecondary,
    failConnection,
    clear,
  };
});
