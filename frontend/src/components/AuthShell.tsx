import { ModalFlashShell } from "@pushkind/frontend-shell/ModalFlashShell";
import type { ReactNode } from "react";

import { AuthNavbar } from "./AuthNavbar";
import type { NavigationItem, UserMenuItem } from "../lib/models";

type AuthShellProps = {
  navigation: NavigationItem[];
  currentUserEmail: string;
  homeUrl: string;
  localMenuItems: UserMenuItem[];
  fetchedMenuItems: UserMenuItem[];
  hubName: string;
  children: ReactNode;
};

export function AuthShell({
  navigation,
  currentUserEmail,
  homeUrl,
  localMenuItems,
  fetchedMenuItems,
  hubName,
  children,
}: AuthShellProps) {
  return (
    <ModalFlashShell
      navbar={
        <AuthNavbar
          navigation={navigation}
          currentUserEmail={currentUserEmail}
          homeUrl={homeUrl}
          localMenuItems={localMenuItems}
          fetchedMenuItems={fetchedMenuItems}
          hubName={hubName}
        />
      }
    >
      {children}
    </ModalFlashShell>
  );
}
