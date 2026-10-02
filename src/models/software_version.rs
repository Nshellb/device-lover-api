use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct SoftwareVersionRow {
    pub id: Uuid,
    pub category: String,
    pub value: String,
    pub label: String,
    pub sort_order: i32,
}
