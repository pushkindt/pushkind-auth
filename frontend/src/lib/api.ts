export interface ApiUser {
  sub: string;
  email: string;
  hub_id: number;
  name: string;
  roles: string[];
  exp: number;
}

export interface ApiHubSummary {
  id: number;
  name: string;
}

export interface ApiEditableProfile {
  name: string;
}

export interface ApiIam {
  user: ApiUser;
  current_hub: ApiHubSummary;
  editable_profile: ApiEditableProfile;
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

export interface ApiFieldError {
  field: string;
  message: string;
}

export interface ApiMutationSuccess {
  message: string;
  redirect_to: string | null;
}

export interface ApiMutationError {
  message: string;
  field_errors: ApiFieldError[];
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

  return (await response.json()) as T;
}

export function toFieldErrorMap(
  error: ApiMutationError,
): Record<string, string> {
  return Object.fromEntries(
    error.field_errors.map((fieldError) => [
      fieldError.field,
      fieldError.message,
    ]),
  );
}

export async function postForm(
  endpoint: string,
  body: URLSearchParams,
): Promise<ApiMutationSuccess> {
  const response = await fetch(endpoint, {
    method: "POST",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/x-www-form-urlencoded;charset=UTF-8",
    },
    body: body.toString(),
  });

  const payload = (await response.json()) as
    | ApiMutationSuccess
    | ApiMutationError;

  if (!response.ok) {
    throw payload as ApiMutationError;
  }

  return payload as ApiMutationSuccess;
}

export async function postEmpty(endpoint: string): Promise<ApiMutationSuccess> {
  const response = await fetch(endpoint, {
    method: "POST",
    headers: {
      Accept: "application/json",
    },
  });

  const payload = (await response.json()) as
    | ApiMutationSuccess
    | ApiMutationError;

  if (!response.ok) {
    throw payload as ApiMutationError;
  }

  return payload as ApiMutationSuccess;
}
