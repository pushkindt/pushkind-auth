import { ModalFlashShell } from "@pushkind/frontend-shell/ModalFlashShell";
import type { ReactNode } from "react";

type AuthModalFlashShellProps = {
  children: ReactNode;
};

export function AuthModalFlashShell({ children }: AuthModalFlashShellProps) {
  return <ModalFlashShell navbar={null}>{children}</ModalFlashShell>;
}
