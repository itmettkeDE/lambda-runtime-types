#[derive(Clone)]
pub struct SmcClient {
    client: aws_sdk_secretsmanager::Client,
}

impl SmcClient {
    pub async fn new() -> Self {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_secretsmanager::Client::new(&config);
        Self { client }
    }

    pub async fn generate_new_password(
        &self,
        puncutation: bool,
        length: Option<i64>,
    ) -> anyhow::Result<String> {
        use anyhow::Context;

        self.client
            .get_random_password()
            .exclude_characters("\"")
            .exclude_punctuation(puncutation)
            .set_password_length(length)
            .send()
            .await
            .context("Unable to generate new password")?
            .random_password
            .context("Generated password is empty")
    }

    pub async fn get_secret_value<S: serde::de::DeserializeOwned>(
        &self,
        secret_id: &str,
        version_stage: &str,
    ) -> anyhow::Result<crate::rotate::smc::Secret<S>> {
        use anyhow::Context;

        let secret_value = self
            .client
            .get_secret_value()
            .secret_id(secret_id)
            .version_stage(version_stage)
            .send()
            .await
            .with_context(|| format!("Unable to fetch SecretValue with id: {}", secret_id))?;
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
            (_, Some(bytes)) => serde_json::from_slice(bytes.as_ref()),
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

        self.client
            .put_secret_value()
            .set_client_request_token(request_token.map(|v| v.to_string()))
            .secret_id(secret_id)
            .secret_string(secret_str)
            .version_stages("AWSPENDING")
            .send()
            .await
            .with_context(|| {
                format!(
                    "Unable to push new SecretValue to AWSPENDING for id: {}",
                    secret_id
                )
            })?;
        Ok(())
    }

    pub async fn set_pending_secret_value_to_current(
        &self,
        secret_arn: String,
        secret_current_version_id: String,
        secret_pending_version_id: String,
    ) -> anyhow::Result<()> {
        use anyhow::Context;

        self.client
            .update_secret_version_stage()
            .move_to_version_id(secret_pending_version_id)
            .remove_from_version_id(secret_current_version_id)
            .secret_id(&secret_arn)
            .version_stage("AWSCURRENT")
            .send()
            .await
            .with_context(|| {
                format!(
                    "Unable to push new SecretValue to AWSPENDING for arn: {}",
                    secret_arn
                )
            })?;
        Ok(())
    }
}
