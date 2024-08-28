use std::collections::HashMap;
use std::sync::mpsc::Sender;

use ers::SemVer;
use ers::kafka::ProducerRecord;

use jsonschema::JSONSchema;

// #[derive(Clone)]
pub struct AppState {
    pub topic: String,
    pub tx: Sender<ProducerRecord>,
    pub schemas: HashMap<SemVer, JSONSchema>,
}
