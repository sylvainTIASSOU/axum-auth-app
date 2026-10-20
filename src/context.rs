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

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use redis::{AsyncTypedCommands, aio::MultiplexedConnection};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    config::{Config, RsaJwtConfig},
    error::Report,
};
use crate::models::token::{TokenClaims, TokenDetails};

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
    pub exp: i64,
}


/*
Note on implementation: I converted the try_from method from a TryFrom<T>
 trait implementation to a regular async method. The trait version caused
 runtime panics because block_on was being called in an async context during application startup.

Here's how token storage works:

    We serialize the entire TokenDetails struct to JSON and store it as a Redis string
    The set_ex method automatically expires the token after its TTL (time-to-live),
    ensuring Redis cleans up stale tokens
    The set method is a fallback that handles tokens without expiration
    (though this should rarely occur in practice)
    For explicit logout, we use revoke_refresh_token to immediately delete the token from Redis

*/
impl AppContext {
    pub async fn store_refresh_token(&self, token_details: &TokenDetails) -> Result<(), Report> {
        let mut conn = self.redis.clone();
        let key = format!("refresh_token:{}", token_details.token_id);
        let value = serde_json::to_string(token_details)?;

        if let Some(expires_in) = token_details.expires_in {
            let ttl = (expires_in - chrono::Utc::now().timestamp()) as u64;
            conn.set_ex(&key, &value, ttl).await?;
        } else {
            conn.set(&key, &value).await?;
        }

        Ok(())
    }

    pub async fn revoke_refresh_token(&self, token_id: Uuid) -> Result<(), Report> {
        let mut conn = self.redis.clone();
        let key = format!("refresh_token:{}", token_id);

        conn.del(&key).await?;

        Ok(())
    }

    pub async fn try_from(config: &Config) -> Result<Self, Report> {
        let db = config.database().pool().await;
        let redis = config.redis().multiplexed_connection().await?;

        let auth = AuthContext {
            access: config.auth().access().try_into()?,
            refresh: config.auth().refresh().try_into()?,
        };

        Ok(Self {
            redis,
            db,
            auth,
            config: config.clone(),
        })
    }
}
/*
Understanding Generating & Verifying Tokens.

The generate_token method creates a new JWT by initialising token metadata
(TokenDetails) and building JWT claims (TokenClaims).
You can read more about claims here. Then we create a token,
which is a base64 string, using the encoding key (private key).

The verify_token method is pretty simple; we use the decoding key
(public key) to decode the string and then deserialise it to the TokenClaims struct.
But since the user_pid and token_id are of type Uuid we parse them to that type.
*/

impl JwtContext {
    pub fn generate_token(&self, sub: Uuid) -> Result<TokenDetails, Report> {
        let now = chrono::Utc::now();
        let mut token_details = TokenDetails{
            token: None,
            user_pid: sub,
            token_id: Uuid::new_v4(),
            expires_in: Some((now + chrono::Duration::seconds(self.exp)).timestamp())
        };
        let claims = TokenClaims {
            sub: token_details.user_pid.to_string(),
            id: token_details.token_id.to_string(),
            exp: token_details.expires_in.ok_or(crate::Error::TokenError)?,
            iat: now.timestamp(),
            nbf: now.timestamp(),
        };

        let header = Header::new(Algorithm::RS256);
        let token = jsonwebtoken::encode(&header, &claims, &self.encoding_key)?;

        token_details.token = Some(token);

        Ok(token_details)
    }

    pub fn verify_token(&self, token: &str) -> Result<TokenDetails, Report> {
        let validation = Validation::new(Algorithm::RS256);
        let token_data = jsonwebtoken::decode::<TokenClaims>(token,&self.decoding_key, &validation)?;

        let user_pid = Uuid::parse_str(&token_data.claims.sub)?;
        let token_id = Uuid::parse_str(&token_data.claims.id)?;

        Ok(TokenDetails {
            token: None,
            token_id,
            user_pid,
            expires_in: None,
        })
    }
}

impl TryFrom<&RsaJwtConfig> for JwtContext {
    type Error = Report;

    fn try_from(config: &RsaJwtConfig) -> Result<Self, Self::Error> {
        let encoding_key = config.encoding_key()?;
        let decoding_key = config.decoding_key()?;

        let exp = config.exp();

        Ok(Self {
            encoding_key,
            decoding_key,
            exp,
        })
    }
}