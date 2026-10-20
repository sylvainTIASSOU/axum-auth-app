/*
A note on Cow<'a, str>: You'll notice we use Cow<'a, str> instead of
String for the request structs. Cow (Clone on Write) is more
memory-efficient because it can hold either a borrowed reference or an owned string.
 For deserialized JSON data that we only read from, this avoids unnecessary heap allocations.
 The actual User struct from the database uses owned String types since that data persists
 beyond the request lifecycle.

We use the argon2 crate for secure password hashing with randomly generated salts.
 The verify_password method compares submitted passwords against the stored hash,
 converting any password mismatch into our custom InvalidCredentials error.
*/

use std::borrow::Cow;

use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher},
};
use argon2::password_hash::phc::SaltString;
use chrono::{
    DateTime, FixedOffset,
    format::{DelayedFormat, StrftimeItems},
};
use jsonwebtoken::signature::rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::{Encode, Executor, Postgres, prelude::FromRow};
use uuid::Uuid;
use crate::models::ModelError;
use crate::Result;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterUser<'a> {
    email: Cow<'a, str>,
    name: Cow<'a, str>,
    password: Cow<'a, str>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoginUser<'a> {
    email: Cow<'a, str>,
    password: Cow<'a, str>,
}

impl LoginUser<'_> {
    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

#[derive(Debug, Deserialize, Clone, FromRow, Encode)]
pub struct User {
    id: i32,
    pid: Uuid,
    email: String,
    name: String,
    password: String,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

impl User {
    pub async fn create_user<'e, C>(db: &C, new_user: &RegisterUser<'_>) -> Result<Self>
    where
            for<'a> &'a C: Executor<'e, Database = Postgres>,
    {
        let user = sqlx::query_as::<_, Self>(
            r"
           INSERT INTO users (email, name, password)
           VALUES ($1, $2, $3)
           RETURNING *
           ",
        )
            .bind(new_user.email.trim())
            .bind(new_user.name.trim())
            .bind(password_hash(&new_user.password)?)
            .fetch_one(db)
            .await?;
        Ok(user)
    }

    pub async fn find_by_email<'e, C>(db: &C, email: &str) -> Result<Option<Self>>
    where
            for<'a> &'a C: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as(
            r"
            SELECT * FROM users WHERE email = $1
        ",
        )
            .bind(email.trim())
            .fetch_optional(db)
            .await
            .map_err(Into::into)
    }

    pub async fn find_by_pid<'e, C>(db: &C, pid: Uuid) -> Result<Self>
    where
            for<'a> &'a C: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as(
            r"
            SELECT * FROM users WHERE pid = $1
        ",
        )
            .bind(pid)
            .fetch_optional(db)
            .await?
            .ok_or(crate::Error::Model(ModelError::EntityNotFound).into())
    }

    pub fn verify_password(&self, password: &str) -> Result<()> {
        let password_hash =
            PasswordHash::new(&self.password).map_err(|err| crate::Error::PasswordHash(err.into()))?;
        //crate::Error::PasswordHash

        Argon2::default()
            .verify_password(password.as_bytes(), &password_hash)
            .map_err(|err| match err {
                argon2::password_hash::Error::PasswordInvalid => crate::Error::InvalidCredentials,
                _ => crate::Error::PasswordHash(err),
            })?;

        Ok(())
    }

    pub fn pid(&self) -> Uuid {
        self.pid
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn created_at(&self) -> DelayedFormat<StrftimeItems<'_>> {
        self.created_at.format("%Y-%m-%d %H:%M")
    }
}

fn password_hash(plain_password: &str) -> Result<String> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate();

    let hash = argon2
        .hash_password(plain_password.as_bytes())
        .map_err(|err| crate::Error::PasswordHash(err.into()))?;

    Ok(hash.to_string())
}
