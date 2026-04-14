import { useEffect, useState } from "react";
import type { FormEvent } from "react";

import { AuthShell } from "../components/AuthShell";
import { AuthShellFatalState } from "../components/AuthShellFatalState";
import {
  isRedirectResponseError,
  postForm,
  toFieldErrorMap,
  type ApiMutationError,
} from "../lib/api";
import { useAuthShell } from "../lib/useAuthShell";

export function MainBasicPage() {
  const shellState = useAuthShell("Не удалось загрузить оболочку Auth.");
  const [name, setName] = useState("");
  const [password, setPassword] = useState("");
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    if (shellState.status === "ready") {
      setName(shellState.shell.currentUser.name);
    }
  }, [shellState]);

  if (shellState.status === "loading") {
    return null;
  }

  if (shellState.status === "error") {
    return <AuthShellFatalState message={shellState.message} />;
  }

  const rolesValue = shellState.shell.currentUser.roles.join(" ");

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
    <AuthShell
      navigation={shellState.shell.navigation}
      currentUserEmail={shellState.shell.currentUser.email}
      homeUrl={shellState.shell.homeUrl}
      localMenuItems={shellState.shell.localMenuItems}
      fetchedMenuItems={shellState.authMenuItems}
      hubName={shellState.shell.hubName}
    >
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
                    value={shellState.shell.currentUser.email}
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
    </AuthShell>
  );
}
