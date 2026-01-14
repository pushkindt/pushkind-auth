-- Your SQL goes here
CREATE INDEX idx_users_hub_id_id ON users(hub_id, id);
CREATE INDEX idx_user_roles_role_id_user_id ON user_roles(role_id, user_id);
CREATE INDEX idx_menu_hub_id ON menu(hub_id);
