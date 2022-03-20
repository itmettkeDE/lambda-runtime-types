#[derive(Clone)]
pub struct SmcClient {
    client: rusoto_secretsmanager::SecretsManagerClient,
}

impl SmcClient {
    pub fn new(region: &str) -> anyhow::Result<Self> {
        use anyhow::Context;
        use std::str::FromStr;

        let region =
            rusoto_core::Region::from_str(region).context("invalid region given to lambda")?;
        let client = rusoto_secretsmanager::SecretsManagerClient::new(region);
        Ok(Self { client })
    }

    pub async fn generate_new_password(
        &self,
        puncutation: bool,
        length: Option<i64>,
    ) -> anyhow::Result<String> {
        use anyhow::Context;
        use rusoto_secretsmanager::SecretsManager;

        let mut retries = 1;
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
            if Self::is_wait_and_repeat(&res, retries).await {
                retries += 1;
                continue;
            }
            break res.context("Unable to generate new password")?;
        };
        password
            .random_password
            .context("Generated password is empty")
    }

    pub async fn get_secret_value<S: serde::de::DeserializeOwned>(
        &self,
        secret_id: &str,
        version_stage: &str,
    ) -> anyhow::Result<crate::rotate::smc::Secret<S>> {
        use anyhow::Context;
        use rusoto_secretsmanager::SecretsManager;

        let mut retries = 1;
        let secret_value = loop {
            let res = self
                .client
                .get_secret_value(rusoto_secretsmanager::GetSecretValueRequest {
                    secret_id: secret_id.to_string(),
                    version_id: None,
                    version_stage: Some(version_stage.to_string()),
                })
                .await;
            if Self::is_wait_and_repeat(&res, retries).await {
                retries += 1;
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
        Ok(crate::rotate::smc::Secret {
            arn,
            version_id,
            inner,
        })
    }

    pub async fn put_secret_value_pending(
        &self,
        secret_id: &str,
        request_token: Option<&str>,
        secret_str: &str,
    ) -> anyhow::Result<()> {
        use anyhow::Context;
        use rusoto_secretsmanager::SecretsManager;

        let mut retries = 1;
        loop {
            let res = self
                .client
                .put_secret_value(rusoto_secretsmanager::PutSecretValueRequest {
                    client_request_token: request_token.map(|v| v.to_string()),
                    secret_binary: None,
                    secret_id: secret_id.to_string(),
                    secret_string: Some(secret_str.into()),
                    version_stages: Some(vec!["AWSPENDING".into()]),
                })
                .await;
            if Self::is_wait_and_repeat(&res, retries).await {
                retries += 1;
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

    pub async fn set_pending_secret_value_to_current(
        &self,
        secret_arn: String,
        secret_current_version_id: String,
        secret_pending_version_id: String,
    ) -> anyhow::Result<()> {
        use anyhow::Context;
        use rusoto_secretsmanager::SecretsManager;

        let mut retries = 1;
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
            if Self::is_wait_and_repeat(&res, retries).await {
                retries += 1;
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
        retries: u64,
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
                503 => {
                    let search = b"SlowDown";
                    body.as_ref().windows(search.len()).any(|sub| sub == search)
                }
                _ => false,
            };
            if cooldown {
                println!("Info: Cooling down to prevent request limits");
                tokio::time::sleep(tokio::time::Duration::from_millis((2 ^ retries) * 100)).await;
                return true;
            }
        }
        false
    }
}
