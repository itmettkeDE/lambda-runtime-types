#[derive(Debug, serde::Deserialize)]
struct Event {
    timeout_secs: Option<u64>,
}

struct Runner;

#[async_trait::async_trait]
impl lambda_runtime_types::Runner<(), Event, ()> for Runner {
    async fn run<'a>(
        _shared: &'a (),
        event: Event,
        _region: &'a str,
        _ctx: lambda_runtime_types::Context,
    ) -> anyhow::Result<()> {
        let timeout_secs = event.timeout_secs.unwrap_or(60);
        let timeout = tokio::time::Duration::from_secs(timeout_secs);
        tokio::time::sleep(timeout).await;
        Ok(())
    }

    async fn setup() -> anyhow::Result<()> {
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
