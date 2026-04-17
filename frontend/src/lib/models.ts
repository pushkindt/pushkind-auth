import type {
  FrontendShellCurrentUser,
  FrontendShellData,
  FrontendShellNavigationItem,
  FrontendShellUserMenuItem,
} from "@pushkind/frontend-shell/types";

export type NavigationItem = FrontendShellNavigationItem;
export type UserMenuItem = FrontendShellUserMenuItem;
export type CurrentUser = FrontendShellCurrentUser;

export type ShellData = FrontendShellData & {
  hubName: string;
};
