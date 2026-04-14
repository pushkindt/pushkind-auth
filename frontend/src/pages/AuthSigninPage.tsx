import { useState } from "react";
import type { FormEvent } from "react";

import { AuthModalFlashShell } from "../components/AuthModalFlashShell";
import { postForm, toFieldErrorMap, type ApiMutationError } from "../lib/api";
import { getNextFromLocation, type HubOption, withNext } from "../lib/auth";

export type SigninPageData = HubOption[];

export function AuthSigninPage({ hubs }: { hubs: SigninPageData }) {
  const [passwordVisible, setPasswordVisible] = useState(false);
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [hubId, setHubId] = useState("");
  const [recoverErrors, setRecoverErrors] = useState<Record<string, string>>(
    {},
  );
  const [loginErrors, setLoginErrors] = useState<Record<string, string>>({});
  const [isSubmitting, setIsSubmitting] = useState(false);
  const next = getNextFromLocation();
  const passwordInputType = passwordVisible ? "text" : "password";
  const passwordIconClassName = passwordVisible
    ? "bi bi-eye-slash"
    : "bi bi-eye";

  const handleRecoverClick = async () => {
    if (!email.trim()) {
      const emailInput = document.getElementById("email");
      if (emailInput instanceof HTMLInputElement) {
        emailInput.reportValidity();
      }
      return;
    }

    if (!hubId) {
      const hubSelect = document.getElementById("hub_id");
      if (hubSelect instanceof HTMLSelectElement) {
        hubSelect.reportValidity();
      }
      return;
    }

    const shouldRecover = window.confirm("Выслать на электронную почту?");
    if (!shouldRecover) {
      return;
    }

    try {
      const body = new URLSearchParams();
      body.set("email", email);
      body.set("hub_id", hubId);

      const result = await postForm("/auth/recover", body);
      setRecoverErrors({});
      window.showFlashMessage?.(result.message, "success");
    } catch (error) {
      const mutationError = error as ApiMutationError;
      setRecoverErrors(toFieldErrorMap(mutationError));
      window.showFlashMessage?.(mutationError.message, "danger");
    }
  };

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsSubmitting(true);
    setLoginErrors({});

    const body = new URLSearchParams();
    body.set("email", email);
    body.set("password", password);
    body.set("hub_id", hubId);

    try {
      const result = await postForm(withNext("/auth/login", next), body);
      window.location.assign(result.redirect_to ?? "/");
    } catch (error) {
      const mutationError = error as ApiMutationError;
      setLoginErrors(toFieldErrorMap(mutationError));
      window.showFlashMessage?.(mutationError.message, "danger");
    } finally {
      setIsSubmitting(false);
    }
  }

  const emailError = recoverErrors.email || loginErrors.email;
  const hubError = recoverErrors.hub_id || loginErrors.hub_id;
  const passwordError = loginErrors.password;
  const emailClassName = emailError
    ? "form-control is-invalid"
    : "form-control";
  const hubClassName = hubError ? "form-select is-invalid" : "form-select";
  const passwordClassName = passwordError
    ? "form-control is-invalid"
    : "form-control";

  return (
    <AuthModalFlashShell>
      <div className="row justify-content-center">
        <div className="col-md-6">
          <div className="card mt-5">
            <div className="card-header text-muted fw-bold">Авторизация</div>
            <div className="card-body">
              <form onSubmit={(event) => void handleSubmit(event)}>
                <div className="row mb-3">
                  <label className="col-md-4 col-form-label" htmlFor="email">
                    Электронная почта
                  </label>
                  <div className="col-md-6">
                    <input
                      autoFocus
                      className={emailClassName}
                      id="email"
                      name="email"
                      required
                      type="email"
                      defaultValue=""
                      onChange={(event) => {
                        setEmail(event.target.value);
                        setRecoverErrors((errors) => ({
                          ...errors,
                          email: "",
                        }));
                        setLoginErrors((errors) => ({ ...errors, email: "" }));
                      }}
                    />
                    {emailError ? (
                      <div className="invalid-feedback">{emailError}</div>
                    ) : null}
                  </div>
                </div>
                <div className="row mb-3">
                  <label className="col-md-4 col-form-label" htmlFor="password">
                    Пароль
                  </label>
                  <div className="col-md-6">
                    <div className="input-group">
                      <button
                        className="btn btn-outline-secondary"
                        type="button"
                        onClick={() => void handleRecoverClick()}
                        title="Восстановить пароль"
                      >
                        ?
                      </button>
                      <input
                        type={passwordInputType}
                        className={passwordClassName}
                        id="password"
                        name="password"
                        required
                        value={password}
                        onChange={(event) => {
                          setPassword(event.target.value);
                          setLoginErrors((errors) => ({
                            ...errors,
                            password: "",
                          }));
                        }}
                      />
                      <button
                        className="btn btn-outline-secondary"
                        type="button"
                        id="togglePassword"
                        tabIndex={-1}
                        onClick={() => setPasswordVisible((value) => !value)}
                      >
                        <i
                          className={passwordIconClassName}
                          id="togglePasswordIcon"
                        ></i>
                      </button>
                    </div>
                    {passwordError ? (
                      <div className="invalid-feedback d-block">
                        {passwordError}
                      </div>
                    ) : null}
                  </div>
                </div>
                <div className="row mb-3">
                  <label className="col-md-4 col-form-label" htmlFor="hub_id">
                    Хаб
                  </label>
                  <div className="col-md-6">
                    <select
                      className={hubClassName}
                      id="hub_id"
                      name="hub_id"
                      required
                      defaultValue=""
                      onChange={(event) => {
                        setHubId(event.target.value);
                        setRecoverErrors((errors) => ({
                          ...errors,
                          hub_id: "",
                        }));
                        setLoginErrors((errors) => ({ ...errors, hub_id: "" }));
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
                    {hubError ? (
                      <div className="invalid-feedback">{hubError}</div>
                    ) : null}
                  </div>
                </div>
                <div className="row mb-3">
                  <div className="col-md-6 offset-md-4">
                    <input
                      className="btn btn-primary text-white"
                      id="submit"
                      name="submit"
                      type="submit"
                      value="Авторизация"
                      disabled={isSubmitting}
                    />
                    <a
                      href={withNext("/auth/signup", next)}
                      className="btn btn-link"
                    >
                      Регистрация
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
