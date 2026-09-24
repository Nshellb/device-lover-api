use std::sync::Arc;
use std::time::Instant;

use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::config::Settings;
use crate::dto::PopularDevicesResponse;

pub(crate) type PopularDevicesCache = Arc<RwLock<Option<CachedPopularDevices>>>;

pub(crate) struct CachedPopularDevices {
    pub expires_at: Instant,
    pub response: PopularDevicesResponse,
}

#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<Settings>,
    pub db: PgPool,
    pub(crate) popular_devices_cache: PopularDevicesCache,
}

impl AppState {
    pub fn new(settings: Settings, db: PgPool) -> Self {
        Self {
            settings: Arc::new(settings),
            db,
            popular_devices_cache: Arc::new(RwLock::new(None)),
        }
    }
}
