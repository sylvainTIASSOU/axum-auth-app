/*
Understanding Token Details

The TokenDetails struct serves as a bridge between our JWT implementation and Redis storage.
For refresh tokens, which need to persist across sessions,
 we can store the TokenDetails in Redis using the token_id as the key. This allows us to:

    Track Active Sessions: Store long-lived refresh tokens to track user sessions.
    Enable Token Revocation: Delete a token from Redis to immediately invalidate it, even before expiration.
    Link Tokens to Users: Use the user_pid to find all active tokens for a specific user.
    Automatic Cleanup: expires_in lets us implement Redis TTL (time-to-live).

*/

use serde::{Deserialize, Serialize};
use uuid::Uuid;



/// The token string deserialises to this struct
/// The `sub` field will be the user's pid
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenClaims {
    pub sub: String,
    pub id: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
}


/// This struct will let us store our token in Redis
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenDetails {
    pub token: Option<String>,
    pub token_id: Uuid,
    pub user_pid: Uuid,
    pub expires_in: Option<i64>
}