/// Secret returned by Secret Manager
#[cfg_attr(docsrs, doc(cfg(feature = "rotate")))]
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
#[cfg_attr(docsrs, doc(cfg(feature = "rotate")))]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SecretContainer<S> {
    /// Secret data as defined by `S`
    #[serde(flatten)]
    pub data: S,
    /// Other fields not defined by `S`. Necessary to preserve
    /// available fields, which are not defined in the type.
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
#[cfg_attr(docsrs, doc(cfg(feature = "rotate")))]
#[derive(Clone)]
pub struct Smc {
    client: rusoto_secretsmanager::SecretsManagerClient,
}

impl std::fmt::Debug for Smc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Smc").field("client", &"[...]").finish()
    }
}

impl Smc {
    /// Create a new secret manager client
    pub fn new(region: rusoto_core::Region) -> Self {
        Self {
            client: rusoto_secretsmanager::SecretsManagerClient::new(region),
        }
    }

    /// Generate a new password
    pub async fn generate_new_password(
        &self,
        puncutation: bool,
        length: Option<i64>,
    ) -> anyhow::Result<String> {
        use anyhow::Context;
        use rusoto_secretsmanager::SecretsManager;

        let password = loop {
            let res = self
                .client
                .get_random_password(rusoto_secretsmanager::GetRandomPasswordRequest {
                    exclude_characters: Some("\"".to_string()),
                    exclude_punctuation: Some(!puncutation),
                    password_length: length,
                    ..rusoto_secretsmanager::GetRandomPasswordRequest::default()
                })
                .await;
            if Self::is_wait_and_repeat(&res).await {
                continue;
            }
            break res.context("Unable to generate new password")?;
        };
        password
            .random_password
            .context("Generated password is empty")
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
        use anyhow::Context;
        use rusoto_secretsmanager::SecretsManager;

        let secret_value = loop {
            let res = self
                .client
                .get_secret_value(rusoto_secretsmanager::GetSecretValueRequest {
                    secret_id: secret_id.to_string(),
                    version_id: None,
                    version_stage: Some(version_stage.to_string()),
                })
                .await;
            if Self::is_wait_and_repeat(&res).await {
                continue;
            }
            break res
                .with_context(|| format!("Unable to fetch SecretValue with id: {}", secret_id))?;
        };
        let arn = secret_value.arn.with_context(|| {
            format!("Arn is unavailable for secret value with id: {}", secret_id)
        })?;
        let version_id = secret_value.version_id.with_context(|| {
            format!(
                "version_id is unavailable for secret value with id: {}",
                secret_id
            )
        })?;
        let inner = match (secret_value.secret_string, secret_value.secret_binary) {
            (Some(string), _) => serde_json::from_str(&string),
            (_, Some(bytes)) => serde_json::from_slice(&bytes),
            _ => anyhow::bail!("Neither secret_string nor secret_binary is set for id: {}", secret_id),
        }
        .with_context(|| format!("Unable to parse secret value. Value does not confirm to required structure. Id: {}", secret_id))?;
        Ok(Secret {
            arn,
            version_id,
            inner,
        })
    }

    pub(crate) async fn put_secret_value_pending<S: serde::Serialize>(
        &self,
        secret_id: &str,
        request_token: Option<&str>,
        value: &SecretContainer<S>,
    ) -> anyhow::Result<()> {
        use anyhow::Context;
        use rusoto_secretsmanager::SecretsManager;

        loop {
            let secret_string = serde_json::to_string(value).with_context(|| {
                format!("Unable to serialize secret_value with id: {}", secret_id)
            })?;
            let res = self
                .client
                .put_secret_value(rusoto_secretsmanager::PutSecretValueRequest {
                    client_request_token: request_token.map(|v| v.to_string()),
                    secret_binary: None,
                    secret_id: secret_id.to_string(),
                    secret_string: Some(secret_string),
                    version_stages: Some(vec!["AWSPENDING".into()]),
                })
                .await;
            if Self::is_wait_and_repeat(&res).await {
                continue;
            }
            let _ = res.with_context(|| {
                format!(
                    "Unable to push new SecretValue to AWSPENDING for id: {}",
                    secret_id
                )
            })?;
            break Ok(());
        }
    }

    pub(crate) async fn set_pending_secret_value_to_current(
        &self,
        secret_arn: String,
        secret_current_version_id: String,
        secret_pending_version_id: String,
    ) -> anyhow::Result<()> {
        use anyhow::Context;
        use rusoto_secretsmanager::SecretsManager;

        loop {
            let res = self
                .client
                .update_secret_version_stage(
                    rusoto_secretsmanager::UpdateSecretVersionStageRequest {
                        move_to_version_id: Some(secret_pending_version_id.clone()),
                        remove_from_version_id: Some(secret_current_version_id.clone()),
                        secret_id: secret_arn.clone(),
                        version_stage: "AWSCURRENT".into(),
                    },
                )
                .await;
            if Self::is_wait_and_repeat(&res).await {
                continue;
            }
            let _ = res.with_context(|| {
                format!(
                    "Unable to push new SecretValue to AWSPENDING for arn: {}",
                    secret_arn
                )
            })?;
            break Ok(());
        }
    }

    /// Checks whether the given result is a throttling error
    /// and waits for 100 ms if it is
    async fn is_wait_and_repeat<D: Send + Sync, E: std::fmt::Debug + Send + Sync>(
        error: &Result<D, rusoto_core::RusotoError<E>>,
    ) -> bool {
        if let Err(rusoto_core::RusotoError::Unknown(
            rusoto_core::request::BufferedHttpResponse {
                ref status,
                ref body,
                ..
            },
        )) = *error
        {
            let cooldown = match status.as_u16() {
                400 => {
                    let search = b"ThrottlingException";
                    body.as_ref().windows(search.len()).any(|sub| sub == search)
                }
                429 => {
                    let search = b"Too Many Requests";
                    body.as_ref().windows(search.len()).any(|sub| sub == search)
                }
                _ => false,
            };
            if cooldown {
                println!("Info: Cooling down to prevent request limits");
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                return true;
            }
        }
        false
    }
}
