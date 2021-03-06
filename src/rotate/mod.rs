//! Provides types for lambdas used for Secret Manager rotation.
//!
//! # Usage
//!
//! ```no_run
//! #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
//! struct Secret {
//!     user: String,
//!     password: String,
//! }
//!
//! struct Runner;
//!
//! #[async_trait::async_trait]
//! impl<'a> lambda_runtime_types::rotate::RotateRunner<'a, (), Secret> for Runner {
//!     async fn setup(region: &'a str) -> anyhow::Result<()> {
//!         // Setup logging to make sure that errors are printed
//!         Ok(())
//!     }
//!
//!     async fn create(
//!         shared: &'a (),
//!         secret_cur: lambda_runtime_types::rotate::SecretContainer<Secret>,
//!         smc: &lambda_runtime_types::rotate::Smc,
//!     ) -> anyhow::Result<lambda_runtime_types::rotate::SecretContainer<Secret>> {
//!         // Create a new secret without setting it yet.
//!         // Only called if there is no pending secret available
//!         // (which may happen if rotation fails at any stage)  
//!         unimplemented!()
//!     }
//!
//!     async fn set(
//!         shared: &'a (),
//!         secret_cur: lambda_runtime_types::rotate::SecretContainer<Secret>,
//!         secret_new: lambda_runtime_types::rotate::SecretContainer<Secret>,
//!     ) -> anyhow::Result<()> {
//!         // Set the secret in the service
//!         // Only called if password is not already set, checked by  
//!         // calling [`test`] with new password beforehand. The reason
//!         // for that it, that a failure in a later stage means all
//!         // stages are called again with set failing as the old password
//!         // does not work anymore
//!         Ok(())
//!     }
//!
//!     async fn test(
//!         shared: &'a (),
//!         secret_new: lambda_runtime_types::rotate::SecretContainer<Secret>,
//!     ) -> anyhow::Result<()> {
//!         // Test whether a connection with the given secret works
//!         Ok(())
//!     }
//!
//!     async fn finish(
//!         shared: &'a (),
//!         secret_cur: lambda_runtime_types::rotate::SecretContainer<Secret>,
//!         secret_new: lambda_runtime_types::rotate::SecretContainer<Secret>,
//!     ) -> anyhow::Result<()> {
//!         // Optional: Perform any work which may be necessary to
//!         // complete rotation
//!         Ok(())
//!     }
//!
//! }
//!
//! pub fn main() -> anyhow::Result<()> {
//!     lambda_runtime_types::exec_tokio::<_, _, Runner, _>()
//! }
//! ```
//!
//! For further usage like `Shared` Data, refer to the main [documentation](`crate`)

#[cfg(feature = "rotate_aws_sdk")]
mod aws_sdk;
#[cfg(feature = "rotate_rusoto")]
mod rusoto;
mod smc;

pub use smc::{SecretContainer, Smc};

/// `Event` which is send by the `SecretManager` to the rotation lambda
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "rotate_rusoto", feature = "rotate_aws_sdk")))
)]
#[derive(Clone, serde::Deserialize)]
pub struct Event<Secret> {
    /// Request Token used for `SecretManager` Operations
    #[serde(rename = "ClientRequestToken")]
    pub client_request_token: String,
    /// Id of the secret to rotate
    #[serde(rename = "SecretId")]
    pub secret_id: String,
    /// Current step of the rotation
    #[serde(rename = "Step")]
    pub step: Step,
    #[doc(hidden)]
    #[serde(skip)]
    pub _m: std::marker::PhantomData<Secret>,
}

impl<Secret> std::fmt::Debug for Event<Secret> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Event")
            .field("client_request_token", &self.client_request_token)
            .field("secret_id", &self.secret_id)
            .field("step", &self.step)
            .finish()
    }
}

/// Available steps for in a Secret Manager rotation
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "rotate_rusoto", feature = "rotate_aws_sdk")))
)]
#[derive(Debug, Copy, Clone, serde::Deserialize)]
pub enum Step {
    /// Secret creation
    #[serde(rename = "createSecret")]
    Create,
    /// Secret configuration in service
    #[serde(rename = "setSecret")]
    Set,
    /// Secret testing in service
    #[serde(rename = "testSecret")]
    Test,
    /// Secret rotation finalization
    #[serde(rename = "finishSecret")]
    Finish,
}

