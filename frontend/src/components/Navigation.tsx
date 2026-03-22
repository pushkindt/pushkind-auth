export interface NavigationMenuItem {
  name: string;
  url: string;
}

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
              <li>
                <hr className="dropdown-divider" />
              </li>
              <li>
                <form method="POST" action="/auth/logout">
                  <button type="submit" className="dropdown-item icon-link">
                    <i className="bi bi-box-arrow-right mb-2"></i>
                    Выйти
                  </button>
                </form>
              </li>
            </ul>
          </div>
        </div>
      </nav>
    </div>
  );
}
