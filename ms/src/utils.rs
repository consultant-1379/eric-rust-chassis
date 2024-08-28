//! Utility functions

/**
 * An example function to illustrate testing
 *
 * Example usage:
 *
 * ```
 * # use ers::utils::welcome;
 * println!("{}", welcome());
 *
 * # assert_eq!(welcome(), "Hello, Ericsson!");
 * ```
 *
 * *It also uses documentation testing*.
 */
pub fn welcome() -> String {
    String::from("Hello, Ericsson!")
}

/**
 * An example function illustrating the usage of a 3PP
 */
pub fn serde() -> serde_json::Value {
    let data = r#"
    {
        "host": "http://something.com",
        "port": 8888
    }
    "#;

    serde_json::from_str(data).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_welcome() {
        assert_eq!(welcome(), "Hello, Ericsson!");
    }

    #[test]
    fn test_serde() {
        let json = serde();
        assert_eq!(json["host"], String::from("http://something.com"));
        assert_eq!(json["port"], 8888);
    }
}
