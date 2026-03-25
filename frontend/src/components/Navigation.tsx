import {
  UserMenuDropdown,
  type UserMenuItem as NavigationMenuItem,
} from "./UserMenuDropdown";

export type { NavigationMenuItem };

interface NavigationProps {
  currentHubName: string;
  currentPage: string;
  currentUserEmail: string;
  menu: NavigationMenuItem[];
}

export function Navigation({
  currentHubName,
  currentPage,
  currentUserEmail,
  menu,
}: NavigationProps) {
  return (
    <div className="container">
      <nav className="navbar navbar-expand-sm bg-body-tertiary">
        <div className="container-fluid">
          <a className="navbar-brand" href="/">
            {currentHubName}
          </a>
          <button
            className="navbar-toggler"
            type="button"
            data-bs-toggle="collapse"
            data-bs-target="#navbarSupportedContent"
            aria-controls="navbarSupportedContent"
            aria-expanded="false"
            aria-label="Toggle navigation"
          >
            <span className="navbar-toggler-icon"></span>
          </button>
          <div className="collapse navbar-collapse" id="navbarSupportedContent">
            <ul className="navbar-nav me-auto">
              <li className="nav-item">
                <a
                  className={`nav-link ${currentPage === "index" ? "active" : ""}`}
                  href="/"
                >
                  Главная
                </a>
              </li>
              {menu.map((menuItem) => (
                <li
                  key={`${menuItem.url}-${menuItem.name}`}
                  className="nav-item"
                >
                  <a className="nav-link" href={menuItem.url}>
                    {menuItem.name}
                  </a>
                </li>
              ))}
            </ul>
          </div>
          <UserMenuDropdown
            currentUserEmail={currentUserEmail}
            items={menu}
            homeUrl="/"
            homeLabel="Главная"
            logoutAction="/auth/logout"
          />
        </div>
      </nav>
    </div>
  );
}
