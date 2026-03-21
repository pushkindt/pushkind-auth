import { useState } from "react";

import { AppShell } from "../components/AppShell";
import { type AuthPageBootstrap, withNext } from "../lib/auth";

export interface SigninPageBootstrap extends AuthPageBootstrap {}

async function recoverPassword(email: string, hubId: string): Promise<void> {
  const body = new URLSearchParams();
  body.set("email", email);
  body.set("hub_id", hubId);

  const response = await fetch("/auth/recover", {
    method: "POST",
    headers: {
      "Content-Type": "application/x-www-form-urlencoded;charset=UTF-8",
    },
    body: body.toString(),
    redirect: "follow",
  });

  if (response.redirected) {
    window.location.assign(response.url);
    return;
  }

  if (!response.ok) {
    throw new Error("Recovery request failed");
  }

  const message = await response.text();
  window.showFlashMessage?.(message, "success");
}

export function AuthSigninPage({ shell, next, hubs }: SigninPageBootstrap) {
  const [passwordVisible, setPasswordVisible] = useState(false);
  const [email, setEmail] = useState("");
  const [hubId, setHubId] = useState("");
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
      await recoverPassword(email, hubId);
    } catch {
      window.showFlashMessage?.(
        "Ошибка при отправке ссылки для входа.",
        "danger",
      );
    }
  };

  return (
    <AppShell alerts={shell.alerts}>
      <div className="row justify-content-center">
        <div className="col-md-6">
          <div className="card mt-5">
            <div className="card-header text-muted fw-bold">Авторизация</div>
            <div className="card-body">
              <form method="POST" action={withNext("/auth/login", next)}>
                <div className="row mb-3">
                  <label className="col-md-4 col-form-label" htmlFor="email">
                    Электронная почта
                  </label>
                  <div className="col-md-6">
                    <input
                      autoFocus
                      className="form-control"
                      id="email"
                      name="email"
                      required
                      type="email"
                      defaultValue=""
                      onChange={(event) => setEmail(event.target.value)}
                    />
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
                        className="form-control"
                        id="password"
                        name="password"
                        required
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
                  </div>
                </div>
                <div className="row mb-3">
                  <label className="col-md-4 col-form-label" htmlFor="hub_id">
                    Хаб
                  </label>
                  <div className="col-md-6">
                    <select
                      className="form-select"
                      id="hub_id"
                      name="hub_id"
                      required
                      defaultValue=""
                      onChange={(event) => setHubId(event.target.value)}
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
    </AppShell>
  );
}
