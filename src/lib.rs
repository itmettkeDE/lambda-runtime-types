//! This crate provides types and traits to simplify
//! the creation of lambda functions in rust. It
//! provides Event and Return types and specific
//! Runners for various lambda types.
//!
//! # Basic Lambda with no shared data
//!
//! Creating a normal lambda is very easy. First create a type which implements [`Runner`] and
//! then use it either in the [`exec`] or [`exec_tokio`] function:
//!
//! ```no_run
//! struct Runner;
//!
//! #[async_trait::async_trait]
//! impl lambda_runtime_types::Runner<(), (), ()> for Runner {
//!     async fn run<'a>(shared: &'a (), event: (), region: &'a str) -> anyhow::Result<()> {
//!         // Run code on every invocation
//!         Ok(())
//!     }
//!
//!     async fn setup() -> anyhow::Result<()> {
//!         // Setup logging to make sure that errors are printed
//!         Ok(())
//!     }
//! }
//!
//! pub fn main() -> anyhow::Result<()> {
//!     lambda_runtime_types::exec_tokio::<_, _, Runner, _>()
//! }
//! ```
//!
//! # Available lambda types
//!
//! There are various modules which predefined Event and Return types and Runner traits
//! specialised for differnet lambda usages. Check out the modules for examples or their
//! usage.
//!
//! * [`rotate`]
//!
//! # Custom Event and Return types
//!
//! If the predefined types are not enough, custom types can be used as long as types for
//! events implement [`serde::Deserialize`] and return types implement [`serde::Serialize`].
//!
//! ```no_run
//! #[derive(serde::Deserialize, Debug)]
//! struct Event {
//!     #[serde(flatten)]
//!     attributes: std::collections::HashMap<String, serde_json::Value>,
//! }
//!
//! #[derive(serde::Serialize, Debug)]
//! struct Return {
//!     data: std::borrow::Cow<'static, str>,
//! }
//!
//! struct Runner;
//!
//! #[async_trait::async_trait]
//! impl lambda_runtime_types::Runner<(), Event, Return> for Runner {
//!     async fn run<'a>(shared: &'a (), event: Event, region: &'a str) -> anyhow::Result<Return> {
//!         println!("{:?}", event);
//!         Ok(Return {
//!             data: event
//!                 .attributes
//!                 .get("test")
//!                 .and_then(|a| a.as_str())
//!                 .map(ToOwned::to_owned)
//!                 .map(Into::into)
//!                 .unwrap_or_else(|| "none".into()),
//!         })
//!     }
//!
//!     async fn setup() -> anyhow::Result<()> {
//!         // Setup logging to make sure that errors are printed
//!         Ok(())
//!     }
//! }
//!
//! pub fn main() -> anyhow::Result<()> {
//!     lambda_runtime_types::exec_tokio::<_, _, Runner, _>()
//! }
//! ```
//!
//! # Shared Data
//!
//! With AWS Lambda, its possible to share data between invocations, as long as both
//! invocations use the same runtime environment. To use this functinality, its possible
//! to define a shared data type which will persist data by using Interior Mutability:
//!
//! ```no_run
//! #[derive(Default)]
//! struct Shared  {
//!     invocations: tokio::sync::Mutex<u64>,
//! }
//!
//! struct Runner;
//!
//! #[async_trait::async_trait]
//! impl lambda_runtime_types::Runner<Shared, (), ()> for Runner {
//!     async fn run<'a>(shared: &'a Shared, event: (), region: &'a str) -> anyhow::Result<()> {
//!         *shared.invocations.lock().await += 1;
//!         Ok(())
//!     }
//!
//!     async fn setup() -> anyhow::Result<()> {
//!         // Setup logging to make sure that errors are printed
//!         Ok(())
//!     }
//! }
//!
//! pub fn main() -> anyhow::Result<()> {
//!     lambda_runtime_types::exec_tokio::<_, _, Runner, _>()
//! }
//! ```
//!
//! Its important to know, that lambda execution evironments never run multiple invocations
//! simultaneously. Its therefore possible to keep the mutex unlocked for the whole invocation
//! as it will never block other invocations. Instead it is even recommended to do so, to
//! make sure that there are no unnessary things slowing down lambda execution time.
//!
//! # Timeout handling
//!
//! This crate implements a timeout handling logic. Normally, if a lambda runs into a timeout,
//! it will not create an error, which then does not get propagated by `on_error` destinations.
//!
//! To fix that, a timeout handler is setup, which will "fail" 100 miliseconds before the lambda
//! would run into a timeout, creating an error which then is propagated. There is, however, no
//! gurantee that this handler will fail in time. It will only work, when there are multiple
//! tokio threads or when the main lambda code is currently awaiting, giving tokio the chance
//! to switch tasks (or run them in parallel) and fail the execution.
//!
//! # Memory exhaustion
//!
//! Another thing to consider when running lambdas is memory exhaustion. Unfortunatly it is not
//! possible in rust to check the current memory usage. Therefore it is also not possible to
//! fail before running into OOF. When running lambdas, it may be necessary to setup checks to
//! verify that a lambda completed successfully, and did not run into OOF, as these errors also
//! do not get propagated to `on_error` destinations.
//!

