use std::{env, net::SocketAddr, path::PathBuf, time::Duration};

use axum::http::HeaderValue;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    Missing(&'static str),
    #[error("invalid environment variable {name}: {source}")]
    Invalid {
        name: &'static str,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("invalid environment variable {name}: must be between {min} and {max}")]
    OutOfRange {
        name: &'static str,
        min: u64,
        max: u64,
    },
}

const MIN_COMPILE_TIMEOUT_SECONDS: u64 = 1;
const MAX_COMPILE_TIMEOUT_SECONDS: u64 = 90;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: SocketAddr,
    pub frontend_origin: HeaderValue,
    pub compiler_enabled: bool,
    pub compiler_path: PathBuf,
    pub compile_timeout: Duration,
    pub session_ttl: Duration,
    pub cookie_secure: bool,
    pub google: Option<GoogleConfig>,
}

#[derive(Clone, Debug)]
pub struct GoogleConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url =
            env::var("DATABASE_URL").map_err(|_| ConfigError::Missing("DATABASE_URL"))?;
        let bind_addr = parse(
            "BIND_ADDR",
            env::var("BIND_ADDR")
                .ok()
                .unwrap_or_else(|| "127.0.0.1:18732".into()),
        )?;
        let frontend_origin = parse(
            "FRONTEND_ORIGIN",
            env::var("FRONTEND_ORIGIN")
                .ok()
                .unwrap_or_else(|| "http://localhost:5173".into()),
        )?;
        let compiler_enabled = parse(
            "LATEX_COMPILER_ENABLED",
            env::var("LATEX_COMPILER_ENABLED")
                .ok()
                .unwrap_or_else(|| "false".into()),
        )?;
        let compiler_path = env::var("LATEX_COMPILER_PATH")
            .unwrap_or_else(|_| "xelatex".into())
            .into();
        let compile_timeout = parse_compile_timeout(
            env::var("COMPILE_TIMEOUT_SECONDS")
                .ok()
                .unwrap_or_else(|| "30".into()),
        )?;
        let session_ttl = Duration::from_secs(parse(
            "SESSION_TTL_SECONDS",
            env::var("SESSION_TTL_SECONDS")
                .ok()
                .unwrap_or_else(|| "2592000".into()),
        )?);
        let cookie_secure = parse(
            "COOKIE_SECURE",
            env::var("COOKIE_SECURE")
                .ok()
                .unwrap_or_else(|| "false".into()),
        )?;
        let google = match (
            env::var("GOOGLE_CLIENT_ID").ok(),
            env::var("GOOGLE_CLIENT_SECRET").ok(),
            env::var("GOOGLE_REDIRECT_URI").ok(),
        ) {
            (None, None, None) => None,
            (Some(client_id), Some(client_secret), Some(redirect_uri)) => Some(GoogleConfig {
                client_id,
                client_secret,
                redirect_uri,
            }),
            (None, _, _) => return Err(ConfigError::Missing("GOOGLE_CLIENT_ID")),
            (_, None, _) => return Err(ConfigError::Missing("GOOGLE_CLIENT_SECRET")),
            (_, _, None) => return Err(ConfigError::Missing("GOOGLE_REDIRECT_URI")),
        };

        Ok(Self {
            database_url,
            bind_addr,
            frontend_origin,
            compiler_enabled,
            compiler_path,
            compile_timeout,
            session_ttl,
            cookie_secure,
            google,
        })
    }

    pub fn origin_is_allowed(&self, origin: &HeaderValue) -> bool {
        origin == &self.frontend_origin
    }
}

fn parse<T>(name: &'static str, value: String) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    value.parse().map_err(|source| ConfigError::Invalid {
        name,
        source: Box::new(source),
    })
}

fn parse_compile_timeout(value: String) -> Result<Duration, ConfigError> {
    let seconds = parse("COMPILE_TIMEOUT_SECONDS", value)?;
    if !(MIN_COMPILE_TIMEOUT_SECONDS..=MAX_COMPILE_TIMEOUT_SECONDS).contains(&seconds) {
        return Err(ConfigError::OutOfRange {
            name: "COMPILE_TIMEOUT_SECONDS",
            min: MIN_COMPILE_TIMEOUT_SECONDS,
            max: MAX_COMPILE_TIMEOUT_SECONDS,
        });
    }
    Ok(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {
    use super::parse_compile_timeout;

    #[test]
    fn compile_timeout_preserves_stale_recovery_grace() {
        assert_eq!(parse_compile_timeout("90".into()).unwrap().as_secs(), 90);
        assert!(parse_compile_timeout("0".into()).is_err());
        assert!(parse_compile_timeout("91".into()).is_err());
    }
}
