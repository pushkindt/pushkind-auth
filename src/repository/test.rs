//! Test-only in-memory repository used in unit tests.
//!
//! Provides a simple, non-mutating implementation of repository traits for
//! services to run logic against predictable data.

use chrono::{NaiveDateTime, Utc};
use pushkind_common::repository::errors::{RepositoryError, RepositoryResult};

use crate::domain::hub::{Hub, NewHub};
use crate::domain::menu::{Menu, NewMenu};
use crate::domain::role::{NewRole, Role};
use crate::domain::user::{NewUser, UpdateUser, User, UserWithRoles};
use crate::repository::{
    HubReader, HubWriter, MenuReader, MenuWriter, RoleReader, RoleWriter, UserListQuery,
    UserReader, UserWriter,
};

#[derive(Default, Clone)]
pub struct TestRepository {
    pub users: Vec<UserWithRoles>,
    pub roles: Vec<Role>,
    pub menus: Vec<Menu>,
    pub hubs: Vec<Hub>,
}

impl TestRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_users(users: Vec<UserWithRoles>) -> Self {
        Self {
            users,
            ..Default::default()
        }
    }

    pub fn with_roles(mut self, roles: Vec<Role>) -> Self {
        self.roles = roles;
        self
    }

    pub fn with_menus(mut self, menus: Vec<Menu>) -> Self {
        self.menus = menus;
        self
    }

    pub fn with_hubs(mut self, hubs: Vec<Hub>) -> Self {
        self.hubs = hubs;
        self
    }

    pub fn now() -> NaiveDateTime {
        Utc::now().naive_utc()
    }

    /// Helper to create a `UserWithRoles` with provided roles by name.
    pub fn make_user(id: i32, email: &str, hub_id: i32, roles: Vec<&str>) -> UserWithRoles {
        let now = Self::now();
        let user = User {
            id,
            email: email.to_string(),
            name: Some(format!("User{id}")),
            hub_id,
            password_hash: "hash".into(),
            created_at: now,
            updated_at: now,
            roles: vec![],
        };
        let roles_vec: Vec<Role> = roles
            .into_iter()
            .enumerate()
            .map(|(i, name)| Role {
                id: i as i32 + 1,
                name: name.to_string(),
                created_at: now,
                updated_at: now,
            })
            .collect();
        UserWithRoles {
            user,
            roles: roles_vec,
        }
    }
}

impl UserReader for TestRepository {
    fn get_user_by_id(&self, id: i32, hub_id: i32) -> RepositoryResult<Option<UserWithRoles>> {
        let found = self
            .users
            .iter()
            .find(|u| u.user.id == id && u.user.hub_id == hub_id);
        Ok(found.map(|u| UserWithRoles {
            user: u.user.clone(),
            roles: u.roles.clone(),
        }))
    }

    fn get_user_by_email(
        &self,
        email: &str,
        hub_id: i32,
    ) -> RepositoryResult<Option<UserWithRoles>> {
        let found = self
            .users
            .iter()
            .find(|u| u.user.email == email && u.user.hub_id == hub_id);
        Ok(found.map(|u| UserWithRoles {
            user: u.user.clone(),
            roles: u.roles.clone(),
        }))
    }

    fn list_users(&self, query: UserListQuery) -> RepositoryResult<(usize, Vec<UserWithRoles>)> {
        let mut filtered: Vec<UserWithRoles> = self
            .users
            .iter()
            .filter(|u| u.user.hub_id == query.hub_id)
            .map(|u| UserWithRoles {
                user: u.user.clone(),
                roles: u.roles.clone(),
            })
            .collect();
        if let Some(role) = query.role {
            filtered.retain(|u| u.roles.iter().any(|r| r.name == role));
        }
        let total = filtered.len();
        if let Some(p) = query.pagination {
            let start = (p.page.max(1) - 1) * p.per_page;
            let end = (start + p.per_page).min(filtered.len());
            filtered = if start < filtered.len() {
                filtered[start..end]
                    .iter()
                    .map(|u| UserWithRoles {
                        user: u.user.clone(),
                        roles: u.roles.clone(),
                    })
                    .collect()
            } else {
                vec![]
            };
        }
        Ok((total, filtered))
    }

    fn verify_password(&self, password: &str, _stored_hash: &str) -> bool {
        // Keep it simple for tests
        password == "pass"
    }

    fn get_roles(&self, user_id: i32) -> RepositoryResult<Vec<Role>> {
        let roles = self
            .users
            .iter()
            .find(|u| u.user.id == user_id)
            .map(|u| u.roles.clone())
            .unwrap_or_default();
        Ok(roles)
    }

