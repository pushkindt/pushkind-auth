import type {
  FrontendShellCurrentUser,
  FrontendShellData,
  FrontendShellNavigationItem,
  FrontendShellUserMenuItem,
} from "../../../../pushkind-common/frontend/src/types";

export type NavigationItem = FrontendShellNavigationItem;
export type UserMenuItem = FrontendShellUserMenuItem;
export type CurrentUser = FrontendShellCurrentUser;

export type ShellData = FrontendShellData & {
  hubName: string;
};
