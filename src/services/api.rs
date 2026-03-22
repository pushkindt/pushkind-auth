//! API services for retrieving and listing users.

use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::pagination::DEFAULT_ITEMS_PER_PAGE;
use pushkind_common::routes::ensure_role;
use pushkind_common::services::errors::{ServiceError, ServiceResult};

use crate::SERVICE_ACCESS_ROLE;
use crate::domain::types::{HubId, UserEmail, UserId};
use crate::dto::api::{
    AdminDashboardDto, AdminHubItemDto, AdminMenuItemDto, AdminRoleItemDto, ApiV1UsersQueryParams,
    EditableProfileDto, HubListItemDto, HubMenuItemDto, HubSummaryDto, IamDto, UserDto,
};
use crate::repository::{HubReader, MenuReader, RoleReader, UserListQuery, UserReader};

/// Returns the authenticated user when `id` is `None`, otherwise
/// attempts to fetch the user by `id` limited to the current hub.
///
/// On success returns `Some(user)`; returns `None` when not found.
pub fn get_user_by_optional_id(
    id: Option<i32>,
    current_user: AuthenticatedUser,
    repo: &impl UserReader,
) -> ServiceResult<Option<UserDto>> {
    let hub_id = HubId::new(current_user.hub_id)?;
    match id {
        None => Ok(Some(current_user.into())),
        Some(id) => {
            let user_id = UserId::new(id)?;
            let found = repo.get_user_by_id(user_id, hub_id)?;
            Ok(found.map(|u| UserDto::from(AuthenticatedUser::from(u.user))))
        }
    }
}

/// Lists users for a hub with optional role filter, search query,
/// and pagination. Returns only the users (total is ignored upstream).
pub fn list_users(
    query: ApiV1UsersQueryParams,
    current_user: &AuthenticatedUser,
    repo: &impl UserReader,
) -> ServiceResult<Vec<UserDto>> {
    let hub_id = HubId::new(current_user.hub_id)?;
    let mut list_query = UserListQuery::new(hub_id);

    if let Some(role) = &query.role {
        list_query = list_query.role(role);
    }

    if let Some(page) = query.page {
        list_query = list_query.paginate(page, DEFAULT_ITEMS_PER_PAGE);
    }

    if let Some(query) = &query.query {
        list_query = list_query.search(query);
    }

    let (_total, users_with_roles) = repo.list_users(list_query)?;

    let users = users_with_roles
        .into_iter()
        .map(|u| UserDto::from(AuthenticatedUser::from(u)))
        .collect();

    Ok(users)
}

/// Lists hubs in a DTO shape suitable for the future `/api/v1/hubs` endpoint.
pub fn list_hubs(repo: &impl HubReader) -> ServiceResult<Vec<HubListItemDto>> {
    let hubs = repo.list_hubs()?;
    Ok(hubs.into_iter().map(HubListItemDto::from).collect())
}

/// Builds an IAM-style DTO around the current authenticated user.
///
/// This helper is intentionally layered on top of [`UserDto`] so the existing
/// `/api/v1/id` endpoint can be reused or extended instead of duplicating the
/// current-user identity contract.
pub fn get_iam(
    current_user: AuthenticatedUser,
    repo: &(impl HubReader + UserReader),
) -> ServiceResult<IamDto> {
    let hub_id = HubId::new(current_user.hub_id)?;
    let email = UserEmail::new(&current_user.email)?;
    let hub = repo.get_hub_by_id(hub_id)?.ok_or(ServiceError::NotFound)?;
    let user_name = repo
        .get_user_by_email(&email, hub_id)?
        .and_then(|user| user.user.name.map(|name| name.into_inner()))
        .unwrap_or_default();

    Ok(IamDto {
        user: UserDto::from(current_user),
        current_hub: HubSummaryDto::from(hub),
        editable_profile: EditableProfileDto { name: user_name },
    })
}

/// Lists menu items for the authenticated user's current hub.
///
/// The future `/api/v1/hubs/{hub_id}/menu-items` route should continue to
/// enforce that callers cannot request menu data for a different hub.
pub fn list_hub_menu_items(
    requested_hub_id: i32,
    current_user: &AuthenticatedUser,
    repo: &impl MenuReader,
) -> ServiceResult<Vec<HubMenuItemDto>> {
    if requested_hub_id != current_user.hub_id {
        return Err(ServiceError::Unauthorized);
    }

    let hub_id = HubId::new(requested_hub_id)?;
    let menu_items = repo.list_menu(hub_id)?;

    Ok(menu_items.into_iter().map(HubMenuItemDto::from).collect())
}

