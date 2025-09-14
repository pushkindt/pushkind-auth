use mockall::mock;
use pushkind_common::repository::errors::RepositoryResult;

use crate::domain::hub::{Hub, NewHub};
use crate::domain::menu::{Menu, NewMenu};
use crate::domain::role::{NewRole, Role};
use crate::domain::user::{NewUser, UpdateUser, User, UserWithRoles};
use crate::repository::{
    HubReader, HubWriter, MenuReader, MenuWriter, RoleReader, RoleWriter, UserListQuery,
    UserReader, UserWriter,
};

mock! {
    pub Repository {}

    impl UserReader for Repository {
        fn get_user_by_id(&self, id: i32, hub_id: i32) -> RepositoryResult<Option<UserWithRoles>>;
        fn get_user_by_email(&self, email: &str, hub_id: i32) -> RepositoryResult<Option<UserWithRoles>>;
        fn list_users(&self, query: UserListQuery) -> RepositoryResult<(usize, Vec<UserWithRoles>)>;
        fn search_users(&self, query: UserListQuery) -> RepositoryResult<(usize, Vec<UserWithRoles>)>;
        fn login(&self, email: &str, password: &str, hub_id: i32) -> RepositoryResult<Option<UserWithRoles>>;
        fn get_roles(&self, user_id: i32) -> RepositoryResult<Vec<Role>>;
        fn verify_password(&self, password: &str, stored_hash: &str) -> bool;
    }

    impl UserWriter for Repository {
        fn create_user(&self, new_user: &NewUser) -> RepositoryResult<User>;
        fn assign_roles_to_user(&self, user_id: i32, role_ids: &[i32]) -> RepositoryResult<usize>;
        fn update_user(&self, user_id: i32, hub_id: i32, updates: &UpdateUser) -> RepositoryResult<User>;
        fn delete_user(&self, user_id: i32) -> RepositoryResult<usize>;
    }

    impl RoleReader for Repository {
        fn get_role_by_id(&self, id: i32) -> RepositoryResult<Option<Role>>;
        fn get_role_by_name(&self, name: &str) -> RepositoryResult<Option<Role>>;
        fn list_roles(&self) -> RepositoryResult<Vec<Role>>;
    }

    impl RoleWriter for Repository {
        fn create_role(&self, new_role: &NewRole) -> RepositoryResult<Role>;
        fn delete_role(&self, role_id: i32) -> RepositoryResult<usize>;
    }

    impl MenuReader for Repository {
        fn get_menu_by_id(&self, id: i32, hub_id: i32) -> RepositoryResult<Option<Menu>>;
        fn list_menu(&self, hub_id: i32) -> RepositoryResult<Vec<Menu>>;
    }

    impl MenuWriter for Repository {
        fn create_menu(&self, new_menu: &NewMenu) -> RepositoryResult<Menu>;
        fn delete_menu(&self, menu_id: i32) -> RepositoryResult<usize>;
    }

    impl HubReader for Repository {
        fn get_hub_by_id(&self, id: i32) -> RepositoryResult<Option<Hub>>;
        fn get_hub_by_name(&self, name: &str) -> RepositoryResult<Option<Hub>>;
        fn list_hubs(&self) -> RepositoryResult<Vec<Hub>>;
    }

    impl HubWriter for Repository {
        fn create_hub(&self, new_hub: &NewHub) -> RepositoryResult<Hub>;
        fn delete_hub(&self, hub_id: i32) -> RepositoryResult<usize>;
    }
}
