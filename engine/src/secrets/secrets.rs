use async_trait::async_trait;
use sec::Secret;

use super::error::SecretError;

type SecretString = Secret<String>;

#[async_trait]
pub trait SecretsReader {
    async fn get_secret(secret_name: &str) -> Result<SecretString, SecretError>;
}

pub struct EnvSecrets;

#[async_trait]
impl SecretsReader for EnvSecrets {
    async fn get_secret(secret_name: &str) -> Result<SecretString, SecretError> {
        let secret = std::env::var(secret_name).map_err(|e| match e {
            std::env::VarError::NotPresent => SecretError::SecretNotFound(secret_name.to_owned()),
            std::env::VarError::NotUnicode(e) => SecretError::OtherError(e.into()),
        });

        Ok(SecretString::new(secret))
    }
}
