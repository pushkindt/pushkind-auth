use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// A navigation item available to users of a [`Hub`].
pub struct Menu {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub hub_id: i32,
}

#[derive(Clone, Debug, Deserialize)]
/// Parameters required to create a new [`Menu`].
pub struct NewMenu {
    pub name: String,
    pub url: String,
    pub hub_id: i32,
}
