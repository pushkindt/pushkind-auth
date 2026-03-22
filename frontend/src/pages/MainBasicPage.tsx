import { useState } from "react";
import type { FormEvent } from "react";

import { AppShell } from "../components/AppShell";
import { Navigation, type NavigationMenuItem } from "../components/Navigation";
import {
  isRedirectResponseError,
  postForm,
  toFieldErrorMap,
  type ApiIam,
  type ApiMutationError,
} from "../lib/api";

export interface BasicDashboardPageData {
  iam: ApiIam;
  menu: NavigationMenuItem[];
}

export function MainBasicPage({ iam, menu }: BasicDashboardPageData) {
  const [name, setName] = useState(iam.editable_profile.name);
  const [password, setPassword] = useState("");
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});
  const [isSubmitting, setIsSubmitting] = useState(false);
  const rolesValue = iam.user.roles.join(" ");

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsSubmitting(true);
    setFieldErrors({});

    const body = new URLSearchParams();
    body.set("name", name);
    body.set("password", password);

    try {
      const result = await postForm("/user/save", body);
      setPassword("");
      window.showFlashMessage?.(result.message, "success");
    } catch (error) {
      if (isRedirectResponseError(error)) {
        return;
      }

      const mutationError = error as ApiMutationError;
      setFieldErrors(toFieldErrorMap(mutationError));
      window.showFlashMessage?.(mutationError.message, "danger");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <AppShell alerts={[]}>
      <Navigation
        currentHubName={iam.current_hub.name}
        currentPage="index"
        currentUserEmail={iam.user.email}
        menu={menu}
      />
      <div className="container my-2">
        <div className="row">
          <div className="col">
            <div className="alert alert-primary">
              Для продолжения, откройте тот сайт, который вы хотели открыть,
              если это не произошло автоматически.
            </div>
            <form onSubmit={(event) => void handleSubmit(event)}>
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
                    value={iam.user.email}
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
                    className={
                      fieldErrors.name
                        ? "form-control is-invalid"
                        : "form-control"
                    }
                    id="name"
                    name="name"
                    value={name}
                    placeholder="Имя"
                    required
                    onChange={(event) => {
                      setName(event.target.value);
                      setFieldErrors((errors) => ({ ...errors, name: "" }));
                    }}
                  />
                  {fieldErrors.name ? (
                    <div className="invalid-feedback">{fieldErrors.name}</div>
                  ) : null}
                </div>
              </div>
              <div className="mb-3 row">
                <label htmlFor="password" className="col-sm-2 col-form-label">
                  Пароль
                </label>
                <div className="col-sm-10">
                  <input
                    type="password"
                    className={
                      fieldErrors.password
                        ? "form-control is-invalid"
                        : "form-control"
                    }
                    id="password"
                    name="password"
                    placeholder="*****"
                    value={password}
                    onChange={(event) => {
                      setPassword(event.target.value);
                      setFieldErrors((errors) => ({
                        ...errors,
                        password: "",
                      }));
                    }}
                  />
                  {fieldErrors.password ? (
                    <div className="invalid-feedback">
                      {fieldErrors.password}
                    </div>
                  ) : null}
                </div>
              </div>
              <button
                type="submit"
                className="btn btn-primary"
                disabled={isSubmitting}
              >
                Изменить
              </button>
            </form>
          </div>
        </div>
      </div>
    </AppShell>
  );
}
