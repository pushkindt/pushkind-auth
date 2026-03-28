export interface UserMenuItem {
  name: string;
  url: string;
  iconClass?: string;
}

type UserMenuDropdownProps = {
  currentUserEmail: string;
  localItems?: UserMenuItem[];
  fetchedItems: UserMenuItem[];
  logoutAction: string;
};

export function UserMenuDropdown({
  currentUserEmail,
  localItems = [],
  fetchedItems,
  logoutAction,
}: UserMenuDropdownProps) {
  const hasNavigationItems = localItems.length > 0 || fetchedItems.length > 0;

  return (
    <div className="dropdown-center">
      <button
        className="btn btn-link nav-link align-items-center text-muted dropdown-toggle"
        type="button"
        data-bs-toggle="dropdown"
        aria-expanded="false"
      >
        <i className="bi bi-person-circle fs-4"></i>
      </button>
      <ul className="dropdown-menu dropdown-menu-end">
        <li>
          <h6 className="dropdown-header">{currentUserEmail}</h6>
        </li>
        {hasNavigationItems ? (
          <li>
            <hr className="dropdown-divider" />
          </li>
        ) : null}
        {localItems.map((item) => (
          <li key={`local-${item.url}-${item.name}`}>
            <a className="dropdown-item icon-link" href={item.url}>
              <i className={`${item.iconClass ?? "bi bi-grid"} mb-2`}></i>
              {item.name}
            </a>
          </li>
        ))}
        {fetchedItems.map((item) => (
          <li key={`fetched-${item.url}-${item.name}`}>
            <a className="dropdown-item icon-link" href={item.url}>
              <i className={`${item.iconClass ?? "bi bi-grid"} mb-2`}></i>
              {item.name}
            </a>
          </li>
        ))}
        <li>
          <form method="POST" action={logoutAction}>
            <button type="submit" className="dropdown-item icon-link">
              <i className="bi bi-box-arrow-right mb-2"></i>
              Выйти
            </button>
          </form>
        </li>
      </ul>
    </div>
  );
}
