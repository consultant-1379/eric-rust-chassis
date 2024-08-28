use std::{
    collections::HashMap,
    sync::{
        Arc,
        Mutex
    },
    str
};

use actix_web::{
    get,
    post,
    web,
    App,
    HttpRequest,
    HttpResponse,
    HttpServer,
    Responder,
    http::header::{
        AsHeaderName,
        ContentType,
        HeaderMap,
        ToStrError
    }
};

use jsonschema::JSONSchema;
use log::info;
use serde_json::json;

use ers::{
    kafka::ProducerRecord,
    SemVer
};

use crate::{
    types::AppState,
    ves::{
        failed_schema_validation, invalid_api_version, VesError
    },
};

const HEADER_MINOR_VERSION: &str = "X-MinorVersion";
const HEADER_PATCH_VERSION: &str = "X-PatchVersion";
const HEADER_LATEST_VERSION: &str = "X-LatestVersion";
const HEADER_CONTENT_TYPE: &str = "Content-Type";
const CONTENT_TYPE_JSON: &str = "application/json";

#[actix_web::main]
pub async fn start(port: String, app_state: Arc<Mutex<AppState>>) {
    let addr = "0.0.0.0:".to_owned() + &port;
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(health)
            .service(process_event)
            .service(process_event_batch)
    })
    .workers(1) // TODO(cfg): read from configuration
    .bind(&addr)
    .unwrap()
    .run();

    info!("Server is live at {}", addr);
    server.await.unwrap();
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body(String::from("Healthy\n"))
}

#[post("/eventListener/{version}")]
async fn process_event(
    req: HttpRequest,
    path: web::Path<String>,
    data: web::Bytes,
    context: web::Data<Arc<Mutex<AppState>>>,
) -> impl Responder {
    return process_event_common(req, path, data, context, "event");
}

#[post("/eventListener/{version}/eventBatch")]
async fn process_event_batch(
    req: HttpRequest,
    path: web::Path<String>,
    data: web::Bytes,
    context: web::Data<Arc<Mutex<AppState>>>,
) -> impl Responder {
    return process_event_common(req, path, data, context, "eventList");
}

/// HTTP request handler for processing VES events.
///
/// ## Example HTTP request
///
/// ```sh
/// curl localhost:8080/eventListener/v7 -H "Content-Type: application/json" -H "X-MinorVersion: 2" -d@ms/res/ves-7.2.1-domain_pnfRegistration.json
/// ```
///
pub fn process_event_common(
    req: HttpRequest,
    path: web::Path<String>,
    data: web::Bytes,
    context: web::Data<Arc<Mutex<AppState>>>,
    event_mode: &str,
) -> impl Responder {
    let payload = data.into_iter().collect();
    let version = path.into_inner();

    let result = validate_request(
        &version.as_str(),
        req.headers(),
        &payload,
        event_mode,
        &context.get_ref().lock().unwrap().schemas
    );

    if result.is_err() {
        return HttpResponse::BadRequest()
            .content_type(ContentType::json())
            .body(
                serde_json::to_string(
                    &json!({ "requestError": result.err().unwrap() })
                ).unwrap()
            );
    }

    for event in parse_events(&payload) {
        let msg = ProducerRecord {
            key: "test".into(),
            topic: context.get_ref().lock().unwrap().topic.to_owned(),
            headers: HashMap::new(),
            payload: event,
        };

        let _ = context
            .get_ref()
            .lock()
            .unwrap()
            .tx.send(msg)
                .map_err(|err| {
                    log::warn!("Failed to queue message: {}", err);
                    err
                });
    }

    HttpResponse::Accepted()
        .insert_header((HEADER_MINOR_VERSION, 1))           // TODO(feat): implement the correct response
        .insert_header((HEADER_PATCH_VERSION, 1))
        .insert_header((HEADER_LATEST_VERSION, "7.1.1"))
        .body("")
}

/// Parse events out of the JSON.
///
/// Either a single event in `event` key or multiple events in `eventList` key.
///
/// ## Examples
///
/// ### Single event
///
/// ```
/// # use serde_json::json;
/// # use ves::http_server::parse_events;
/// let event = json!({
///     "event": {
///         "a": 1
///     }
/// });
///
/// let expected = vec![
///     r#"{"a":1}"#
/// ];
/// # let input = event.to_string().as_bytes().to_vec();
/// # let result: Vec<_> = expected.into_iter().map(|x| x.as_bytes().to_vec()).collect();
///
/// # assert_eq!(parse_events(&input), result);
/// ```
///
/// ### Event batch
///
/// ```
/// # use serde_json::json;
/// # use ves::http_server::parse_events;
/// let event_list = json!({
///     "eventList": [
///         { "a": 1 },
///         { "b": 2 }
///     ]
/// });
///
/// let expected = vec![
///     r#"{"a":1}"#,
///     r#"{"b":2}"#
/// ];
/// # let input = event_list.to_string().as_bytes().to_vec();
/// # let result: Vec<_> = expected.into_iter().map(|x| x.as_bytes().to_vec()).collect();
///
/// # assert_eq!(parse_events(&input), result);
/// ```
///
pub fn parse_events(data: &Vec<u8>) -> Vec<Vec<u8>> {        // TODO(perf): slice?
    str::from_utf8(&data).map(
        |string_data| serde_json::from_str::<serde_json::Value>(string_data).map(
            |json_data| match json_data["event"] {
                serde_json::Value::Null => {
                    json_data["eventList"]
                        .as_array()
                        .expect("")
                        .iter()
                        .map(|event| { event.to_string().as_bytes().to_vec() })
                        .collect()
                }
                _ => vec![json_data["event"].to_string().as_bytes().to_vec()]
            }
        )
    ).expect("Logic error in validation").expect("Logic error again")
}

