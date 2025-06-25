// @generated automatically by Diesel CLI.

diesel::table! {
    hubs (id) {
        id -> Integer,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    menu (id) {
        id -> Integer,
        name -> Text,
        url -> Text,
        hub_id -> Integer,
    }
}

diesel::table! {
    roles (id) {
        id -> Integer,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    user_roles (user_id, role_id) {
        user_id -> Integer,
        role_id -> Integer,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        email -> Text,
        name -> Nullable<Text>,
        hub_id -> Integer,
        password_hash -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(menu -> hubs (hub_id));
diesel::joinable!(user_roles -> roles (role_id));
diesel::joinable!(user_roles -> users (user_id));
diesel::joinable!(users -> hubs (hub_id));

diesel::allow_tables_to_appear_in_same_query!(
    hubs,
    menu,
    roles,
    user_roles,
    users,
);
