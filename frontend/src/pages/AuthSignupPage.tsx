import { useMemo, useState } from "react";
import type { FormEvent } from "react";

import { AuthModalFlashShell } from "../components/AuthModalFlashShell";
import { postForm, toFieldErrorMap, type ApiMutationError } from "../lib/api";
import { getNextFromLocation, type HubOption, withNext } from "../lib/auth";

export type SignupPageData = HubOption[];

export function AuthSignupPage({ hubs }: { hubs: SignupPageData }) {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [passwordConfirmation, setPasswordConfirmation] = useState("");
  const [hubId, setHubId] = useState("");
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});
  const [isSubmitting, setIsSubmitting] = useState(false);
  const next = getNextFromLocation();

  const passwordsMatch = useMemo(
    () => password === passwordConfirmation,
    [password, passwordConfirmation],
  );

  const submitClassName = passwordsMatch
    ? "btn btn-primary text-white"
    : "btn btn-danger text-white";

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!passwordsMatch) {
      return;
    }

    setIsSubmitting(true);
    setFieldErrors({});

    const body = new URLSearchParams();
    body.set("email", email);
    body.set("password", password);
    body.set("hub_id", hubId);

    try {
      const result = await postForm("/auth/register", body);
      window.showFlashMessage?.(result.message, "success");
      window.location.assign(result.redirect_to ?? "/auth/signin");
    } catch (error) {
      const mutationError = error as ApiMutationError;
      setFieldErrors(toFieldErrorMap(mutationError));
      window.showFlashMessage?.(mutationError.message, "danger");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <AuthModalFlashShell>
      <div className="row justify-content-center">
        <div className="col-md-6">
          <div className="card mt-5">
            <div className="card-header text-muted fw-bold">Регистрация</div>
            <div className="card-body">
              <form onSubmit={(event) => void handleSubmit(event)}>
                <div className="row mb-3">
                  <label className="col-md-4 col-form-label" htmlFor="email">
                    Электронная почта
                  </label>
                  <div className="col-md-6">
                    <input
                      autoFocus
                      className={
                        fieldErrors.email
                          ? "form-control is-invalid"
                          : "form-control"
                      }
                      id="email"
                      name="email"
                      required
                      type="email"
                      value={email}
                      onChange={(event) => {
                        setEmail(event.target.value);
                        setFieldErrors((errors) => ({ ...errors, email: "" }));
                      }}
                    />
                    {fieldErrors.email ? (
                      <div className="invalid-feedback">
                        {fieldErrors.email}
                      </div>
                    ) : null}
                  </div>
                </div>
                <div className="row mb-3">
                  <label className="col-md-4 col-form-label" htmlFor="password">
                    Пароль
                  </label>
                  <div className="col-md-6">
                    <input
                      className={
                        fieldErrors.password
                          ? "form-control is-invalid"
                          : "form-control"
                      }
                      id="password"
                      name="password"
                      required
                      type="password"
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
                <div className="row mb-3">
                  <label
                    className="col-md-4 col-form-label"
                    htmlFor="password2"
                  >
                    Повторите пароль
                  </label>
                  <div className="col-md-6">
                    <input
                      className="form-control"
                      id="password2"
                      required
                      type="password"
                      value={passwordConfirmation}
                      onChange={(event) =>
                        setPasswordConfirmation(event.target.value)
                      }
                    />
                  </div>
                </div>
                <div className="row mb-3">
                  <label className="col-md-4 col-form-label" htmlFor="hub_id">
                    Хаб
                  </label>
                  <div className="col-md-6">
                    <select
                      className={
                        fieldErrors.hub_id
                          ? "form-select is-invalid"
                          : "form-select"
                      }
                      id="hub_id"
                      name="hub_id"
                      required
                      defaultValue=""
                      onChange={(event) => {
                        setHubId(event.target.value);
                        setFieldErrors((errors) => ({ ...errors, hub_id: "" }));
                      }}
                    >
                      <option value="" disabled>
                        Выбор хаба
                      </option>
                      {hubs.map((hub) => (
                        <option key={hub.id} value={hub.id}>
                          {hub.name}
                        </option>
                      ))}
                    </select>
                    {fieldErrors.hub_id ? (
                      <div className="invalid-feedback">
                        {fieldErrors.hub_id}
                      </div>
                    ) : null}
                  </div>
                </div>
                <div className="row mb-3">
                  <div className="col-md-6 offset-md-4">
                    <input
                      className={submitClassName}
                      disabled={!passwordsMatch || isSubmitting}
                      id="submit"
                      name="submit"
                      type="submit"
                      value="Регистрация"
                    />
                    <a
                      href={withNext("/auth/signin", next)}
                      className="btn btn-link"
                    >
                      Авторизация
                    </a>
                  </div>
                </div>
              </form>
            </div>
          </div>
        </div>
      </div>
    </AuthModalFlashShell>
  );
}