/// Validate event based on JSON schema with some pre-checks.
///
/// The `data` is okay if it is:
/// - a valid string, so no binary data
/// - and a valid JSON,
/// - and it has "event" or "eventList" for root key
/// - and also valid according to the schema
///
pub fn validate_event(data: &Vec<u8>, schema: &JSONSchema, root_key: &str) -> bool {
    str::from_utf8(&data).is_ok_and(
        |string_data| serde_json::from_str::<serde_json::Value>(string_data).is_ok_and(
            |json_data| match json_data[root_key] {
                serde_json::Value::Null => false,
                _ => schema.validate(&json_data).is_ok()
            }
        )
    )
}

trait DefaultStringGetter {
    fn get_str_or<'a>(&'a self, key: impl AsHeaderName, def: &'a str) -> Result<&'a str, ToStrError>;
}

impl DefaultStringGetter for HeaderMap {
    fn get_str_or<'a>(&'a self, key: impl AsHeaderName, def: &'a str) -> Result<&'a str, ToStrError> {
        self.get(key)
            .map(|x| x.to_str())
            .unwrap_or_else(
                || Ok(def)
            )
    }
}

/// Validate everything in the incoming request.
///
/// The request is valid if all of the followings are true (checked in order):
/// - Content-Type is `application/json` (TODO)
/// - Message size is below 2 MB (TODO)
/// - `version` path is for example `v7`, where the number matches one of the supported schemas
/// - The given minor and patch version in the HTTP headers are supported;
///   default is 0 and 1 for minor and patch respectively (full example: v7.0.1)
/// - Finally, the event is valid according to [`validate_event`]
///
fn validate_request(
    version: &str,
    headers: &HeaderMap,
    data: &Vec<u8>,
    event_mode: &str,
    schemas: &HashMap<SemVer, JSONSchema>
)
    -> Result<(), VesError>
{
    if !version.starts_with("v") {
        return Err(invalid_api_version());
    }

    let Ok(major) = version[1..].parse() else {
        return Err(invalid_api_version());
    };

    let Ok(Ok(minor)) = headers.get_str_or(HEADER_MINOR_VERSION, "0").map(|x| x.parse::<u16>()) else {
        return Err(invalid_api_version());
    };

    let Ok(Ok(patch)) = headers.get_str_or(HEADER_PATCH_VERSION, "1") .map(|x| x.parse::<u16>()) else {
        return Err(invalid_api_version());
    };

    if patch > 1 {
        if schemas.keys().filter(|x| **x == SemVer{ major, minor, patch }).count() == 0 {
            return Err(invalid_api_version());
        }
    } else {
        let patch_versions = schemas
            .keys()
            .filter(|supported_version| supported_version.major == major && supported_version.minor == minor)
            .map(|supported_version| supported_version.patch)
            .collect::<Vec<u16>>();

        if patch_versions.len() == 0 {
            return Err(invalid_api_version());
        }

        let latest_patch = patch_versions.into_iter().max().expect("Logic error!");
        let Some(schema) = schemas.get(&SemVer{ major, minor, patch: latest_patch }) else {
            return Err(invalid_api_version());
        };

        if !validate_event(&data, schema, event_mode) {
            return Err(failed_schema_validation());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use actix_web::http::header::{HeaderName, HeaderValue};

    use super::*;

    fn schema() -> JSONSchema {
        let x = serde_json::from_str(include_str!("../res/schemas/CommonEventFormat_30.2.1.json")).expect("Invalid JSON");
        JSONSchema::compile(&x).expect("Invalid JSON schema")
    }

    fn schema_map() -> HashMap<SemVer, JSONSchema> {
        let mut schemas = HashMap::new();
        schemas.insert(SemVer::from_str("7.2.1").unwrap(), schema());
        schemas
    }

    #[test]
    fn test_validate_invalid_utf8() {
        let input = vec![0x01, 0xFF];
        assert!(!validate_event(&input, &schema(), "event"));
    }

    #[test]
    fn test_validate_invalid_json() {
        let input = "{".into();
        assert!(!validate_event(&input, &schema(), "event"));
    }

    #[test]
    fn test_validate_invalid_event_ves_schema() {
        let input = r#"{"event": 1}"#.into();
        assert!(!validate_event(&input, &schema(), "event"));
    }

    #[test]
    fn test_validate_invalid_event_root_key_check() {
        let input = r#"{"bla": 1}"#.into();
        assert!(!validate_event(&input, &schema(), "event"));
    }

    #[test]
    fn test_validate_valid_event() {
        let input = include_str!("../res/ves-7.2.1-domain_pnfRegistration.json").into();
        assert!(validate_event(&input, &schema(), "event"));
    }

    #[test]
    fn test_valid_request() {
        let mut headers = HeaderMap::new();
        headers.insert(HeaderName::from_str(HEADER_CONTENT_TYPE).unwrap(), HeaderValue::from_str(CONTENT_TYPE_JSON).unwrap());
        headers.insert(HeaderName::from_str(HEADER_MINOR_VERSION).unwrap(), HeaderValue::from_str("2").unwrap());

        let data = include_str!("../res/ves-7.2.1-domain_pnfRegistration.json").to_string().into_bytes();

        assert!(validate_request("v7", &headers, &data, "event", &schema_map()).is_ok());
    }

    #[test]
    fn test_invalid_request_version() {
        let headers = HeaderMap::new();
        let data = Vec::new();
        assert!(validate_request("7", &headers, &data, "event", &schema_map()).is_err());
    }
}
