import {
  fetchHubMenuItems as fetchSharedHubMenuItems,
  fetchJson as fetchSharedJson,
  isJsonResponse,
  ensureResponseIsNotAuthRedirect,
  parseCurrentUser,
  parseMenuItems,
  parseNavigationItems,
  readJsonResponse as readSharedJsonResponse,
} from "@pushkind/frontend-shell/shellApi";
// ts-prune-ignore-next
export type { ApiFieldError, ApiMutationSuccess } from "@pushkind/frontend-shell/mutations";
export type { ApiMutationError } from "@pushkind/frontend-shell/mutations";
export {
  isApiMutationError,
  postEmpty,
  postForm,
  toFieldErrorMap,
} from "@pushkind/frontend-shell/mutations";

import { redirectTo } from "./redirect";
import type { ShellData, UserMenuItem } from "./models";

export interface ApiUser {
  sub: string;
  email: string;
  hub_id: number;
  name: string;
  roles: string[];
  exp: number;
}

export interface ApiShellPayload {
  current_user: ApiUser;
  home_url: string;
  navigation: ApiMenuItem[];
  local_menu_items: ApiMenuItem[];
  hub_name: string;
}

export interface ApiMenuItem {
  name: string;
  url: string;
}

export interface ApiAdminRole {
  id: number;
  name: string;
  can_delete: boolean;
}

export interface ApiAdminHub {
  id: number;
  name: string;
  can_delete: boolean;
}

export interface ApiAdminMenuItem {
  id: number;
  name: string;
}

export interface ApiAdminDashboard {
  roles: ApiAdminRole[];
  hubs: ApiAdminHub[];
  admin_menu: ApiAdminMenuItem[];
}

export interface ApiUserListItem {
  sub: string;
  email: string;
  hub_id: number;
  name: string;
  roles: string[];
  exp: number;
}

export interface DashboardUser {
  id: number;
  email: string;
  name: string;
  roles: string[];
}

export class RedirectResponseError extends Error {
  redirectTo: string;

  constructor(redirectTo: string) {
    super(`Expected JSON response but received a redirect to ${redirectTo}`);
    this.name = "RedirectResponseError";
    this.redirectTo = redirectTo;
  }
}

export function isRedirectResponseError(
  error: unknown,
): error is RedirectResponseError {
  return error instanceof RedirectResponseError;
}

function handleUnexpectedResponse(response: Response, endpoint: string): never {
  if (response.redirected && response.url) {
    redirectTo(response.url);
    throw new RedirectResponseError(response.url);
  }

  throw new Error(
    `Expected JSON response from ${endpoint} with status ${response.status}`,
  );
}

async function readJsonResponse<T>(
  response: Response,
  endpoint: string,
): Promise<T> {
  if (!isJsonResponse(response)) {
    handleUnexpectedResponse(response, endpoint);
  }

  return await readSharedJsonResponse<T>(response, endpoint);
}

export async function fetchJson<T>(endpoint: string): Promise<T> {
  const response = await fetch(endpoint, {
    headers: {
      Accept: "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(
      `Request failed for ${endpoint} with status ${response.status}`,
    );
  }

  return readJsonResponse<T>(response, endpoint);
}

export async function fetchShellData(): Promise<ShellData> {
  const payload = (await fetchSharedJson("/api/v1/iam", {
    unauthorizedMessage: "Сессия истекла.",
  })) as ApiShellPayload;

  return {
    currentUser: parseCurrentUser(payload.current_user),
    homeUrl: payload.home_url,
    navigation: parseNavigationItems(payload.navigation),
    localMenuItems: parseMenuItems(payload.local_menu_items),
    hubName: payload.hub_name,
  };
}

export async function fetchHubMenuItems(
  _homeUrl: string,
  hubId: number,
): Promise<UserMenuItem[]> {
  return fetchSharedHubMenuItems<UserMenuItem>(
    `/api/v1/hubs/${hubId}/menu-items`,
    "Сессия истекла.",
  );
}

export async function postJson<T>(endpoint: string): Promise<T> {
  const response = await fetch(endpoint, {
    method: "POST",
    headers: {
      Accept: "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(
      `Request failed for ${endpoint} with status ${response.status}`,
    );
  }

  ensureResponseIsNotAuthRedirect(response);

  return readSharedJsonResponse<T>(response, endpoint);
}