    fn search_users(&self, query: UserListQuery) -> RepositoryResult<(usize, Vec<UserWithRoles>)> {
        let mut filtered: Vec<UserWithRoles> = self
            .users
            .iter()
            .filter(|u| u.user.hub_id == query.hub_id)
            .map(|u| UserWithRoles {
                user: u.user.clone(),
                roles: u.roles.clone(),
            })
            .collect();
        if let Some(role) = query.role {
            filtered.retain(|u| u.roles.iter().any(|r| r.name == role));
        }
        if let Some(search) = query.search {
            let s = search.to_lowercase();
            filtered.retain(|u| {
                u.user.email.to_lowercase().contains(&s)
                    || u.user
                        .name
                        .clone()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&s)
            });
        }
        let total = filtered.len();
        if let Some(p) = query.pagination {
            let start = (p.page.max(1) - 1) * p.per_page;
            let end = (start + p.per_page).min(filtered.len());
            filtered = if start < filtered.len() {
                filtered[start..end]
                    .iter()
                    .map(|u| UserWithRoles {
                        user: u.user.clone(),
                        roles: u.roles.clone(),
                    })
                    .collect()
            } else {
                vec![]
            };
        }
        Ok((total, filtered))
    }
}

impl UserWriter for TestRepository {
    fn create_user(&self, new_user: &NewUser) -> RepositoryResult<User> {
        let now = Self::now();
        Ok(User {
            id: 1,
            email: new_user.email.clone(),
            name: new_user.name.clone(),
            hub_id: new_user.hub_id,
            password_hash: "hash".into(),
            created_at: now,
            updated_at: now,
            roles: vec![],
        })
    }

    fn assign_roles_to_user(&self, _user_id: i32, role_ids: &[i32]) -> RepositoryResult<usize> {
        Ok(role_ids.len())
    }

    fn update_user(
        &self,
        user_id: i32,
        hub_id: i32,
        updates: &UpdateUser,
    ) -> RepositoryResult<User> {
        let now = Self::now();
        Ok(User {
            id: user_id,
            email: format!("user{user_id}@hub{hub_id}"),
            name: Some(updates.name.clone()),
            hub_id,
            password_hash: if updates.password.is_some() {
                "new_hash".into()
            } else {
                "hash".into()
            },
            created_at: now,
            updated_at: now,
            roles: updates.roles.clone().unwrap_or_default(),
        })
    }

    fn delete_user(&self, _user_id: i32) -> RepositoryResult<usize> {
        Ok(1)
    }
}

impl RoleReader for TestRepository {
    fn get_role_by_id(&self, id: i32) -> RepositoryResult<Option<Role>> {
        Ok(self.roles.iter().find(|r| r.id == id).cloned())
    }

    fn get_role_by_name(&self, name: &str) -> RepositoryResult<Option<Role>> {
        Ok(self.roles.iter().find(|r| r.name == name).cloned())
    }

    fn list_roles(&self) -> RepositoryResult<Vec<Role>> {
        Ok(self.roles.clone())
    }
}

impl RoleWriter for TestRepository {
    fn create_role(&self, new_role: &NewRole) -> RepositoryResult<Role> {
        let now = Self::now();
        Ok(Role {
            id: 1,
            name: new_role.name.clone(),
            created_at: now,
            updated_at: now,
        })
    }

    fn delete_role(&self, role_id: i32) -> RepositoryResult<usize> {
        if self.roles.iter().any(|r| r.id == role_id) {
            Ok(1)
        } else {
            Err(RepositoryError::NotFound)
        }
    }
}

impl MenuReader for TestRepository {
    fn get_menu_by_id(&self, id: i32, hub_id: i32) -> RepositoryResult<Option<Menu>> {
        Ok(self
            .menus
            .iter()
            .find(|m| m.id == id && m.hub_id == hub_id)
            .cloned())
    }

    fn list_menu(&self, hub_id: i32) -> RepositoryResult<Vec<Menu>> {
        Ok(self
            .menus
            .iter()
            .filter(|m| m.hub_id == hub_id)
            .cloned()
            .collect())
    }
}

impl MenuWriter for TestRepository {
    fn create_menu(&self, new_menu: &NewMenu) -> RepositoryResult<Menu> {
        Ok(Menu {
            id: 1,
            name: new_menu.name.clone(),
            url: new_menu.url.clone(),
            hub_id: new_menu.hub_id,
        })
    }

    fn delete_menu(&self, menu_id: i32) -> RepositoryResult<usize> {
        if self.menus.iter().any(|m| m.id == menu_id) {
            Ok(1)
        } else {
            Err(RepositoryError::NotFound)
        }
    }
}

impl HubReader for TestRepository {
    fn get_hub_by_id(&self, id: i32) -> RepositoryResult<Option<Hub>> {
        Ok(self.hubs.iter().find(|h| h.id == id).cloned())
    }

    fn get_hub_by_name(&self, name: &str) -> RepositoryResult<Option<Hub>> {
        Ok(self.hubs.iter().find(|h| h.name == name).cloned())
    }

    fn list_hubs(&self) -> RepositoryResult<Vec<Hub>> {
        Ok(self.hubs.clone())
    }
}

impl HubWriter for TestRepository {
    fn create_hub(&self, new_hub: &NewHub) -> RepositoryResult<Hub> {
        let now = Self::now();
        Ok(Hub {
            id: 1,
            name: new_hub.name.clone(),
            created_at: now,
            updated_at: now,
        })
    }

    fn delete_hub(&self, hub_id: i32) -> RepositoryResult<usize> {
        if self.hubs.iter().any(|h| h.id == hub_id) {
            Ok(1)
        } else {
            Err(RepositoryError::NotFound)
        }
    }
}
