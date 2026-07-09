import { reactive, readonly } from "vue";

type ServerConnectionPhase = "idle" | "primary" | "secondary" | "failed";

interface ServerConnectionStatus {
  phase: ServerConnectionPhase;
  primaryUrl?: string;
  secondaryUrl?: string;
  error?: string;
}

const status = reactive<ServerConnectionStatus>({
  phase: "idle",
});

export function useServerConnectionStatus() {
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

  const clearConnectionStatus = () => {
    status.phase = "idle";
    status.primaryUrl = undefined;
    status.secondaryUrl = undefined;
    status.error = undefined;
  };

  return {
    serverConnectionStatus: readonly(status),
    connectPrimary,
    connectSecondary,
    failConnection,
    clearConnectionStatus,
  };
}
