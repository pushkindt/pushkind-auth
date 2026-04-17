import { ServiceNavbar } from "@pushkind/frontend-shell/ServiceNavbar";

import type { NavigationItem, UserMenuItem } from "../lib/models";

type AuthNavbarProps = {
  navigation: NavigationItem[];
  currentUserEmail: string;
  homeUrl: string;
  localMenuItems: UserMenuItem[];
  fetchedMenuItems: UserMenuItem[];
  hubName: string;
};

export function AuthNavbar({
  navigation,
  currentUserEmail,
  homeUrl,
  localMenuItems,
  fetchedMenuItems,
  hubName,
}: AuthNavbarProps) {
  return (
    <ServiceNavbar
      brand={hubName}
      brandHref={homeUrl}
      collapseId="auth-foundation-navbar"
      navigation={[
        ...navigation,
        ...fetchedMenuItems.map((item) => ({ name: item.name, url: item.url })),
      ]}
      currentUserEmail={currentUserEmail}
      homeUrl={homeUrl}
      localMenuItems={localMenuItems}
      fetchedMenuItems={fetchedMenuItems}
      logoutAction="/auth/logout"
      isNavigationItemActive={(item, pathname) => pathname === item.url}
    />
  );
}
