use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Menu {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub hub_id: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewMenu {
    pub name: String,
    pub url: String,
    pub hub_id: i32,
}
