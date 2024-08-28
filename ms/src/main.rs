use std::{
    env,
    thread,
    thread::JoinHandle,
    collections::HashMap,
    sync::{
        Arc,
        Mutex,
        mpsc,
        mpsc::Receiver,
    },
};

use ers::{
    SemVer,
    kafka::{
        ProducerRecord,
        KafkaProducer,
    }
};

use config::Config;
use jsonschema::JSONSchema;

mod kafka_consumer;     // TODO: this depends on `MyApp`

use ves::{
    http_server,
    types::AppState
};

#[derive(Clone)]
struct MyApp {
    config: Config,
}

impl MyApp {
    fn new_with_prefix(prefix: &str) -> Self {
        let mut file_path = String::from("./settings");
        if let Ok(v) = env::var(prefix.to_owned() + "_CONFIG_FILE") {
            file_path = v.to_owned();
        }
        let config = Config::builder()
            .add_source(config::File::with_name(&file_path))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(config::Environment::with_prefix(prefix))
            .build()
            .expect("Error");

        MyApp { config }
    }

    fn get_param(&self, key: &str) -> String {
        self.config
            .get_string(key)
            .unwrap_or_else(|_| panic!("{} is missing from configuration", key))
    }

    fn get_param_bool(&self, key: &str) -> bool {
        self.config
            .get_bool(key)
            .unwrap_or_else(|_| panic!("{} is missing from configuration", key))
    }
}

fn read_schema(path: &String) -> JSONSchema {
    // std::fs::read_to_string(path).map_or("Invalid path",
        // |content| serde_json::from_str::<serde_json::Value>(content.as_str()).map_or("Invalid JSON",
            // |json| JSONSchema::compile(&json).or("Invalid JSON 2")
        // )
    // )

    JSONSchema::compile(
        &serde_json::from_str(
            std::fs::read_to_string(path)
                .expect(format!("Failed to read file: {}", path).as_str())
                .as_str()
        ).expect("Invalid Json schema")
    ).expect("Invalid Json schema")
}

fn read_schemas(config: &Config) -> HashMap<SemVer, JSONSchema> {
    let mut schemas = HashMap::new();

    for (version, filepath) in config.get_table("schemas.apiVersion").unwrap() {
        schemas.insert(
            version.parse().expect("Invalid SemVer, most likely an incorrect configuration file!"),
            read_schema(&filepath.into_string().unwrap())
        );
    }

    schemas
}

fn main() {
    env_logger::init();
    let my_app = MyApp::new_with_prefix("APP");

    let producer = create_kafka_producer(&my_app);
    let p = Arc::clone(&producer);
    let (tx, rx) = mpsc::channel();
    let topic = my_app.get_param("interfaces.northbound.kafka.output_1.topic");
    let app_state = Arc::new(Mutex::new(AppState{ topic, tx, schemas: read_schemas(&my_app.config) }));
    let http_thread = start_http_server(&my_app, Arc::clone(&app_state));

    let rt = tokio::runtime::Runtime::new().unwrap();

    if my_app.get_param_bool("interfaces.southbound.kafka.enable") {
        rt.spawn(kafka_consumer::consume_topic_1_and_print(my_app));
    }

    rt.spawn(start_kafka_producer(p, rx));

    let _ = http_thread.join();

    log::debug!("{:?}", producer.metrics());
}

fn start_http_server(my_app: &MyApp, app_state: Arc<Mutex<AppState>>) -> JoinHandle<()> {
    let port = my_app.get_param("interfaces.http.port");
    let _topic = my_app.get_param("interfaces.northbound.kafka.output_1.topic");
    thread::spawn(|| http_server::start(port, app_state))
}

fn create_kafka_producer(my_app: &MyApp) -> Arc<KafkaProducer> {
    let mut cfg = HashMap::new();
    cfg.insert(
        "bootstrap.servers".to_string(),
        my_app.get_param("kafka.address"),
    );
    Arc::new(KafkaProducer::new(cfg))
}

async fn start_kafka_producer(
    producer: Arc<KafkaProducer>,
    rx: Receiver<ProducerRecord>,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("ves:kf".into())
        .spawn(move || loop {
            match rx.recv() {
                Ok(msg) => {
                    match producer.send(&msg) {
                        Ok(_) => log::info!("Message sent to Kafka broker"),
                        Err((e, _)) => log::warn!("Failed to send message to Kafka broker: {}", e),
                    };
                }
                Err(_) => {
                    log::info!("Channel closed");
                    return;
                }
            };
        }).expect("Invalid thread name")
}
