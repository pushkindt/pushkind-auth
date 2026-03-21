import { AppShell } from "../components/AppShell";
import { Navigation, type NavigationMenuItem } from "../components/Navigation";

export interface BasicDashboardBootstrap {
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
  user_name: string | null;
}

export function MainBasicPage({
  shell,
  current_user,
  current_hub,
  current_page,
  menu,
  user_name,
}: BasicDashboardBootstrap) {
  const rolesValue = current_user.roles.join(" ");

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
          <div className="col">
            <div className="alert alert-primary">
              Для продолжения, откройте тот сайт, который вы хотели открыть,
              если это не произошло автоматически.
            </div>
            <form method="POST" action="/user/save">
              <div className="mb-3 row">
                <label htmlFor="email" className="col-sm-2 col-form-label">
                  Электронная почта
                </label>
                <div className="col-sm-10">
                  <input
                    type="text"
                    readOnly
                    className="form-control-plaintext"
                    id="email"
                    value={current_user.email}
                  />
                </div>
              </div>
              <div className="mb-3 row">
                <label htmlFor="roles" className="col-sm-2 col-form-label">
                  Роли
                </label>
                <div className="col-sm-10">
                  <input
                    type="text"
                    readOnly
                    className="form-control-plaintext"
                    id="roles"
                    value={rolesValue}
                  />
                </div>
              </div>
              <div className="mb-3 row">
                <label htmlFor="name" className="col-sm-2 col-form-label">
                  Имя
                </label>
                <div className="col-sm-10">
                  <input
                    type="text"
                    className="form-control"
                    id="name"
                    name="name"
                    defaultValue={user_name ?? ""}
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
              <button type="submit" className="btn btn-primary">
                Изменить
              </button>
            </form>
          </div>
        </div>
      </div>
    </AppShell>
  );
}
