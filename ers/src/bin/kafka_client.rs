use clap::Parser;
use ers::kafka;

use rand::{distributions, Rng};
use std::{collections::HashMap, thread::sleep, time::Duration};

fn generate_message(length: usize) -> String {
    let rng = rand::thread_rng();

    rng.sample_iter(&distributions::Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

fn send(producer: &kafka::KafkaProducer, payload: &String) {
    let message = kafka::ProducerRecord {
        key: "test".into(),
        topic: String::from("dev-test"),
        headers: HashMap::new(),
        payload: payload.as_bytes().into(),
    };

    match producer.send(&message) {
        Ok(_) => (),
        Err((e, _)) => log::error!("{}", e),
    }
}

#[derive(clap::Parser)]
struct Args {
    /// Number of messages to send to Kafka broker
    #[arg(short, long, default_value_t = 1)]
    num_records: u64,

    /// The address of the Kafka broker
    #[arg(short, long, default_value_t = String::from("localhost:9092"))]
    broker: String,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    let mut cfg = HashMap::new();
    cfg.insert("bootstrap.servers".to_string(), args.broker);
    let producer = kafka::KafkaProducer::new(cfg);

    let message = generate_message(300);

    let time = std::time::Instant::now();
    for _ in 1..=args.num_records {
        send(&producer, &message);
    }

    log::info!("Elapsed time: {:?}", time.elapsed());

    // Producer's `send` is async function call, which is most likely slower than
    // this thread reaching the debug log statement
    sleep(Duration::new(0, 200_000_000));
    log::debug!("{:?}", producer.metrics());
}
