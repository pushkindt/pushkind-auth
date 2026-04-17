import { useEffect, useRef, useState } from "react";
import type { FormEvent, KeyboardEvent } from "react";

import { AuthShell } from "../components/AuthShell";
import { AuthShellFatalState } from "../components/AuthShellFatalState";
import {
  DropdownMultiSelect,
  type DropdownMultiSelectOption,
} from "@pushkind/frontend-shell/DropdownMultiSelect";
import {
  fetchHubMenuItems,
  fetchJson,
  fetchShellData,
  isApiMutationError,
  isRedirectResponseError,
  postEmpty,
  postForm,
  postJson,
  toFieldErrorMap,
  type ApiAdminDashboard,
  type ApiMutationError,
  type ApiUserListItem,
  type DashboardUser,
} from "../lib/api";
import type { ShellData, UserMenuItem } from "../lib/models";
import { useServiceShell } from "@pushkind/frontend-shell/useServiceShell";

interface RoleOption {
  id: number;
  name: string;
}

interface AdminEditableUser {
  id: number;
  email: string;
  name: string;
  roles: number[];
}

interface AdminUserModalBootstrap {
  user: AdminEditableUser | null;
  roles: RoleOption[];
}

interface AdminUserFormState {
  id: number;
  email: string;
  name: string;
  password: string;
  roles: string[];
}

type AdminPageState =
  | { status: "loading" }
  | { status: "ready"; admin: ApiAdminDashboard; users: DashboardUser[] }
  | { status: "error"; message: string };

function toMutationError(
  error: unknown,
  fallbackMessage: string,
): ApiMutationError {
  if (isApiMutationError(error)) {
    return error;
  }

  return {
    message: fallbackMessage,
    field_errors: [],
  };
}

function mapUsers(users: ApiUserListItem[]): DashboardUser[] {
  return users.map((user) => ({
    id: Number(user.sub),
    email: user.email,
    name: user.name,
    roles: user.roles,
  }));
}

