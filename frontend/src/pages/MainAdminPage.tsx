import { useRef, useState } from "react";
import type { FormEvent, KeyboardEvent } from "react";

import { AppShell } from "../components/AppShell";
import { Navigation, type NavigationMenuItem } from "../components/Navigation";

interface AdminRole {
  id: number;
  name: string;
  can_delete: boolean;
}

interface AdminHub {
  id: number;
  name: string;
  can_delete: boolean;
}

interface AdminMenuItem {
  id: number;
  name: string;
}

interface AdminUserListItem {
  id: number;
  name: string;
  email: string;
  roles: string[];
}

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

export interface AdminDashboardBootstrap {
  shell: {
    alerts: Array<{
      message: string;
      level: string;
    }>;
  };
  current_user: {
    email: string;
    roles: string[];
  };
  current_hub: {
    name: string;
  };
  current_page: string;
  menu: NavigationMenuItem[];
  roles: AdminRole[];
  hubs: AdminHub[];
  admin_menu: AdminMenuItem[];
  users: AdminUserListItem[];
}

function confirmDelete(event: FormEvent<HTMLFormElement>) {
  if (!window.confirm("Удалить?")) {
    event.preventDefault();
  }
}

function AdminUserModalContent({ data }: { data: AdminUserModalBootstrap }) {
  if (!data.user) {
    return (
      <div className="modal-body">
        <div className="alert alert-warning mb-0">Пользователь не найден.</div>
      </div>
    );
  }

  return (
    <>
      <div className="modal-body" key={data.user.id}>
        <form action={`/admin/user/update/${data.user.id}`} method="POST">
          <input type="hidden" value={data.user.id} name="id" required />
          <div className="row mb-3">
            <label htmlFor="modalUserEmail" className="col-md-2 col-form-label">
              Электронный адрес
            </label>
            <div className="col-md-10">
              <input
                type="email"
                readOnly
                className="form-control-plaintext"
                id="modalUserEmail"
                value={data.user.email}
                placeholder="Электронный адрес"
                required
              />
            </div>
          </div>
          <div className="row mb-3">
            <label htmlFor="modalUserName" className="col-md-2 col-form-label">
              Имя
            </label>
            <div className="col-md-10">
              <input
                name="name"
                type="text"
                className="form-control"
                id="modalUserName"
                defaultValue={data.user.name}
                placeholder="Имя"
                required
              />
            </div>
          </div>
          <div className="mb-3 row">
            <label htmlFor="password" className="col-sm-2 col-form-label">
              Пароль
            </label>
            <div className="col-sm-10">
              <input
                type="password"
                className="form-control"
                id="password"
                name="password"
                placeholder="*****"
              />
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
              <select
                multiple
                className="form-control my-1"
                name="roles"
                id="user-assign-form-role-id"
                defaultValue={data.user.roles.map(String)}
              >
                {data.roles.map((role) => (
                  <option key={role.id} value={role.id}>
                    {role.name}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <div className="row mb-3">
            <div className="col">
              <button className="btn btn-primary" type="submit">
                Сохранить
              </button>
            </div>
          </div>
        </form>
      </div>

      <form
        action={`/admin/user/delete/${data.user.id}`}
        method="POST"
        onSubmit={confirmDelete}
      >
        <div className="modal-footer">
          <button className="btn btn-danger" type="submit">
            Удалить
          </button>
        </div>
      </form>
    </>
  );
}

export function MainAdminPage({
  shell,
  current_user,
  current_hub,
  current_page,
  menu,
  roles,
  hubs,
  admin_menu,
  users,
}: AdminDashboardBootstrap) {
  const [filterValue, setFilterValue] = useState("");
  const [isLoadingModal, setIsLoadingModal] = useState(false);
  const [modalError, setModalError] = useState<string | null>(null);
  const [modalData, setModalData] = useState<AdminUserModalBootstrap | null>(
    null,
  );
  const modalRef = useRef<HTMLDivElement | null>(null);
  const requestIdRef = useRef(0);

  const normalizedFilter = filterValue.trim().toLowerCase();
  const filteredUsers = users.filter((user) => {
    if (!normalizedFilter) {
      return true;
    }

    return [user.name, user.email, user.roles.join(" ")]
      .join(" ")
      .toLowerCase()
      .includes(normalizedFilter);
  });

  async function openUserModal(userId: number) {
    const requestId = requestIdRef.current + 1;
    requestIdRef.current = requestId;
    setIsLoadingModal(true);
    setModalError(null);
    setModalData(null);

    if (modalRef.current) {
      window.bootstrap.Modal.getOrCreateInstance(modalRef.current).show();
    }

    try {
      const response = await fetch(`/admin/user/modal/${userId}`, {
        method: "POST",
        headers: {
          Accept: "application/json",
        },
      });

      if (!response.ok) {
        throw new Error(`Request failed with status ${response.status}`);
      }

      const data = (await response.json()) as AdminUserModalBootstrap;

      if (requestIdRef.current !== requestId) {
        return;
      }

      setModalData(data);
    } catch (error) {
      if (requestIdRef.current !== requestId) {
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

  function handleRowKeyDown(
    event: KeyboardEvent<HTMLDivElement>,
    userId: number,
  ) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      void openUserModal(userId);
    }
  }

  return (
    <AppShell alerts={shell.alerts}>
      <Navigation
        currentHubName={current_hub.name}
        currentPage={current_page}
        currentUserEmail={current_user.email}
        menu={menu}
      />

      <div className="container my-2">
        <div className="row">
          <div className="col-md">
            <h5>Роли</h5>
            <form method="POST" action="/admin/role/add">
              <div className="row">
                <div className="col">
                  <input
                    className="form-control my-1"
                    type="text"
                    name="name"
                    placeholder="Название"
                    required
                  />
                </div>
                <div className="col-auto">
                  <button className="btn btn-primary my-1" type="submit">
                    <i className="bi bi-plus"></i>
                  </button>
                </div>
              </div>
            </form>
            {roles.map((role) =>
              role.can_delete ? (
                <form
                  key={role.id}
                  action={`/admin/role/delete/${role.id}`}
                  method="POST"
                  style={{ display: "inline-block" }}
                  onSubmit={confirmDelete}
                >
                  <button
                    type="submit"
                    className="btn btn-sm btn-outline-secondary mt-1"
                  >
                    {role.name}
                    <span className="badge rounded-pill bg-danger">
                      <i className="bi bi-x"></i>
                    </span>
                  </button>
                </form>
              ) : (
                <button
                  key={role.id}
                  type="button"
                  className="btn btn-sm btn-outline-secondary mt-1"
                >
                  {role.name}
                </button>
              ),
            )}
          </div>

          <div className="col-md">
            <h5>Хабы</h5>
            <form method="POST" action="/admin/hub/add">
              <div className="row">
                <div className="col">
                  <input
                    className="form-control my-1"
                    type="text"
                    name="name"
                    placeholder="Название"
                    required
                  />
                </div>
                <div className="col-auto">
                  <button className="btn btn-primary my-1" type="submit">
                    <i className="bi bi-plus"></i>
                  </button>
                </div>
              </div>
            </form>
            {hubs.map((hub) =>
              hub.can_delete ? (
                <form
                  key={hub.id}
                  action={`/admin/hub/delete/${hub.id}`}
                  method="POST"
                  style={{ display: "inline-block" }}
                  onSubmit={confirmDelete}
                >
                  <button
                    type="submit"
                    className="btn btn-sm btn-outline-secondary mt-1"
                  >
                    {hub.name}
                    <span className="badge rounded-pill bg-danger">
                      <i className="bi bi-x"></i>
                    </span>
                  </button>
                </form>
              ) : (
                <button
                  key={hub.id}
                  type="button"
                  className="btn btn-sm btn-outline-secondary mt-1"
                >
                  {hub.name}
                </button>
              ),
            )}
          </div>

          <div className="col-md">
            <h5>Меню</h5>
            <form method="POST" action="/admin/menu/add">
              <div className="row">
                <div className="col">
                  <input
                    className="form-control my-1"
                    type="text"
                    name="name"
                    placeholder="Название"
                    required
                  />
                </div>
                <div className="col">
                  <input
                    className="form-control my-1"
                    type="text"
                    name="url"
                    placeholder="URL"
                    required
                  />
                </div>
                <div className="col-auto">
                  <button className="btn btn-primary my-1" type="submit">
                    <i className="bi bi-plus"></i>
                  </button>
                </div>
              </div>
            </form>
            {admin_menu.map((menuItem) => (
              <form
                key={menuItem.id}
                action={`/admin/menu/delete/${menuItem.id}`}
                method="POST"
                style={{ display: "inline-block" }}
                onSubmit={confirmDelete}
              >
                <button
                  type="submit"
                  className="btn btn-sm btn-outline-secondary mt-1"
                >
                  {menuItem.name}
                  <span className="badge rounded-pill bg-danger">
                    <i className="bi bi-x"></i>
                  </span>
                </button>
              </form>
            ))}
          </div>
        </div>
      </div>

      {users.length > 0 ? (
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
                ) : modalData ? (
                  <AdminUserModalContent
                    key={modalData.user?.id ?? 0}
                    data={modalData}
                  />
                ) : (
                  <div className="modal-body"></div>
                )}
              </div>
            </div>
          </div>
        </>
      ) : null}
    </AppShell>
  );
}
