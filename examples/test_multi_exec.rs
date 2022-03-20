#[derive(Default)]
struct Shared {
    invocations: tokio::sync::Mutex<u64>,
}

struct Runner;

#[async_trait::async_trait]
impl<'a> lambda_runtime_types::Runner<'a, Shared, (), u64> for Runner {
    async fn run(
        shared: &'a Shared,
        _event: lambda_runtime_types::LambdaEvent<'a, ()>,
    ) -> anyhow::Result<u64> {
        let mut invocations = shared.invocations.lock().await;
        *invocations += 1;
        Ok(*invocations)
    }

    async fn setup(_region: &'a str) -> anyhow::Result<Shared> {
        simple_logger::SimpleLogger::new()
            .with_level(log::LevelFilter::Info)
            .init()
            .expect("Unable to setup logging");
        Ok(Shared::default())
    }
}

pub fn main() -> anyhow::Result<()> {
    lambda_runtime_types::exec_tokio::<_, _, Runner, _>()
}
