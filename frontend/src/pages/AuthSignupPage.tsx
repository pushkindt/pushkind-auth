import { useMemo, useState } from "react";

import { AppShell, type FlashAlert } from "../components/AppShell";
import { type AuthPageBootstrap, withNext } from "../lib/auth";

export interface SignupPageBootstrap extends AuthPageBootstrap {}

export function AuthSignupPage({ shell, next, hubs }: SignupPageBootstrap) {
  const [password, setPassword] = useState("");
  const [passwordConfirmation, setPasswordConfirmation] = useState("");

  const passwordsMatch = useMemo(
    () => password === passwordConfirmation,
    [password, passwordConfirmation],
  );

  const submitClassName = passwordsMatch
    ? "btn btn-primary text-white"
    : "btn btn-danger text-white";

  return (
    <AppShell alerts={shell.alerts}>
      <div className="row justify-content-center">
        <div className="col-md-6">
          <div className="card mt-5">
            <div className="card-header text-muted fw-bold">Регистрация</div>
            <div className="card-body">
              <form method="POST" action={withNext("/auth/register", next)}>
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
                    />
                  </div>
                </div>
                <div className="row mb-3">
                  <label className="col-md-4 col-form-label" htmlFor="password">
                    Пароль
                  </label>
                  <div className="col-md-6">
                    <input
                      className="form-control"
                      id="password"
                      name="password"
                      required
                      type="password"
                      value={password}
                      onChange={(event) => setPassword(event.target.value)}
                    />
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
                      className="form-select"
                      id="hub_id"
                      name="hub_id"
                      required
                      defaultValue=""
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
                      className={submitClassName}
                      disabled={!passwordsMatch}
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
    </AppShell>
  );
}
