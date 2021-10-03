#[cfg(feature = "rotate")]
#[test]
fn test_rotation_event_parsing() {
    let test_data = include_str!("./rotate.json");
    let _event: lambda_runtime_types::TestData<lambda_runtime_types::rotate::Event<()>> =
        serde_json::from_str(test_data).expect("Unable to parse test data");
}
