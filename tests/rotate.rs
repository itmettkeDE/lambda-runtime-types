#[cfg(feature = "rotate")]
#[test]
fn test_rotation_event_parsing() {
    let test_data = include_str!("./rotate.json");
    let _event: lambda_runtime_types::TestData<lambda_runtime_types::rotate::Event<()>> =
        serde_json::from_str(test_data).expect("Unable to parse test data");
}

#[cfg(feature = "rotate")]
#[test]
fn test_rotation_secret_parsing_struct_exact() {
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct TestData {
        test: String,
    }
    let json = serde_json::json!({
        "test": "test_data",
    });
    let expected_str = serde_json::to_string(&json).expect("Unable to serialize json test data");
    let secret: lambda_runtime_types::rotate::SecretContainer<TestData> =
        serde_json::from_value(json).expect("Unable to deserialize to structure");
    let result_str = serde_json::to_string(&secret).expect("Unable to serialize secret");
    assert!(secret.o.is_empty());
    assert_eq!(expected_str, result_str);
}

#[cfg(feature = "rotate")]
#[test]
fn test_rotation_secret_parsing_struct_additional() {
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct TestData {
        test: String,
    }
    let json = serde_json::json!({
        "test": "test_data",
        "test2": "test_data2",
    });
    let expected_str = serde_json::to_string(&json).expect("Unable to serialize json test data");
    let secret: lambda_runtime_types::rotate::SecretContainer<TestData> =
        serde_json::from_value(json).expect("Unable to deserialize to structure");
    let result_str = serde_json::to_string(&secret).expect("Unable to serialize secret");
    assert_eq!(secret.o.len(), 1);
    assert_eq!(expected_str, result_str);
}

#[cfg(feature = "rotate")]
#[test]
fn test_rotation_secret_parsing_empty() {
    let json = serde_json::json!({
        "test": "test_data",
        "test2": "test_data2",
    });
    let expected_str = serde_json::to_string(&json).expect("Unable to serialize json test data");
    let secret: lambda_runtime_types::rotate::SecretContainer<()> =
        serde_json::from_value(json.clone()).expect("Unable to deserialize to structure");
    let result_json = serde_json::to_value(&secret).expect("Unable to serialize secret");
    assert_eq!(secret.o.len(), 2);
}
