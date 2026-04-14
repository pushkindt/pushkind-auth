import { useServiceShell } from "../../../../pushkind-common/frontend/src/useServiceShell";

import { fetchHubMenuItems, fetchShellData } from "./api";
import type { ShellData, UserMenuItem } from "./models";

export function useAuthShell(errorMessage: string) {
  return useServiceShell<ShellData, UserMenuItem>({
    errorMessage,
    menuLoadWarning:
      "Failed to load auth navigation menu. Falling back to local auth menu only.",
    fetchShellData,
    fetchHubMenuItems,
  });
}
