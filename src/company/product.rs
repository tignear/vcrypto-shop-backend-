use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize,Debug)]
pub(crate) struct Role {
    pub(crate) unit: String,
    pub(crate) price: String,
    pub(crate) role_id: String,
}

#[derive(Serialize, Deserialize,Debug)]
pub(crate) enum Product {
    Role(Role),
}
