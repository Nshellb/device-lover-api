use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::BrandRow;

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminBrand {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
}

impl From<BrandRow> for AdminBrand {
    fn from(row: BrandRow) -> Self {
        Self {
            id: row.id,
            slug: row.slug,
            name: row.name,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BrandInput {
    pub slug: String,
    pub name: String,
}
