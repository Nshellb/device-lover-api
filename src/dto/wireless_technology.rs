use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::WirelessTechnologyRow;

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminWirelessTechnology {
    pub id: Uuid,
    pub category: String,
    pub value: String,
    pub label: String,
    pub sort_order: i32,
}

impl From<WirelessTechnologyRow> for AdminWirelessTechnology {
    fn from(row: WirelessTechnologyRow) -> Self {
        Self {
            id: row.id,
            category: row.category,
            value: row.value,
            label: row.label,
            sort_order: row.sort_order,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WirelessTechnologyInput {
    pub category: String,
    pub value: String,
    pub label: String,
    pub sort_order: i32,
}
