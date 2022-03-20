/// Secret returned by Secret Manager
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "rotate_rusoto", feature = "rotate_aws_sdk")))
)]
#[derive(Debug, Clone)]
pub struct Secret<S> {
    /// Arn to the secret
    pub arn: String,
    /// Secret version_id
    pub version_id: String,
    /// Inner custom secret
    pub inner: SecretContainer<S>,
}

/// Transparent container to inner value.
/// Prevents accidental override of values not defined by `S`
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "rotate_rusoto", feature = "rotate_aws_sdk")))
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SecretContainer<S> {
    /// Secret data as defined by `S`
    #[serde(flatten)]
    pub data: S,
    /// Other fields not defined by `S`. Necessary to preserve
    /// available fields, which are not defined in the type.
    /// Enabled by default with feature `rotate_with_preserve`
    #[cfg_attr(docsrs, doc(cfg(feature = "rotate_with_preserve")))]
    #[cfg(feature = "rotate_with_preserve")]
    #[serde(flatten)]
    pub o: std::collections::HashMap<String, serde_json::Value>,
}

impl<S> std::ops::Deref for SecretContainer<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<S> std::ops::DerefMut for SecretContainer<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// Secret Manager Client
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "rotate_rusoto", feature = "rotate_aws_sdk")))
)]
#[derive(Clone)]
pub struct Smc {
    #[cfg(feature = "rotate_aws_sdk")]
    aws_sdk_client: super::aws_sdk::SmcClient,
    #[cfg(feature = "rotate_rusoto")]
    rusoto_client: super::rusoto::SmcClient,
}

impl std::fmt::Debug for Smc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Smc").field("client", &"[...]").finish()
    }
}

impl Smc {
    /// Create a new secret manager client
    pub async fn new(_region: &str) -> anyhow::Result<Self> {
        Ok(Self {
            #[cfg(feature = "rotate_aws_sdk")]
            aws_sdk_client: super::aws_sdk::SmcClient::new().await,
            #[cfg(feature = "rotate_rusoto")]
            rusoto_client: super::rusoto::SmcClient::new(_region)?,
        })
    }

    /// Generate a new password
    pub async fn generate_new_password(
        &self,
        puncutation: bool,
        length: Option<i64>,
    ) -> anyhow::Result<String> {
        #[cfg(all(feature = "rotate_aws_sdk", not(feature = "rotate_rusoto")))]
        let client = &self.aws_sdk_client;
        #[cfg(all(feature = "rotate_rusoto", not(feature = "rotate_aws_sdk")))]
        let client = &self.rusoto_client;
        #[cfg(all(feature = "rotate_rusoto", feature = "rotate_aws_sdk"))]
        compile_error("Only rotate_rusoto or rotate_aws_sdk can be enabled at once");

        client.generate_new_password(puncutation, length).await
    }

    /// Fetches the current secret value of the given secret_id
    pub(crate) async fn get_secret_value_current<S: serde::de::DeserializeOwned>(
        &self,
        secret_id: &str,
    ) -> anyhow::Result<Secret<S>> {
        self.get_secret_value(secret_id, "AWSCURRENT").await
    }

    /// Fetches the pending secret value of the given secret_id
    pub(crate) async fn get_secret_value_pending<S: serde::de::DeserializeOwned>(
        &self,
        secret_id: &str,
    ) -> anyhow::Result<Secret<S>> {
        self.get_secret_value(secret_id, "AWSPENDING").await
    }

    async fn get_secret_value<S: serde::de::DeserializeOwned>(
        &self,
        secret_id: &str,
        version_stage: &str,
    ) -> anyhow::Result<Secret<S>> {
        #[cfg(all(feature = "rotate_aws_sdk", not(feature = "rotate_rusoto")))]
        let client = &self.aws_sdk_client;
        #[cfg(all(feature = "rotate_rusoto", not(feature = "rotate_aws_sdk")))]
        let client = &self.rusoto_client;
        #[cfg(all(feature = "rotate_rusoto", feature = "rotate_aws_sdk"))]
        compile_error("Only rotate_rusoto or rotate_aws_sdk can be enabled at once");

        client.get_secret_value(secret_id, version_stage).await
    }

    pub(crate) async fn put_secret_value_pending<S: serde::Serialize + Send + Sync>(
        &self,
        secret_id: &str,
        request_token: Option<&str>,
        value: &SecretContainer<S>,
    ) -> anyhow::Result<()> {
        use anyhow::Context;

        #[cfg(all(feature = "rotate_aws_sdk", not(feature = "rotate_rusoto")))]
        let client = &self.aws_sdk_client;
        #[cfg(all(feature = "rotate_rusoto", not(feature = "rotate_aws_sdk")))]
        let client = &self.rusoto_client;
        #[cfg(all(feature = "rotate_rusoto", feature = "rotate_aws_sdk"))]
        compile_error("Only rotate_rusoto or rotate_aws_sdk can be enabled at once");

        let secret_string: String = serde_json::to_string(value)
            .with_context(|| format!("Unable to serialize secret_value with id: {}", secret_id))?;
        client
            .put_secret_value_pending(secret_id, request_token, &secret_string)
            .await
    }

    pub(crate) async fn set_pending_secret_value_to_current(
        &self,
        secret_arn: String,
        secret_current_version_id: String,
        secret_pending_version_id: String,
    ) -> anyhow::Result<()> {
        #[cfg(all(feature = "rotate_aws_sdk", not(feature = "rotate_rusoto")))]
        let client = &self.aws_sdk_client;
        #[cfg(all(feature = "rotate_rusoto", not(feature = "rotate_aws_sdk")))]
        let client = &self.rusoto_client;
        #[cfg(all(feature = "rotate_rusoto", feature = "rotate_aws_sdk"))]
        compile_error("Only rotate_rusoto or rotate_aws_sdk can be enabled at once");

        client
            .set_pending_secret_value_to_current(
                secret_arn,
                secret_current_version_id,
                secret_pending_version_id,
            )
            .await
    }
}
