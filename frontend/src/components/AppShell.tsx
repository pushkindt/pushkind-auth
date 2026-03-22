import { useEffect, useRef } from "react";
import type { ReactNode } from "react";

export interface FlashAlert {
  message: string;
  level: string;
}

interface AppShellProps {
  alerts: FlashAlert[];
  children: ReactNode;
}

declare global {
  interface Window {
    bootstrap: {
      Modal: {
        getOrCreateInstance: (
          element: string | Element,
          options?: object,
        ) => {
          hide: () => void;
          show: () => void;
        };
      };
      Popover: new (element: Element) => unknown;
      Tooltip: new (element: Element) => unknown;
    };
    showFlashMessage?: (message: string, category?: string) => void;
    createFilter?: (
      filterInputId: string,
      itemContainerId: string,
      itemSelector: string,
    ) => void;
  }
}

function FlashMessages({ alerts }: { alerts: FlashAlert[] }) {
  return (
    <div id="flashMessages">
      {alerts.map((alert, index) => (
        <div
          key={`${alert.level}-${index}`}
          className={`alert alert-${alert.level} alert-dismissible`}
          role="alert"
        >
          {alert.message}
          <button
            type="button"
            className="btn-close"
            data-bs-dismiss="alert"
            aria-label="Close"
          ></button>
        </div>
      ))}
    </div>
  );
}

export function AppShell({ alerts, children }: AppShellProps) {
  const ajaxFlashContentRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const popoverTriggerList = Array.from(
      document.querySelectorAll("[data-bs-toggle='popover']"),
    );
    const tooltipTriggerList = Array.from(
      document.querySelectorAll("[data-bs-toggle='tooltip']"),
    );

    popoverTriggerList.forEach((element) => {
      new window.bootstrap.Popover(element);
    });
    tooltipTriggerList.forEach((element) => {
      new window.bootstrap.Tooltip(element);
    });

    window.showFlashMessage = (message, category = "primary") => {
      const flashes = ajaxFlashContentRef.current;
      const modal = window.bootstrap.Modal.getOrCreateInstance(
        "#ajax-flash-modal",
        {},
      );

      if (!flashes || !modal) {
        return;
      }

      flashes.innerHTML = `<div class="alert alert-${category} alert-dismissible mb-0" role="alert">${message}<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button></div>`;
      modal.show();
    };

    window.createFilter = (filterInputId, itemContainerId, itemSelector) => {
      const filter = document.getElementById(filterInputId);
      const items = document.getElementById(itemContainerId);

      if (!filter || !items) {
        return;
      }

      filter.addEventListener("keyup", () => {
        const filterValue = (filter as HTMLInputElement).value.toLowerCase();
        const itemRows = items.querySelectorAll(itemSelector);

        itemRows.forEach((row) => {
          const rowText = row.textContent?.toLowerCase() ?? "";
          const showRow = rowText.includes(filterValue);
          row.classList.toggle("d-none", !showRow);
        });
      });
    };

    return () => {
      delete window.showFlashMessage;
      delete window.createFilter;
    };
  }, []);

  return (
    <>
      <FlashMessages alerts={alerts} />
      <div className="modal" tabIndex={-1} id="ajax-flash-modal">
        <div className="modal-dialog">
          <div className="modal-content">
            <div
              className="modal-body"
              id="ajax-flash-content"
              style={{ padding: 0 }}
              ref={ajaxFlashContentRef}
            ></div>
          </div>
        </div>
      </div>
      {children}
    </>
  );
}
