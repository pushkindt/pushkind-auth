import { ModalFlashShell } from "../../../../pushkind-common/frontend/src/ModalFlashShell";
import type { ReactNode } from "react";

type AuthModalFlashShellProps = {
  children: ReactNode;
};

export function AuthModalFlashShell({ children }: AuthModalFlashShellProps) {
  return <ModalFlashShell navbar={null}>{children}</ModalFlashShell>;
}