#![warn(
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    deprecated_in_future,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    indirect_structural_match,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_copy_implementations,
    missing_crate_level_docs,
    missing_debug_implementations,
    missing_docs,
    missing_doc_code_examples,
    non_ascii_idents,
    private_doc_tests,
    trivial_casts,
    trivial_numeric_casts,
    unaligned_references,
    unreachable_pub,
    unsafe_code,
    unstable_features,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]
#![warn(
    clippy::correctness,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::cargo,
    clippy::nursery
)]
#![allow(clippy::multiple_crate_versions, clippy::future_not_send)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "rotate")]
#[cfg_attr(docsrs, doc(cfg(feature = "rotate")))]
pub mod rotate;

#[cfg(any(test, feature = "binary"))]
use simple_logger as _;

/// Defines a type which is executed every time a lambda
/// is invoced.
///
/// Types:
/// * `Shared`: Type which is shared between lambda
///             invocations. Note that lambda will
///             create multiple environments for
///             simulations invokations and environments
///             are only kept alive for a certain time.
///             It is thus not guaranteed that data
///             can be reused, but with this types
///             its possible.
/// * `Event`:  The expected Event which is being send
///             to the lambda by AWS.
/// * `Return`: Type which is the result of the lamba
///             invocation being returned to AWS
#[async_trait::async_trait]
pub trait Runner<Shared, Event, Return>
where
    Shared: Default + Send + Sync,
    Event: for<'de> serde::Deserialize<'de> + std::fmt::Debug,
    Return: serde::Serialize,
{
    /// Invoked only once before lambda runtime start. Does not get called on each
    /// lambda invocation. Can be used to setup logging and other global services,
    /// but should be short as it delays lambda startup
    async fn setup() -> anyhow::Result<()>;

    /// Invoked for every lambda invocation. Data in `shared` is persisted between
    /// invocations as long as they are running in the same `execution environment`
    ///
    /// More Info: <https://docs.aws.amazon.com/lambda/latest/dg/runtimes-context.html>
    async fn run<'a>(shared: &'a Shared, event: Event, region: &'a str) -> anyhow::Result<Return>;
}

/// Lambda entrypoint. This function sets up a lambda
/// multi-thread runtimes and executes [`exec`]. If you
/// already have your own runtime, use the [`exec`]
/// function.
///
/// Types:
/// * `Shared`: Type which is shared between lambda
///             invocations. Note that lambda will
///             create multiple environments for
///             simulations invokations and environments
///             are only kept alive for a certain time.
///             It is thus not guaranteed that data
///             can be reused, but with this types
///             its possible.
/// * `Event`:  The expected Event which is being send
///             to the lambda by AWS.
/// * `Run`:    Runner which is execued for each lambda
///             invocation.
/// * `Return`: Type which is the result of the lamba
///             invocation being returned to AWS
pub fn exec_tokio<Shared, Event, Run, Return>() -> anyhow::Result<()>
where
    Shared: Default + Send + Sync,
    Event: for<'de> serde::Deserialize<'de> + std::fmt::Debug,
    Run: Runner<Shared, Event, Return>,
    Return: serde::Serialize,
{
    use anyhow::Context;
    use tokio::runtime::Builder;

    Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("Unable to build tokio runtime")?
        .block_on(exec::<Shared, Event, Run, Return>())
}

/// Lambda entrypoint. This function requires a
/// running tokio runtime. Alternativly use [`exec_tokio`]
/// which creates one.
///
/// Types:
/// * `Shared`: Type which is shared between lambda
///             invocations. Note that lambda will
///             create multiple environments for
///             simulations invokations and environments
///             are only kept alive for a certain time.
///             It is thus not guaranteed that data
///             can be reused, but with this types
///             its possible.
/// * `Event`:  The expected Event which is being send
///             to the lambda by AWS.
/// * `Run`:    Runner which is execued for each lambda
///             invocation.
/// * `Return`: Type which is the result of the lamba
///             invocation being returned to AWS
pub async fn exec<Shared, Event, Run, Return>() -> anyhow::Result<()>
where
    Shared: Default + Send + Sync,
    Event: for<'de> serde::Deserialize<'de> + std::fmt::Debug,
    Run: Runner<Shared, Event, Return>,
    Return: serde::Serialize,
{
    use anyhow::{anyhow, Context};
    use lambda_runtime::{handler_fn, Context as LContext};
    use std::env;

    Run::setup().await?;
    log::info!("Starting lambda runtime");
    let region = env::var("AWS_REGION").context("Missing AWS_REGION env variable")?;
    let region_ref = &region;
    let shared = Shared::default();
    let shared_ref = &shared;
    lambda_runtime::run(handler_fn(move |data, context: LContext| {
        log::info!("Received lambda incation with event: {:?}", data);
        let deadline: u64 = context.deadline;
        run::<_, Event, Run, Return>(shared_ref, data, Some(deadline), region_ref)
    }))
    .await
    .map_err(|e| anyhow!(e))
}