/// Defines a type which is executed every time a lambda
/// is invoced. This type is made for `SecretManager`
/// rotation lambdas.
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
/// * `Secret`: The structure of the secret stored in
///             the `SecretManager`. May contain only
///             necessary fields, as other undefined
///             fields are internally preserved.
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "rotate_rusoto", feature = "rotate_aws_sdk")))
)]
#[async_trait::async_trait]
pub trait RotateRunner<'a, Shared, Secret>
where
    Shared: Send + Sync + 'a,
    Secret: 'static + Send,
{
    /// See documentation of [`super::Runner::setup`]
    async fn setup(region: &'a str) -> anyhow::Result<Shared>;

    /// Create a new secret without setting it yet.
    /// Only called if there is no pending secret available
    /// (which may happen if rotation fails at any stage)
    async fn create(
        shared: &'a Shared,
        secret_cur: SecretContainer<Secret>,
        smc: &Smc,
    ) -> anyhow::Result<SecretContainer<Secret>>;

    /// Set the secret in the service
    /// Only called if password is not already set, checked by  
    /// calling [`test`] with new password beforehand. The reason
    /// for that it, that a failure in a later stage means all
    /// stages are called again with set failing as the old password
    /// does not work anymore
    async fn set(
        shared: &'a Shared,
        secret_cur: SecretContainer<Secret>,
        secret_new: SecretContainer<Secret>,
    ) -> anyhow::Result<()>;

    /// Test whether a connection with the given secret works
    async fn test(shared: &'a Shared, secret_new: SecretContainer<Secret>) -> anyhow::Result<()>;

    /// Perform any work which may be necessary to complete rotation
    async fn finish(
        _shared: &'a Shared,
        _secret_cur: SecretContainer<Secret>,
        _secret_new: SecretContainer<Secret>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl<'a, Type, Shared, Sec> super::Runner<'a, Shared, Event<Sec>, ()> for Type
where
    Shared: Send + Sync + 'a,
    Sec: 'static + Send + Sync + Clone + serde::de::DeserializeOwned + serde::Serialize,
    Type: 'static + RotateRunner<'a, Shared, Sec>,
{
    async fn setup(region: &'a str) -> anyhow::Result<Shared> {
        Self::setup(region).await
    }

    async fn run(
        shared: &'a Shared,
        event: crate::LambdaEvent<'a, Event<Sec>>,
    ) -> anyhow::Result<()> {
        let smc = Smc::new(event.region).await?;
        log::info!("{:?}", event.event.step);
        match event.event.step {
            Step::Create => {
                let secret_cur = smc
                    .get_secret_value_current::<Sec>(&event.event.secret_id)
                    .await?;
                let secret_new = smc
                    .get_secret_value_pending::<Sec>(&event.event.secret_id)
                    .await;
                if let Ok(secret_new) = secret_new {
                    if secret_new.version_id != secret_cur.version_id {
                        log::info!("Found existing pending value.");
                        return Ok(());
                    }
                }
                log::info!("Creating new secret value.");
                let secret = Self::create(shared, secret_cur.inner, &smc).await?;
                smc.put_secret_value_pending(
                    &event.event.secret_id,
                    Some(&event.event.client_request_token),
                    &secret,
                )
                .await?;
                Ok(())
            }
            Step::Set => {
                log::info!("Setting secret on remote system.");
                let secret_new = smc
                    .get_secret_value_pending(&event.event.secret_id)
                    .await?
                    .inner;
                if Self::test(shared, SecretContainer::clone(&secret_new))
                    .await
                    .is_err()
                {
                    let secret_cur = smc
                        .get_secret_value_current(&event.event.secret_id)
                        .await?
                        .inner;
                    Self::set(shared, secret_cur, secret_new).await?;
                } else {
                    log::info!("Password already set in remote system.");
                }
                Ok(())
            }
            Step::Test => {
                log::info!("Testing secret on remote system.");
                let secret = smc
                    .get_secret_value_pending(&event.event.secret_id)
                    .await?
                    .inner;
                Self::test(shared, secret).await?;
                Ok(())
            }
            Step::Finish => {
                log::info!("Finishing secret deployment.");
                let secret_current: smc::Secret<Sec> =
                    smc.get_secret_value_current(&event.event.secret_id).await?;
                let secret_pending: smc::Secret<Sec> =
                    smc.get_secret_value_pending(&event.event.secret_id).await?;
                Self::finish(shared, secret_current.inner, secret_pending.inner).await?;
                smc.set_pending_secret_value_to_current(
                    secret_current.arn,
                    secret_current.version_id,
                    secret_pending.version_id,
                )
                .await?;
                Ok(())
            }
        }
    }
}
