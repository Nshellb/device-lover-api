use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::SoftwareVersionRow;

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminSoftwareVersion {
    pub id: Uuid,
    pub category: String,
    pub value: String,
    pub label: String,
    pub sort_order: i32,
}

impl From<SoftwareVersionRow> for AdminSoftwareVersion {
    fn from(row: SoftwareVersionRow) -> Self {
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
pub struct SoftwareVersionInput {
    pub category: String,
    pub value: String,
    pub label: String,
    pub sort_order: i32,
}
