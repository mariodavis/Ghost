use crate::types::EncryptedPayload;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

/// ChatRoom encapsulates a single room's broadcast channel.
/// All messages published to this room's sender will be delivered
/// to all active subscribers without disk or memory buffering.
#[derive(Clone)]
pub struct ChatRoom {
    /// Broadcast channel sender with a fixed capacity of 100 messages.
    /// Once the capacity is exceeded, the oldest message is dropped.
    pub tx: broadcast::Sender<EncryptedPayload>,
}

impl ChatRoom {
    /// Create a new ChatRoom with a broadcast channel of capacity 100.
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self { tx }
    }
}

impl Default for ChatRoom {
    fn default() -> Self {
        Self::new()
    }
}

/// AppState is the global application state.
/// It is an Arc-wrapped DashMap providing lock-free, sharded concurrent access
/// to ChatRoom instances keyed by room_code without global Mutex bottlenecks.
pub type AppState = Arc<DashMap<String, ChatRoom>>;

/// Initialize a new AppState instance.
pub fn create_app_state() -> AppState {
    Arc::new(DashMap::new())
}

/// Retrieve or create a ChatRoom for the given room_code.
/// Returns a cloned ChatRoom (which contains a cloned Sender reference).
pub fn get_or_create_room(state: &AppState, room_code: String) -> ChatRoom {
    state
        .entry(room_code)
        .or_insert_with(ChatRoom::new)
        .clone()
}
