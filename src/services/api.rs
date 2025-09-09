use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::pagination::DEFAULT_ITEMS_PER_PAGE;
use pushkind_common::services::errors::ServiceResult;

use crate::repository::{UserListQuery, UserReader};

/// Returns the authenticated user when `id` is `None`, otherwise
/// attempts to fetch the user by `id` limited to the current hub.
///
/// On success returns `Some(user)`; returns `None` when not found.
pub fn get_user_by_optional_id(
    id: Option<i32>,
    current_user: AuthenticatedUser,
    repo: &impl UserReader,
) -> ServiceResult<Option<AuthenticatedUser>> {
    match id {
        None => Ok(Some(current_user)),
        Some(id) => {
            let found = repo.get_user_by_id(id, current_user.hub_id)?;
            Ok(found.map(|u| AuthenticatedUser::from(u.user)))
        }
    }
}

/// Lists users for a hub with optional role filter, search query,
/// and pagination. Returns only the users (total is ignored upstream).
pub fn list_users(
    role: Option<String>,
    query: Option<String>,
    page: Option<usize>,
    hub_id: i32,
    repo: &impl UserReader,
) -> ServiceResult<Vec<AuthenticatedUser>> {
    let mut list_query = UserListQuery::new(hub_id);

    if let Some(role) = role {
        list_query = list_query.role(role);
    }

    if let Some(page) = page {
        list_query = list_query.paginate(page, DEFAULT_ITEMS_PER_PAGE);
    }

    let (_total, users_with_roles) = match query {
        Some(q) if !q.is_empty() => repo.search_users(list_query.search(q))?,
        _ => repo.list_users(list_query)?,
    };

    let users = users_with_roles
        .into_iter()
        .map(|u| AuthenticatedUser::from(u.user))
        .collect();

    Ok(users)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::test::TestRepository;

    fn sample_repo() -> TestRepository {
        let u1 = TestRepository::make_user(1, "user1@example.com", 10, vec!["admin"]);
        let u2 = TestRepository::make_user(2, "user2@example.com", 10, vec!["member"]);
        let u3 = TestRepository::make_user(3, "other@example.com", 99, vec!["admin"]);
        TestRepository::with_users(vec![u1, u2, u3])
    }

    #[test]
    fn get_user_by_optional_id_none_returns_current() {
        let repo = sample_repo();
        let current = AuthenticatedUser { sub: "42".into(), email: "me@hub".into(), hub_id: 10, name: "Me".into(), roles: vec![], exp: 0 };
        let res = get_user_by_optional_id(None, current, &repo).unwrap();
        assert_eq!(res.unwrap().email, "me@hub");
    }

    #[test]
    fn get_user_by_optional_id_some_found() {
        let repo = sample_repo();
        let current = AuthenticatedUser { sub: "1".into(), email: "user1@example.com".into(), hub_id: 10, name: "User1".into(), roles: vec![], exp: 0 };
        let res = get_user_by_optional_id(Some(1), current, &repo).unwrap();
        assert_eq!(res.unwrap().sub, "1");
    }

    #[test]
    fn get_user_by_optional_id_some_not_found() {
        let repo = sample_repo();
        let current = AuthenticatedUser { sub: "1".into(), email: "user1@example.com".into(), hub_id: 10, name: "User1".into(), roles: vec![], exp: 0 };
        let res = get_user_by_optional_id(Some(999), current, &repo).unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn list_users_no_query() {
        let repo = sample_repo();
        let out = list_users(None, None, None, 10, &repo).unwrap();
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn list_users_with_query_filters() {
        let repo = sample_repo();
        let out = list_users(None, Some("user1".into()), None, 10, &repo).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].email, "user1@example.com");
    }

    #[test]
    fn list_users_with_role_and_pagination() {
        let repo = sample_repo();
        let out = list_users(Some("member".into()), None, Some(1), 10, &repo).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].email, "user2@example.com");
        
    }
}
