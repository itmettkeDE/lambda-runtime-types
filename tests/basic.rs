#[derive(serde::Deserialize, Debug)]
struct Event {
    #[serde(flatten)]
    attributes: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(serde::Serialize, Debug)]
struct Return {
    data: std::borrow::Cow<'static, str>,
}

struct Runner;

#[async_trait::async_trait]
impl lambda_runtime_types::Runner<(), Event, Return> for Runner {
    async fn run<'a>(
        _shared: &'a (),
        event: Event,
        _region: &'a str,
        _ctx: lambda_runtime::Context,
    ) -> anyhow::Result<Return> {
        log::info!("{:?}", event);
        Ok(Return {
            data: event
                .attributes
                .get("test")
                .and_then(|a| a.as_str())
                .map(ToOwned::to_owned)
                .map(Into::into)
                .unwrap_or_else(|| "none".into()),
        })
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
fn test_basic_lambda() {
    let test_data = include_str!("./basic.json");
    lambda_runtime_types::exec_test::<_, _, Runner, _>(test_data)
        .expect("Unable to execute lambda");
}
