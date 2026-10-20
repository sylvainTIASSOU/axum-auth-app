/*
Understanding the AppContext.

Why Clone Trait? - Modern Rust web frameworks (like Axum or Actix)
pass state to handlers by cloning. These types use Arc internally,
making clones cheap (just incrementing the reference count rather
than duplicating connections).

You might wonder why we use Handle::current().block_on() from tokio.
 That is because TryFrom is a synchronous trait but initialising the
 Database and Redis connections is asynchronous in nature.
 This conversion typically occurs once during application startup,
  before the async runtime begins serving requests. The one-time blocking
cost is acceptable because; it only happens at startup (not per-request).
*/

use jsonwebtoken::{DecodingKey, EncodingKey};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;

use crate::{
    config::{Config, RsaJwtConfig},
    error::Report,
};

#[derive(Clone)]
pub struct AppContext {
    pub config: Config,
    pub auth: AuthContext,
    pub db: PgPool,
    pub redis: MultiplexedConnection,
}

impl TryFrom<&Config> for AppContext {
    type Error = Report;

    fn try_from(config: &Config) -> Result<Self, Self::Error> {
        let db =
            tokio::runtime::Handle::current().block_on(async { config.database().pool().await });

        let auth = AuthContext {
            access: config.auth().access().try_into()?,
            refresh: config.auth().refresh().try_into()?,
        };
        let redis = tokio::runtime::Handle::current()
            .block_on(async { config.redis().multiplexed_connection().await })?;

        Ok(Self {
            redis,
            db,
            auth,
            config: config.clone(),
        })
    }
}

#[derive(Clone)]
pub struct AuthContext {
    pub access: JwtContext,
    pub refresh: JwtContext,
}

#[derive(Clone)]
pub struct JwtContext {
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
    pub exp: u64,
}

impl TryFrom<&RsaJwtConfig> for JwtContext {
    type Error = Report;

    fn try_from(config: &RsaJwtConfig) -> Result<Self, Self::Error> {
        let encoding_key = config.encoding_key()?;
        let decoding_key = config.decoding_key()?;

        let exp = config.exp() as u64;

        Ok(Self {
            encoding_key,
            decoding_key,
            exp,
        })
    }
}