use crate::domain::hub::Hub;
use crate::domain::menu::Menu;
use crate::domain::role::Role;
use crate::domain::user::{UpdateUser, UserWithRoles};
use crate::repository::{HubReader, MenuReader, RoleReader, UserListQuery, UserReader, UserWriter};
use pushkind_common::services::errors::ServiceResult;

pub struct IndexData {
    pub hub: Hub,
    pub users: Vec<UserWithRoles>,
    pub roles: Vec<Role>,
    pub hubs: Vec<Hub>,
    pub menu: Vec<Menu>,
    pub user_name: Option<String>,
}

pub fn get_index_data(
    hub_id: i32,
    user_email: &str,
    repo: &(impl HubReader + UserReader + RoleReader + MenuReader),
) -> ServiceResult<IndexData> {
    let hub = repo.get_hub_by_id(hub_id)?.ok_or_else(|| pushkind_common::services::errors::ServiceError::NotFound)?;
    let (_total, users) = repo.list_users(UserListQuery::new(hub_id))?;
    let roles = repo.list_roles()?;
    let hubs = repo.list_hubs()?;
    let menu = repo.list_menu(hub_id)?;
    let user_name = repo
        .get_user_by_email(user_email, hub_id)?
        .map(|u| u.user.name)
        .flatten();
    Ok(IndexData { hub, users, roles, hubs, menu, user_name })
}

pub fn update_current_user(
    user_id: i32,
    hub_id: i32,
    updates: &UpdateUser,
    repo: &impl UserWriter,
) -> ServiceResult<()> {
    repo.update_user(user_id, hub_id, updates)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::test::TestRepository;

    fn sample() -> TestRepository {
        let now = TestRepository::now();
        let hub = Hub { id: 5, name: "h".into(), created_at: now, updated_at: now };
        let user = crate::domain::user::User { id: 9, email: "a@b".into(), name: Some("N".into()), hub_id: 5, password_hash: "".into(), created_at: now, updated_at: now, roles: vec![] };
        let uwr = UserWithRoles { user, roles: vec![] };
        TestRepository::with_users(vec![uwr]).with_hubs(vec![hub]).with_roles(vec![]).with_menus(vec![])
    }

    #[test]
    fn test_get_index_data() {
        let repo = sample();
        let data = get_index_data(5, "a@b", &repo).unwrap();
        assert_eq!(data.hub.id, 5);
        assert_eq!(data.users.len(), 1);
        assert_eq!(data.user_name.as_deref(), Some("N"));
    }
}