/// Builds the admin-only aggregate excluding the users table.
///
/// The future admin dashboard API should reuse `/api/v1/users` for the users
/// list instead of duplicating that payload inside the admin aggregate.
pub fn get_admin_dashboard_data(
    current_user: &AuthenticatedUser,
    repo: &(impl RoleReader + HubReader + MenuReader),
) -> ServiceResult<AdminDashboardDto> {
    ensure_role(current_user, SERVICE_ACCESS_ROLE)?;

    let hub_id = HubId::new(current_user.hub_id)?;
    let roles = repo.list_roles()?;
    let hubs = repo.list_hubs()?;
    let admin_menu = repo.list_menu(hub_id)?;

    Ok(AdminDashboardDto {
        roles: roles.into_iter().map(AdminRoleItemDto::from).collect(),
        hubs: hubs.into_iter().map(AdminHubItemDto::from).collect(),
        admin_menu: admin_menu.into_iter().map(AdminMenuItemDto::from).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::hub::Hub;
    use crate::domain::menu::Menu;
    use crate::domain::role::Role;
    use crate::domain::types::{
        HubId, HubName, MenuId, MenuName, MenuUrl, RoleId, RoleName, UserEmail, UserId,
    };
    use crate::domain::user::{User, UserWithRoles};
    use crate::repository::mock::MockRepository;
    use chrono::Utc;

    fn make_user(id: i32, email: &str, hub_id: i32) -> UserWithRoles {
        let now = Utc::now().naive_utc();
        let user = User::new(
            UserId::new(id).unwrap(),
            UserEmail::new(email).unwrap(),
            Some(crate::domain::types::UserName::new(format!("User{id}")).unwrap()),
            HubId::new(hub_id).unwrap(),
            "hash".into(),
            now,
            now,
            vec![],
        );
        UserWithRoles::new(user, vec![])
    }

    fn make_hub(id: i32, name: &str) -> Hub {
        let now = Utc::now().naive_utc();
        Hub::new(
            HubId::new(id).unwrap(),
            HubName::new(name).unwrap(),
            now,
            now,
        )
    }

    fn make_menu(id: i32, hub_id: i32, name: &str, url: &str) -> Menu {
        Menu::new(
            MenuId::new(id).unwrap(),
            MenuName::new(name).unwrap(),
            MenuUrl::new(format!("https://example.com{url}")).unwrap(),
            HubId::new(hub_id).unwrap(),
        )
    }

    fn make_role(id: i32, name: &str) -> Role {
        let now = Utc::now().naive_utc();
        Role::new(
            RoleId::new(id).unwrap(),
            RoleName::new(name).unwrap(),
            now,
            now,
        )
    }

    #[test]
    fn get_user_by_optional_id_none_returns_current() {
        let repo = MockRepository::new();
        let current = AuthenticatedUser {
            sub: "42".into(),
            email: "me@hub".into(),
            hub_id: 10,
            name: "Me".into(),
            roles: vec![],
            exp: 0,
        };
        let res = get_user_by_optional_id(None, current, &repo).unwrap();
        assert_eq!(res.unwrap().email, "me@hub");
    }

    #[test]
    fn get_user_by_optional_id_some_found() {
        let mut repo = MockRepository::new();
        let user = make_user(1, "user1@example.com", 10);
        repo.expect_get_user_by_id()
            .returning(move |_, _| Ok(Some(user.clone())));
        let current = AuthenticatedUser {
            sub: "1".into(),
            email: "user1@example.com".into(),
            hub_id: 10,
            name: "User1".into(),
            roles: vec![],
            exp: 0,
        };
        let res = get_user_by_optional_id(Some(1), current, &repo).unwrap();
        assert_eq!(res.unwrap().sub, "1");
    }

    #[test]
    fn get_user_by_optional_id_some_not_found() {
        let mut repo = MockRepository::new();
        repo.expect_get_user_by_id().returning(|_, _| Ok(None));
        let current = AuthenticatedUser {
            sub: "1".into(),
            email: "user1@example.com".into(),
            hub_id: 10,
            name: "User1".into(),
            roles: vec![],
            exp: 0,
        };
        let res = get_user_by_optional_id(Some(999), current, &repo).unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn list_users_no_query() {
        let mut repo = MockRepository::new();
        let u1 = make_user(1, "user1@example.com", 10);
        let u2 = make_user(2, "user2@example.com", 10);
        repo.expect_list_users()
            .returning(move |_| Ok((2, vec![u1.clone(), u2.clone()])));
        let params = ApiV1UsersQueryParams {
            role: None,
            query: None,
            page: None,
        };
        let current_user = AuthenticatedUser {
            sub: "1".into(),
            email: "user1@example.com".into(),
            hub_id: 10,
            name: "User1".into(),
            roles: vec![],
            exp: 0,
        };
        let out = list_users(params, &current_user, &repo).unwrap();
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn list_users_with_query_filters() {
        let mut repo = MockRepository::new();
        let u1 = make_user(1, "user1@example.com", 10);
        repo.expect_list_users()
            .returning(move |_| Ok((1, vec![u1.clone()])));
        let params = ApiV1UsersQueryParams {
            role: None,
            query: Some("user1".into()),
            page: None,
        };
        let current_user = AuthenticatedUser {
            sub: "1".into(),
            email: "user1@example.com".into(),
            hub_id: 10,
            name: "User1".into(),
            roles: vec![],
            exp: 0,
        };
        let out = list_users(params, &current_user, &repo).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].email, "user1@example.com");
    }

    #[test]
    fn list_users_with_role_and_pagination() {
        let mut repo = MockRepository::new();
        let u2 = make_user(2, "user2@example.com", 10);
        repo.expect_list_users()
            .returning(move |_| Ok((1, vec![u2.clone()])));
        let params = ApiV1UsersQueryParams {
            role: Some("member".into()),
            query: None,
            page: Some(1),
        };
        let current_user = AuthenticatedUser {
            sub: "2".into(),
            email: "user2@example.com".into(),
            hub_id: 10,
            name: "User2".into(),
            roles: vec![],
            exp: 0,
        };
        let out = list_users(params, &current_user, &repo).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].email, "user2@example.com");
    }

    #[test]
    fn list_hubs_maps_to_resource_dtos() {
        let mut repo = MockRepository::new();
        let hub = make_hub(10, "Main");
        repo.expect_list_hubs()
            .returning(move || Ok(vec![hub.clone()]));

        let hubs = list_hubs(&repo).unwrap();

        assert_eq!(
            hubs,
            vec![HubListItemDto {
                id: 10,
                name: "Main".into()
            }]
        );
    }

    #[test]
    fn get_iam_reuses_user_dto_and_adds_hub_context() {
        let mut repo = MockRepository::new();
        let hub = make_hub(10, "Main");
        let user = make_user(1, "user1@example.com", 10);
        repo.expect_get_hub_by_id()
            .returning(move |_| Ok(Some(hub.clone())));
        repo.expect_get_user_by_email()
            .returning(move |_, _| Ok(Some(user.clone())));

        let current_user = AuthenticatedUser {
            sub: "1".into(),
            email: "user1@example.com".into(),
            hub_id: 10,
            name: "User1".into(),
            roles: vec!["member".into()],
            exp: 123,
        };

        let iam = get_iam(current_user, &repo).unwrap();

        assert_eq!(iam.user.email, "user1@example.com");
        assert_eq!(iam.current_hub.name, "Main");
        assert_eq!(iam.editable_profile.name, "User1");
    }

    #[test]
    fn list_hub_menu_items_rejects_cross_hub_access() {
        let repo = MockRepository::new();
        let current_user = AuthenticatedUser {
            sub: "1".into(),
            email: "user1@example.com".into(),
            hub_id: 10,
            name: "User1".into(),
            roles: vec!["member".into()],
            exp: 0,
        };

        let result = list_hub_menu_items(11, &current_user, &repo);

        assert!(matches!(result, Err(ServiceError::Unauthorized)));
    }

    #[test]
    fn get_admin_dashboard_data_excludes_user_list() {
        let mut repo = MockRepository::new();
        let role = make_role(1, "admin");
        let hub = make_hub(10, "Main");
        let menu = make_menu(1, 10, "Settings", "/settings");
        repo.expect_list_roles()
            .returning(move || Ok(vec![role.clone()]));
        repo.expect_list_hubs()
            .returning(move || Ok(vec![hub.clone()]));
        repo.expect_list_menu()
            .returning(move |_| Ok(vec![menu.clone()]));

        let current_user = AuthenticatedUser {
            sub: "1".into(),
            email: "admin@example.com".into(),
            hub_id: 10,
            name: "Admin".into(),
            roles: vec!["admin".into()],
            exp: 0,
        };

        let dto = get_admin_dashboard_data(&current_user, &repo).unwrap();

        assert_eq!(dto.roles.len(), 1);
        assert_eq!(dto.hubs.len(), 1);
        assert_eq!(dto.admin_menu.len(), 1);
    }
}
