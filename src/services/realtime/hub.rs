use serde::Serialize;
use serde_json::Value;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
    constants::phase_four::{REALTIME_HUB_CHANNEL_CAPACITY, REALTIME_MAX_EVENT_BYTES},
    errors::app_error::AppError,
};

#[derive(Clone, Debug, Serialize)]
pub struct RealtimeEvent {
    pub event_id: Uuid,
    pub event_type: String,
    pub account_id: Uuid,
    pub conversation_id: Option<Uuid>,
    pub payload: Value,
}

#[derive(Clone)]
pub struct RealtimeHub {
    sender: broadcast::Sender<RealtimeEvent>,
}

impl RealtimeHub {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(REALTIME_HUB_CHANNEL_CAPACITY);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RealtimeEvent> {
        self.sender.subscribe()
    }

    pub fn publish(
        &self,
        account_id: Uuid,
        event_type: &str,
        conversation_id: Option<Uuid>,
        payload: &Value,
    ) -> Result<RealtimeEvent, AppError> {
        let serialized = serde_json::to_vec(payload).map_err(|_| AppError::Internal)?;
        if serialized.len() > REALTIME_MAX_EVENT_BYTES {
            return Err(AppError::Internal);
        }
        let event = RealtimeEvent {
            event_id: Uuid::new_v4(),
            event_type: event_type.to_owned(),
            account_id,
            conversation_id,
            payload: payload.clone(),
        };
        // A receiver that has no subscribers is expected during startup; ignore.
        let _ = self.sender.send(event.clone());
        Ok(event)
    }
}

impl Default for RealtimeHub {
    fn default() -> Self {
        Self::new()
    }
}
