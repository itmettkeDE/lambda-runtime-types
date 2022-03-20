# lambda-runtime-types

This crate provides types and traits to simplify
the creation of lambda functions in rust. It
provides Event and Return types and specific
Runners for various lambda types.

## Basic Lambda with no shared data

Creating a normal lambda is very easy. First create a type which implements [`Runner`] and
then use it either in the [`exec`] or [`exec_tokio`] function:

```rust
struct Runner;

#[async_trait::async_trait]
impl<'a> lambda_runtime_types::Runner<'a, (), (), ()> for Runner {
    async fn run(shared: &'a (), event: lambda_runtime_types::LambdaEvent<'a, ()>) -> anyhow::Result<()> {
        // Run code on every invocation
        Ok(())
    }

    async fn setup(_region: &'a str) -> anyhow::Result<()> {
        // Setup logging to make sure that errors are printed
        Ok(())
    }
}

pub fn main() -> anyhow::Result<()> {
    lambda_runtime_types::exec_tokio::<_, _, Runner, _>()
}
```

## Available lambda types

There are various modules which predefined Event and Return types and Runner traits
specialised for differnet lambda usages. Check out the modules for examples or their
usage.

- [`rotate`]

## Custom Event and Return types

If the predefined types are not enough, custom types can be used as long as types for
events implement [`serde::Deserialize`] and return types implement [`serde::Serialize`].

```rust
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
impl<'a> lambda_runtime_types::Runner<'a, (), Event, Return> for Runner {
    async fn run(shared: &'a (), event: lambda_runtime_types::LambdaEvent<'a, Event>) -> anyhow::Result<Return> {
        println!("{:?}", event);
        Ok(Return {
            data: event
                .event
                .attributes
                .get("test")
                .and_then(|a| a.as_str())
                .map(ToOwned::to_owned)
                .map(Into::into)
                .unwrap_or_else(|| "none".into()),
        })
    }

    async fn setup(_region: &'a str) -> anyhow::Result<()> {
        // Setup logging to make sure that errors are printed
        Ok(())
    }
}

pub fn main() -> anyhow::Result<()> {
    lambda_runtime_types::exec_tokio::<_, _, Runner, _>()
}
```

## Shared Data

With AWS Lambda, its possible to share data between invocations, as long as both
invocations use the same runtime environment. To use this functinality, its possible
to define a shared data type which will persist data by using Interior Mutability:

```rust
#[derive(Default)]
struct Shared  {
    invocations: tokio::sync::Mutex<u64>,
}

struct Runner;

#[async_trait::async_trait]
impl<'a> lambda_runtime_types::Runner<'a, Shared, (), ()> for Runner {
    async fn run(shared: &'a Shared, event: lambda_runtime_types::LambdaEvent<'a, ()>) -> anyhow::Result<()> {
        let mut invocations = shared.invocations.lock().await;
        *invocations += 1;
        Ok(())
    }

    async fn setup(_region: &'a str) -> anyhow::Result<Shared> {
        // Setup logging to make sure that errors are printed
        Ok(Shared::default())
    }
}

pub fn main() -> anyhow::Result<()> {
    lambda_runtime_types::exec_tokio::<_, _, Runner, _>()
}
```

Its important to know, that lambda execution evironments never run multiple invocations
simultaneously. Its therefore possible to keep the mutex unlocked for the whole invocation
as it will never block other invocations. Instead it is even recommended to do so, to
make sure that there are no unnessary things slowing down lambda execution time.

## Timeout handling

This crate implements a timeout handling logic. Normally, if a lambda runs into a timeout,
it will not create an error, which then does not get propagated by `on_error` destinations.

To fix that, a timeout handler is setup, which will "fail" 100 miliseconds before the lambda
would run into a timeout, creating an error which then is propagated. There is, however, no
gurantee that this handler will fail in time. It will only work, when there are multiple
tokio threads or when the main lambda code is currently awaiting, giving tokio the chance
to switch tasks (or run them in parallel) and fail the execution.

## Memory exhaustion

Another thing to consider when running lambdas is memory exhaustion. Unfortunatly it is not
possible in rust to check the current memory usage. Therefore it is also not possible to
fail before running into OOF. When running lambdas, it may be necessary to setup checks to
verify that a lambda completed successfully, and did not run into OOF, as these errors also
do not get propagated to `on_error` destinations.

License: MIT OR Apache-2.0