export function MainAdminPage() {
  const shellState = useServiceShell<ShellData, UserMenuItem>({
    errorMessage: "Не удалось загрузить оболочку Auth.",
    menuLoadWarning:
      "Failed to load auth navigation menu. Falling back to local Auth menu only.",
    fetchShellData,
    fetchHubMenuItems,
  });
  const [menuState, setMenuState] = useState<UserMenuItem[]>([]);
  const [pageState, setPageState] = useState<AdminPageState>({
    status: "loading",
  });
  const [filterValue, setFilterValue] = useState("");
  const [roleName, setRoleName] = useState("");
  const [hubName, setHubName] = useState("");
  const [menuName, setMenuName] = useState("");
  const [menuUrl, setMenuUrl] = useState("");
  const [roleErrors, setRoleErrors] = useState<Record<string, string>>({});
  const [hubErrors, setHubErrors] = useState<Record<string, string>>({});
  const [menuErrors, setMenuErrors] = useState<Record<string, string>>({});
  const [isSubmittingRole, setIsSubmittingRole] = useState(false);
  const [isSubmittingHub, setIsSubmittingHub] = useState(false);
  const [isSubmittingMenu, setIsSubmittingMenu] = useState(false);
  const [isLoadingModal, setIsLoadingModal] = useState(false);
  const [modalError, setModalError] = useState<string | null>(null);
  const [modalData, setModalData] = useState<AdminUserModalBootstrap | null>(
    null,
  );
  const [modalForm, setModalForm] = useState<AdminUserFormState | null>(null);
  const [modalFieldErrors, setModalFieldErrors] = useState<
    Record<string, string>
  >({});
  const [isSavingModal, setIsSavingModal] = useState(false);
  const [isDeletingModal, setIsDeletingModal] = useState(false);
  const modalRef = useRef<HTMLDivElement | null>(null);
  const requestIdRef = useRef(0);

  useEffect(() => {
    if (shellState.status === "ready") {
      setMenuState(shellState.authMenuItems);
    }
  }, [shellState]);

  useEffect(() => {
    if (shellState.status !== "ready") {
      return;
    }

    let active = true;

    void Promise.all([
      fetchJson<ApiAdminDashboard>("/api/v1/admin/dashboard"),
      fetchJson<ApiUserListItem[]>("/api/v1/users"),
    ])
      .then(([admin, users]) => {
        if (!active) {
          return;
        }

        setPageState({
          status: "ready",
          admin,
          users: mapUsers(users),
        });
      })
      .catch((error) => {
        if (!active) {
          return;
        }

        setPageState({
          status: "error",
          message:
            error instanceof Error
              ? error.message
              : "Не удалось загрузить административные данные Auth.",
        });
      });

    return () => {
      active = false;
    };
  }, [shellState]);

  if (shellState.status === "loading" || pageState.status === "loading") {
    return null;
  }

  if (shellState.status === "error") {
    return <AuthShellFatalState message={shellState.message} />;
  }

  if (pageState.status === "error") {
    return <AuthShellFatalState message={pageState.message} />;
  }

  const shell = shellState.shell;

  const normalizedFilter = filterValue.trim().toLowerCase();
  const filteredUsers = pageState.users.filter((user) => {
    if (!normalizedFilter) {
      return true;
    }

    return [user.name, user.email, user.roles.join(" ")]
      .join(" ")
      .toLowerCase()
      .includes(normalizedFilter);
  });

  async function refreshAdminPage(): Promise<void> {
    const [nextMenu, nextAdmin, nextUsers] = await Promise.all([
      fetchHubMenuItems(shell.homeUrl, shell.currentUser.hubId),
      fetchJson<ApiAdminDashboard>("/api/v1/admin/dashboard"),
      fetchJson<ApiUserListItem[]>("/api/v1/users"),
    ]);

    setMenuState(nextMenu);
    setPageState({
      status: "ready",
      admin: nextAdmin,
      users: mapUsers(nextUsers),
    });
  }

  async function handleCreateMutation(
    endpoint: string,
    body: URLSearchParams,
    setErrors: (errors: Record<string, string>) => void,
    onSuccess?: () => void,
  ): Promise<boolean> {
    try {
      const result = await postForm(endpoint, body);
      setErrors({});
      await refreshAdminPage();
      onSuccess?.();
      window.showFlashMessage?.(result.message, "success");
      return true;
    } catch (error) {
      if (isRedirectResponseError(error)) {
        return false;
      }

      const mutationError = toMutationError(
        error,
        "Не удалось сохранить изменения.",
      );
      setErrors(toFieldErrorMap(mutationError));
      window.showFlashMessage?.(mutationError.message, "danger");
      return false;
    }
  }

  async function handleDeleteMutation(endpoint: string): Promise<boolean> {
    if (!window.confirm("Удалить?")) {
      return false;
    }

    try {
      const result = await postEmpty(endpoint);
      await refreshAdminPage();
      window.showFlashMessage?.(result.message, "success");
      return true;
    } catch (error) {
      if (isRedirectResponseError(error)) {
        return false;
      }

      const mutationError = toMutationError(
        error,
        "Не удалось удалить запись.",
      );
      window.showFlashMessage?.(mutationError.message, "danger");
      return false;
    }
  }

  function closeModal() {
    if (modalRef.current) {
      window.bootstrap?.Modal.getOrCreateInstance(modalRef.current).hide();
    }
    setModalData(null);
    setModalForm(null);
    setModalFieldErrors({});
    setModalError(null);
  }

  async function openUserModal(userId: number) {
    const requestId = requestIdRef.current + 1;
    requestIdRef.current = requestId;
    setIsLoadingModal(true);
    setModalError(null);
    setModalData(null);
    setModalForm(null);
    setModalFieldErrors({});

    if (modalRef.current) {
      window.bootstrap?.Modal.getOrCreateInstance(modalRef.current).show();
    }

    try {
      const data = await postJson<AdminUserModalBootstrap>(
        `/admin/user/modal/${userId}`,
      );

      if (requestIdRef.current !== requestId) {
        return;
      }

      setModalData(data);
      if (data.user) {
        setModalForm({
          id: data.user.id,
          email: data.user.email,
          name: data.user.name,
          password: "",
          roles: data.user.roles.map(String),
        });
      }
    } catch (error) {
      if (requestIdRef.current !== requestId) {
        return;
      }

      if (isRedirectResponseError(error)) {
        return;
      }

      console.error(error);
      setModalError("Не удалось загрузить данные пользователя.");
    } finally {
      if (requestIdRef.current === requestId) {
        setIsLoadingModal(false);
      }
    }
  }

  async function handleRoleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsSubmittingRole(true);

    const body = new URLSearchParams();
    body.set("name", roleName);

    const didSucceed = await handleCreateMutation(
      "/admin/role/add",
      body,
      setRoleErrors,
      () => setRoleName(""),
    );

    if (didSucceed) {
      setRoleName("");
    }

    setIsSubmittingRole(false);
  }

  async function handleHubSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsSubmittingHub(true);

    const body = new URLSearchParams();
    body.set("name", hubName);

    const didSucceed = await handleCreateMutation(
      "/admin/hub/add",
      body,
      setHubErrors,
      () => setHubName(""),
    );

    if (didSucceed) {
      setHubName("");
    }

    setIsSubmittingHub(false);
  }

  async function handleMenuSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsSubmittingMenu(true);

    const body = new URLSearchParams();
    body.set("name", menuName);
    body.set("url", menuUrl);

    const didSucceed = await handleCreateMutation(
      "/admin/menu/add",
      body,
      setMenuErrors,
      () => {
        setMenuName("");
        setMenuUrl("");
      },
    );

    if (didSucceed) {
      setMenuName("");
      setMenuUrl("");
    }

    setIsSubmittingMenu(false);
  }

  async function handleModalSave(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!modalForm) {
      return;
    }

    setIsSavingModal(true);
    const body = new URLSearchParams();
    body.set("id", String(modalForm.id));
    body.set("name", modalForm.name);
    body.set("password", modalForm.password);
    modalForm.roles.forEach((role) => body.append("roles", role));

    const didSucceed = await handleCreateMutation(
      `/admin/user/update/${modalForm.id}`,
      body,
      setModalFieldErrors,
      closeModal,
    );

    if (didSucceed) {
      closeModal();
    }

    setIsSavingModal(false);
  }

  async function handleModalDelete() {
    if (!modalForm) {
      return;
    }

    setIsDeletingModal(true);
    const didSucceed = await handleDeleteMutation(
      `/admin/user/delete/${modalForm.id}`,
    );

    if (didSucceed) {
      closeModal();
    }

    setIsDeletingModal(false);
  }

  function handleRowKeyDown(
    event: KeyboardEvent<HTMLDivElement>,
    userId: number,
  ) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      void openUserModal(userId);
    }
  }

  function updateModalRoles(values: string[]) {
    setModalFieldErrors((errors) => ({ ...errors, roles: "" }));
    setModalForm((current) =>
      current
        ? {
            ...current,
            roles: values,
          }
        : current,
    );
  }

  return (
    <AuthShell
      navigation={shell.navigation}
      currentUserEmail={shell.currentUser.email}
      homeUrl={shell.homeUrl}
      localMenuItems={shell.localMenuItems}
      fetchedMenuItems={menuState}
      hubName={shell.hubName}
    >
      <div className="container my-2">
        <div className="row">
          <div className="col-md">
            <h5>Роли</h5>
            <form onSubmit={(event) => void handleRoleSubmit(event)}>
              <div className="row">
                <div className="col">
                  <input
                    className={
                      roleErrors.name
                        ? "form-control my-1 is-invalid"
                        : "form-control my-1"
                    }
                    type="text"
                    name="name"
                    placeholder="Название"
                    required
                    value={roleName}
                    onChange={(event) => {
                      setRoleName(event.target.value);
                      setRoleErrors((errors) => ({ ...errors, name: "" }));
                    }}
                  />
                  {roleErrors.name ? (
                    <div className="invalid-feedback d-block">
                      {roleErrors.name}
                    </div>
                  ) : null}
                </div>
                <div className="col-auto">
                  <button
                    className="btn btn-primary my-1"
                    type="submit"
                    disabled={isSubmittingRole}
                  >
                    <i className="bi bi-plus"></i>
                  </button>
                </div>
              </div>
            </form>
            {pageState.admin.roles.map((role) =>
              role.can_delete ? (
                <button
                  key={role.id}
                  type="button"
                  className="btn btn-sm btn-outline-secondary mt-1 me-1"
                  onClick={() =>
                    void handleDeleteMutation(`/admin/role/delete/${role.id}`)
                  }
                >
                  {role.name}
                  <span className="badge rounded-pill bg-danger">
                    <i className="bi bi-x"></i>
                  </span>
                </button>
              ) : (
                <button
                  key={role.id}
                  type="button"
                  className="btn btn-sm btn-outline-secondary mt-1 me-1"
                >
                  {role.name}
                </button>
              ),
            )}
          </div>

          <div className="col-md">
            <h5>Хабы</h5>
            <form onSubmit={(event) => void handleHubSubmit(event)}>
              <div className="row">
                <div className="col">
                  <input
                    className={
                      hubErrors.name
                        ? "form-control my-1 is-invalid"
                        : "form-control my-1"
                    }
                    type="text"
                    name="name"
                    placeholder="Название"
                    required
                    value={hubName}
                    onChange={(event) => {
                      setHubName(event.target.value);
                      setHubErrors((errors) => ({ ...errors, name: "" }));
                    }}
                  />
                  {hubErrors.name ? (
                    <div className="invalid-feedback d-block">
                      {hubErrors.name}
                    </div>
                  ) : null}
                </div>
                <div className="col-auto">
                  <button
                    className="btn btn-primary my-1"
                    type="submit"
                    disabled={isSubmittingHub}
                  >
                    <i className="bi bi-plus"></i>
                  </button>
                </div>
              </div>
            </form>
            {pageState.admin.hubs.map((hub) =>
              hub.can_delete ? (
                <button
                  key={hub.id}
                  type="button"
                  className="btn btn-sm btn-outline-secondary mt-1 me-1"
                  onClick={() =>
                    void handleDeleteMutation(`/admin/hub/delete/${hub.id}`)
                  }
                >
                  {hub.name}
                  <span className="badge rounded-pill bg-danger">
                    <i className="bi bi-x"></i>
                  </span>
                </button>
              ) : (
                <button
                  key={hub.id}
                  type="button"
                  className="btn btn-sm btn-outline-secondary mt-1 me-1"
                >
                  {hub.name}
                </button>
              ),
            )}
          </div>

          <div className="col-md">
            <h5>Меню</h5>
            <form onSubmit={(event) => void handleMenuSubmit(event)}>
              <div className="row">
                <div className="col">
                  <input
                    className={
                      menuErrors.name
                        ? "form-control my-1 is-invalid"
                        : "form-control my-1"
                    }
                    type="text"
                    name="name"
                    placeholder="Название"
                    required
                    value={menuName}
                    onChange={(event) => {
                      setMenuName(event.target.value);
                      setMenuErrors((errors) => ({ ...errors, name: "" }));
                    }}
                  />
                  {menuErrors.name ? (
                    <div className="invalid-feedback d-block">
                      {menuErrors.name}
                    </div>
                  ) : null}
                </div>
                <div className="col">
                  <input
                    className={
                      menuErrors.url
                        ? "form-control my-1 is-invalid"
                        : "form-control my-1"
                    }
                    type="text"
                    name="url"
                    placeholder="URL"
                    required
                    value={menuUrl}
                    onChange={(event) => {
                      setMenuUrl(event.target.value);
                      setMenuErrors((errors) => ({ ...errors, url: "" }));
                    }}
                  />
                  {menuErrors.url ? (
                    <div className="invalid-feedback d-block">
                      {menuErrors.url}
                    </div>
                  ) : null}
                </div>
                <div className="col-auto">
                  <button
                    className="btn btn-primary my-1"
                    type="submit"
                    disabled={isSubmittingMenu}
                  >
                    <i className="bi bi-plus"></i>
                  </button>
                </div>
              </div>
            </form>
            {pageState.admin.admin_menu.map((menuItem) => (
              <button
                key={menuItem.id}
                type="button"
                className="btn btn-sm btn-outline-secondary mt-1 me-1"
                onClick={() =>
                  void handleDeleteMutation(`/admin/menu/delete/${menuItem.id}`)
                }
              >
                {menuItem.name}
                <span className="badge rounded-pill bg-danger">
                  <i className="bi bi-x"></i>
                </span>
              </button>
            ))}
          </div>
        </div>
      </div>

      {pageState.users.length > 0 ? (
        <>
          <div className="container mb-1">
            <div className="row">
              <div className="col">
                <input
                  type="text"
                  className="form-control"
                  placeholder="Фильтр"
                  id="filter"
                  value={filterValue}
                  onChange={(event) => setFilterValue(event.target.value)}
                />
              </div>
            </div>
          </div>
          <div className="container border bg-white" id="items">
            <div className="row mb-3 fw-bold">
              <div className="col overflow-hidden">Имя</div>
              <div className="col overflow-hidden">Email</div>
              <div className="col overflow-hidden">Роли</div>
            </div>
            {filteredUsers.map((user) => (
              <div
                key={user.id}
                className="row mb-3 border-bottom selectable"
                role="button"
                tabIndex={0}
                onClick={() => void openUserModal(user.id)}
                onKeyDown={(event) => handleRowKeyDown(event, user.id)}
              >
                <div className="col overflow-hidden">{user.name}</div>
                <div className="col overflow-hidden">{user.email}</div>
                <div className="col overflow-hidden">
                  {user.roles.map((role) => (
                    <span
                      key={`${user.id}-${role}`}
                      className="badge rounded-pill text-bg-light"
                    >
                      {role}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>

          <div
            className="modal fade"
            id="userModal"
            tabIndex={-1}
            aria-labelledby="userModalLabel"
            aria-hidden="true"
            ref={modalRef}
          >
            <div className="modal-dialog modal-lg">
              <div className="modal-content">
                <div className="modal-header">
                  <h1 className="modal-title fs-5" id="userModalLabel">
                    Редактировать пользователя
                  </h1>
                  <button
                    type="button"
                    className="btn-close"
                    data-bs-dismiss="modal"
                    aria-label="Close"
                  ></button>
                </div>
                {isLoadingModal ? (
                  <div className="modal-body">
                    <div className="text-center py-3">
                      <div
                        className="spinner-border"
                        role="status"
                        aria-hidden="true"
                      ></div>
                    </div>
                  </div>
                ) : modalError ? (
                  <div className="modal-body">
                    <div className="alert alert-danger mb-0">{modalError}</div>
                  </div>
                ) : !modalData?.user || !modalForm ? (
                  <div className="modal-body">
                    <div className="alert alert-warning mb-0">
                      Пользователь не найден.
                    </div>
                  </div>
                ) : (
                  <>
                    <div className="modal-body" key={modalForm.id}>
                      <form onSubmit={(event) => void handleModalSave(event)}>
                        <input
                          type="hidden"
                          value={modalForm.id}
                          name="id"
                          required
                        />
                        <div className="row mb-3">
                          <label
                            htmlFor="modalUserEmail"
                            className="col-md-2 col-form-label"
                          >
                            Электронный адрес
                          </label>
                          <div className="col-md-10">
                            <input
                              type="email"
                              readOnly
                              className="form-control-plaintext"
                              id="modalUserEmail"
                              value={modalForm.email}
                              placeholder="Электронный адрес"
                              required
                            />
                          </div>
                        </div>
                        <div className="row mb-3">
                          <label
                            htmlFor="modalUserName"
                            className="col-md-2 col-form-label"
                          >
                            Имя
                          </label>
                          <div className="col-md-10">
                            <input
                              name="name"
                              type="text"
                              className={
                                modalFieldErrors.name
                                  ? "form-control is-invalid"
                                  : "form-control"
                              }
                              id="modalUserName"
                              value={modalForm.name}
                              placeholder="Имя"
                              required
                              onChange={(event) => {
                                setModalForm((current) =>
                                  current
                                    ? { ...current, name: event.target.value }
                                    : current,
                                );
                                setModalFieldErrors((errors) => ({
                                  ...errors,
                                  name: "",
                                }));
                              }}
                            />
                            {modalFieldErrors.name ? (
                              <div className="invalid-feedback">
                                {modalFieldErrors.name}
                              </div>
                            ) : null}
                          </div>
                        </div>
                        <div className="mb-3 row">
                          <label
                            htmlFor="modalUserPassword"
                            className="col-sm-2 col-form-label"
                          >
                            Пароль
                          </label>
                          <div className="col-sm-10">
                            <input
                              type="password"
                              className={
                                modalFieldErrors.password
                                  ? "form-control is-invalid"
                                  : "form-control"
                              }
                              id="modalUserPassword"
                              name="password"
                              placeholder="*****"
                              value={modalForm.password}
                              onChange={(event) => {
                                setModalForm((current) =>
                                  current
                                    ? {
                                        ...current,
                                        password: event.target.value,
                                      }
                                    : current,
                                );
                                setModalFieldErrors((errors) => ({
                                  ...errors,
                                  password: "",
                                }));
                              }}
                            />
                            {modalFieldErrors.password ? (
                              <div className="invalid-feedback">
                                {modalFieldErrors.password}
                              </div>
                            ) : null}
                          </div>
                        </div>
                        <div className="row mb-3">
                          <label
                            htmlFor="user-assign-form-role-id"
                            className="col-md-2 col-form-label"
                          >
                            Роли
                          </label>
                          <div className="col-md-10">
                            <DropdownMultiSelect
                              id="user-assign-form-role-id"
                              options={modalData.roles.map(
                                (role): DropdownMultiSelectOption => ({
                                  value: String(role.id),
                                  label: role.name,
                                }),
                              )}
                              selectedValues={modalForm.roles}
                              onChange={updateModalRoles}
                              className="my-1"
                              menuHeightClassName="auth-dropdown-multiselect-options-md"
                              searchPlaceholder="Поиск ролей"
                              clearable
                              clearLabel="Очистить выбранные роли"
                            />
                            {modalFieldErrors.roles ? (
                              <div className="invalid-feedback d-block">
                                {modalFieldErrors.roles}
                              </div>
                            ) : null}
                          </div>
                        </div>
                        <div className="row mb-3">
                          <div className="col">
                            <button
                              className="btn btn-primary"
                              type="submit"
                              disabled={isSavingModal}
                            >
                              Сохранить
                            </button>
                          </div>
                        </div>
                      </form>
                    </div>

                    <div className="modal-footer">
                      <button
                        className="btn btn-danger"
                        type="button"
                        disabled={isDeletingModal}
                        onClick={() => void handleModalDelete()}
                      >
                        Удалить
                      </button>
                    </div>
                  </>
                )}
              </div>
            </div>
          </div>
        </>
      ) : null}
    </AuthShell>
  );
}
