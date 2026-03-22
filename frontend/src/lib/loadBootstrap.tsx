import { StrictMode } from "react";
import type { ReactElement } from "react";
import { createRoot } from "react-dom/client";

function LoadingState() {
  return (
    <div className="row justify-content-center">
      <div className="col-md-6">
        <div className="card mt-5">
          <div className="card-body text-center py-5">
            <div
              className="spinner-border"
              role="status"
              aria-hidden="true"
            ></div>
          </div>
        </div>
      </div>
    </div>
  );
}

function ErrorState() {
  return (
    <div className="row justify-content-center">
      <div className="col-md-6">
        <div className="card mt-5">
          <div className="card-body">
            <div className="alert alert-danger mb-0">
              Не удалось загрузить данные страницы.
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export async function loadBootstrapPage<T>(
  rootElement: HTMLElement,
  endpoint: string,
  renderPage: (bootstrap: T) => ReactElement,
): Promise<void> {
  return loadComposedPage(
    rootElement,
    () =>
      fetch(endpoint, {
        headers: {
          Accept: "application/json",
        },
      }).then(async (response) => {
        if (!response.ok) {
          throw new Error(
            `Bootstrap request failed with status ${response.status}`,
          );
        }

        return (await response.json()) as T;
      }),
    renderPage,
  );
}

export async function loadComposedPage<T>(
  rootElement: HTMLElement,
  loadPage: () => Promise<T>,
  renderPage: (bootstrap: T) => ReactElement,
): Promise<void> {
  const root = createRoot(rootElement);

  root.render(
    <StrictMode>
      <LoadingState />
    </StrictMode>,
  );

  try {
    const bootstrap = await loadPage();

    root.render(<StrictMode>{renderPage(bootstrap)}</StrictMode>);
  } catch (error) {
    console.error(error);
    root.render(
      <StrictMode>
        <ErrorState />
      </StrictMode>,
    );
  }
}
