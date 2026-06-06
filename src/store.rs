use std::{alloc::System, collections::HashMap, sync::Arc, time::SystemTime};
use tokio::{sync::RwLock, time::Instant};

pub type Store = Arc<RwLock<HashMap<Vec<u8>, RedisData>>>;

pub struct RedisData {
    pub value: Vec<u8>,
    pub expiry: Option<Instant>,
}
