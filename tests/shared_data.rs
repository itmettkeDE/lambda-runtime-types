#[derive(serde::Deserialize, Debug)]
struct Event {
    #[serde(flatten)]
    attributes: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Default)]
struct Shared {
    prev_value: tokio::sync::Mutex<Option<String>>,
}

#[derive(serde::Serialize, Debug)]
struct Return {
    matches_prev: bool,
}

struct Runner;

#[async_trait::async_trait]
impl lambda_runtime_types::Runner<Shared, Event, Return> for Runner {
    async fn run<'a>(shared: &'a Shared, event: Event, _region: &'a str) -> anyhow::Result<Return> {
        log::info!("{:?}", event);
        let mut prev_value = shared.prev_value.lock().await;
        let this_value = event
            .attributes
            .get("test")
            .and_then(|a| a.as_str())
            .map(ToOwned::to_owned)
            .map(Into::into);
        let matches_prev = this_value == *prev_value;
        *prev_value = this_value;
        Ok(Return { matches_prev })
    }

    async fn setup() -> anyhow::Result<()> {
        simple_logger::SimpleLogger::new()
            .with_level(log::LevelFilter::Info)
            .init()
            .expect("Unable to setup logging");
        Ok(())
    }
}

#[test]
fn test_shared_data_lambda() {
    let test_data = include_str!("./shared_data.json");
    lambda_runtime_types::exec_test::<_, _, Runner, _>(test_data)
        .expect("Unable to execute lambda");
}
