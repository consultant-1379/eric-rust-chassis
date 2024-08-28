use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use rdkafka::message::Header;
use rdkafka::config::ClientConfig;
use rdkafka::error::KafkaError;
use rdkafka::message::OwnedHeaders;
use rdkafka::producer::{BaseRecord, DeliveryResult, ProducerContext, ThreadedProducer};
use rdkafka::ClientContext;

/**
 * A Kafka record for sending messages
 *
 * Example:
 * ```rust,no_run
 *   let message = ers::kafka::ProducerRecord {
 *       key: "test".into(),
 *       topic: String::from("dev-test"),
 *       headers: std::collections::HashMap::new(),
 *       payload: b"payload".into()
 *   };
 * ```
 */
// TODO(feat): `ToBytes` trait?
pub struct ProducerRecord {
    pub key: Vec<u8>,
    pub topic: String,
    pub headers: HashMap<String, Vec<u8>>,
    pub payload: Vec<u8>,
}

/**
 * Thread-safe counters for outgoing data
 */
#[derive(Default, Debug)]
pub struct Metrics {
    /// The number of successfully sent messages
    pub sent_messages: AtomicU64,
    /// The number of successfully sent bytes (only the payload)
    pub sent_bytes: AtomicU64,
    /// The number of messages failed to send
    pub dropped_messages: AtomicU64,
    /// The number of bytes failed to send (only the payload)
    pub dropped_bytes: AtomicU64,
}

struct CustomProducerContext {
    metrics: Arc<Metrics>,
}

impl CustomProducerContext {
    fn new() -> Self {
        Self {
            metrics: Default::default(),
        }
    }
}

impl ClientContext for CustomProducerContext {}

/**
 * Increments the counters stored in `Metrics`
 */
impl ProducerContext for CustomProducerContext {
    type DeliveryOpaque = ();
    fn delivery(&self, delivery_result: &DeliveryResult, _: Self::DeliveryOpaque) {
        match delivery_result {
            Ok(msg) => {
                self.metrics.sent_messages.fetch_add(1, Ordering::Relaxed);
                self.metrics
                    .sent_bytes
                    .fetch_add(msg.payload_len() as u64, Ordering::Relaxed);
            }
            Err((err, msg)) => {
                self.metrics
                    .dropped_messages
                    .fetch_add(1, Ordering::Relaxed);
                self.metrics
                    .dropped_bytes
                    .fetch_add(msg.payload_len() as u64, Ordering::Relaxed);
                dbg!(err);
            }
        }
    }
}

pub struct KafkaProducer {
    producer: ThreadedProducer<CustomProducerContext>,
    metrics: Arc<Metrics>,
}

/**
 * A Kafka producer client
 *
 * Underlying library is `librdkafka`. A separate thread calls the `poll()` periodically.
 */
impl KafkaProducer {
    pub fn new(dict: HashMap<String, String>) -> Self {
        let mut cfg = ClientConfig::new();
        for (key, value) in &dict {
            cfg.set(key, value);
        }

        // TODO: customizable context
        let context = CustomProducerContext::new();
        Self {
            metrics: context.metrics.clone(),
            producer: cfg.create_with_context(context).unwrap(),
        }
    }

    #[allow(clippy::result_large_err)] // TODO(design): optimize return value / API
    pub fn send<'a>(
        &'a self,
        message: &'a ProducerRecord,
    ) -> Result<(), (KafkaError, BaseRecord<'_, Vec<u8>, [u8]>)> {
        let mut headers = OwnedHeaders::new_with_capacity(message.headers.len());
        for (key, value) in &message.headers {
            let header = Header{key: key.as_str(), value: Some(value)};
            headers = headers.insert(header);
        }

        return self.producer.send(
            BaseRecord::to(&message.topic)
                .key(&message.key)
                .payload(message.payload.as_slice())
                .headers(headers)
        );
    }

    pub fn metrics(&self) -> Arc<Metrics> {
        Arc::clone(&self.metrics)
    }
}
