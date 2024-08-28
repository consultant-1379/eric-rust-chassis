use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::error::KafkaResult;
use rdkafka::message::{Headers, Message};
use rdkafka::topic_partition_list::TopicPartitionList;

use crate::MyApp;

struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        log::debug!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        log::debug!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        log::debug!("Committing offsets: {:?}", result);
    }
}

// A type alias with your custom consumer can be created for convenience.
type LoggingConsumer = StreamConsumer<CustomContext>;

pub async fn consume_topic_1_and_print(my_app: MyApp) {
    let enable = my_app.get_param_bool("interfaces.southbound.kafka.input_1.enable");
    if enable == false {
        return;
    }

    let brokers = my_app.get_param("kafka.address");
    let topic = my_app.get_param("interfaces.southbound.kafka.input_1.topic");
    let group_id = my_app.get_param("interfaces.southbound.kafka.input_1.group_id");
    let enable_auto_commit = my_app.get_param("interfaces.southbound.kafka.input_1.enable_auto_commit");

    let context = CustomContext;

    let consumer: LoggingConsumer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", &group_id)
        .set("enable.auto.commit", enable_auto_commit)
        .set("enable.partition.eof", "false")
        .set_log_level(RDKafkaLogLevel::Info)
        .create_with_context(context)
        .expect("Consumer creation failed");

    consumer
        .subscribe(&[topic.as_str()])
        .expect("Can't subscribe to specified topics");
    log::info!(
        "Kafka consumer started for topic {} with group-id {}",
        topic,
        group_id
    );

    loop {
        match consumer.recv().await {
            Err(e) => log::error!("Kafka error: {}", e),
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        log::info!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                log::info!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                      m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                if let Some(headers) = m.headers() {
                    for header in headers.iter() {
                        log::info!("  Header {:#?}: {:?}", header.key, header.value);
                    }
                }
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}