#[allow(clippy::unit_arg)]
async fn run<Shared, Event, Run, Return>(
    shared: &Shared,
    event: Event,
    deadline_in_ms: Option<u64>,
    region: &str,
) -> anyhow::Result<Return>
where
    Shared: Default + Send + Sync,
    Event: for<'de> serde::Deserialize<'de> + std::fmt::Debug,
    Run: Runner<Shared, Event, Return>,
    Return: serde::Serialize,
{
    use anyhow::anyhow;
    use futures::FutureExt;

    let mut runner = Run::run(shared, event, region).fuse();
    let res = if let Some(deadline_in_ms) = deadline_in_ms {
        let mut timeout = Box::pin(timeout_handler(deadline_in_ms).fuse());
        futures::select! {
            res = runner => res,
            _ = timeout => Err(anyhow!("Lambda failed by running into a timeout")),
        }
    } else {
        runner.await
    };
    log::info!("Completed lambda invocation");
    match res {
        Ok(res) => Ok(res),
        Err(err) => {
            log::error!("{:?}", err);
            Err(err)
        }
    }
}

async fn timeout_handler(deadline_in_ms: u64) {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use tokio::time::Instant;

    let epoch = UNIX_EPOCH;
    let now = SystemTime::now();
    let now_instant = Instant::now();

    let duration_from_now = now.duration_since(epoch).expect("Time went backwards");
    let duration_from_epoch = Duration::from_millis(deadline_in_ms);
    let duration_deadline = duration_from_epoch - duration_from_now - Duration::from_millis(100);

    let deadline = now_instant + duration_deadline;
    log::info!("Setting deadline to: {:?}", deadline);
    tokio::time::sleep_until(deadline).await;
}

/// TestData which can be used to test lambda invocations
/// locally in combination with [`exec_test`].
#[derive(serde::Deserialize, Clone, Debug)]
#[cfg(feature = "test")]
#[cfg_attr(docsrs, doc(cfg(feature = "test")))]
pub struct TestData<Event> {
    region: String,
    invocations: Vec<Event>,
}

/// Lambda entrypoint. This function can be used to
/// test one or multiple lambda invocations locally.
///
/// Types:
/// * `Shared`: Type which is shared between lambda
///             invocations. Note that lambda will
///             create multiple environments for
///             simulations invokations and environments
///             are only kept alive for a certain time.
///             It is thus not guaranteed that data
///             can be reused, but with this types
///             its possible.
/// * `Event`:  The expected Event which is being send
///             to the lambda by AWS.
/// * `Run`:    Runner which is execued for each lambda
///             invocation.
/// * `Return`: Type which is the result of the lamba
///             invocation being returned to AWS
#[cfg(feature = "test")]
#[cfg_attr(docsrs, doc(cfg(feature = "test")))]
pub fn exec_test<Shared, Event, Run, Return>(test_data: &str) -> anyhow::Result<()>
where
    Shared: Default + Send + Sync,
    Event: for<'de> serde::Deserialize<'de> + std::fmt::Debug,
    Run: Runner<Shared, Event, Return>,
    Return: serde::Serialize + std::fmt::Debug,
{
    use anyhow::Context;
    use tokio::runtime::Builder;

    log::info!("Creating tokio runtime");
    Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("Unable to build tokio runtime")?
        .block_on(async {
            Run::setup().await?;
            log::info!("Starting lambda test runtime");
            let test_data: TestData<Event> =
                serde_json::from_str(test_data).context("Unable to deserialize test_data")?;
            let shared = Shared::default();
            let shared_ref = &shared;
            let region_ref = &test_data.region;

            for (i, data) in test_data.invocations.into_iter().enumerate() {
                log::info!("Invocation: {}", i);
                let res = run::<_, Event, Run, Return>(shared_ref, data, None, region_ref).await?;
                log::info!("{:?}", res);
            }
            Ok(())
        })
}
