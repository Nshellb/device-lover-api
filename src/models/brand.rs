use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct BrandRow {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
}
