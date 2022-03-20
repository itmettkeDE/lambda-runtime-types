#[derive(Debug, serde::Deserialize)]
struct Event {
    timeout_secs: Option<u64>,
}

struct Runner;

#[async_trait::async_trait]
impl<'a> lambda_runtime_types::Runner<'a, (), Event, ()> for Runner {
    async fn run(
        _shared: &'a (),
        event: lambda_runtime_types::LambdaEvent<'a, Event>,
    ) -> anyhow::Result<()> {
        let timeout_secs = event.event.timeout_secs.unwrap_or(60);
        let timeout = tokio::time::Duration::from_secs(timeout_secs);
        tokio::time::sleep(timeout).await;
        Ok(())
    }

    async fn setup(_region: &'a str) -> anyhow::Result<()> {
        simple_logger::SimpleLogger::new()
            .with_level(log::LevelFilter::Info)
            .init()
            .expect("Unable to setup logging");
        Ok(())
    }
}

pub fn main() -> anyhow::Result<()> {
    lambda_runtime_types::exec_tokio::<_, _, Runner, _>()
}
