import type {
  FrontendShellCurrentUser,
  FrontendShellData,
  FrontendShellNavigationItem,
  FrontendShellUserMenuItem,
} from "@pushkind/frontend-shell/types";

export type NavigationItem = FrontendShellNavigationItem;
export type UserMenuItem = FrontendShellUserMenuItem;
// ts-prune-ignore-next
export type CurrentUser = FrontendShellCurrentUser;

export type ShellData = FrontendShellData & {
  hubName: string;
};
