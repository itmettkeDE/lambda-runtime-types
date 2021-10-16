#[derive(Default)]
struct Shared {
    invocations: tokio::sync::Mutex<u64>,
}

struct Runner;

#[async_trait::async_trait]
impl lambda_runtime_types::Runner<Shared, (), u64> for Runner {
    async fn run<'a>(shared: &'a Shared, _event: (), _region: &'a str) -> anyhow::Result<u64> {
        let mut invocations = shared.invocations.lock().await;
        *invocations += 1;
        Ok(*invocations)
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
