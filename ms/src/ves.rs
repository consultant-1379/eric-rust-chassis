use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct VesException {
    pub message_id: String,
    pub text: String,
}

/// TODO: all "exception" according to ONAP
/// `<https://docs.onap.org/projects/onap-vnfrqts-requirements/en/latest/Chapter8/ves7_1spec.html#service-exceptions>`
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum VesError {
    ServiceException(VesException),
    PolicyException(VesException),
}

#[allow(dead_code)]
pub fn message_size_exceeded() -> VesError {
    VesError::PolicyException(
        VesException{
            message_id: "POL9003".to_string(),
            text: "Message content size exceeds the allowable limit".to_string(),
        }
    )
}

pub fn invalid_api_version() -> VesError {
    VesError::ServiceException(
        VesException{
            message_id: "SVC0002".to_string(),
            text: "Bad parameter (Invalid API version)".to_string(),
        }
    )
}

pub fn failed_schema_validation() -> VesError {
    VesError::ServiceException(
        VesException{
            message_id: "SVC0002".to_string(),
            text: "Bad parameter (JSON does not conform to schema)".to_string(),
        }
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use super::*;

    fn to_json_string(error: VesError) -> String {
        serde_json::to_string(&error).unwrap()
    }

    #[test]
    fn test_message_size_exceeded() {
        assert_eq!(
            to_json_string(message_size_exceeded()),
            json!(
            {
                "policyException": {
                    "messageId": "POL9003",
                    "text": "Message content size exceeds the allowable limit"
                }
            }
            ).to_string()
        );
    }

    #[test]
    fn test_invalid_api_version() {
        assert_eq!(
            to_json_string(invalid_api_version()),
            json!(
            {
                "serviceException": {
                    "messageId": "SVC0002",
                    "text": "Bad parameter (Invalid API version)"
                }
            }
            ).to_string()
        );
    }
}
