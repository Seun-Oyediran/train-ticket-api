use chrono::{NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode,
    errors::Error as JsonWebTokenError,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::user::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
    pub jti: Uuid,
}

impl Claims {
    pub fn new(sub: Uuid, email: String, duration: TimeDelta) -> Self {
        let current_utc = Utc::now();
        let exp = (current_utc + duration).timestamp() as usize;
        let iat = (current_utc).timestamp() as usize;

        Self {
            sub,
            email,
            exp,
            iat,
            jti: Uuid::new_v4(),
        }
    }

    pub fn encode_jwt(&self, key: &EncodingKey) -> Result<String, JsonWebTokenError> {
        encode(&Header::default(), &self, key)
    }

    pub fn decode_jwt(
        token: &str,
        key: &DecodingKey,
    ) -> Result<TokenData<Claims>, JsonWebTokenError> {
        let token_message = decode::<Claims>(&token, key, &Validation::new(Algorithm::HS256))?;

        Ok(token_message)
    }

    pub fn generate_response(
        access_claim: &Claims,
        refresh_claim: &Claims,
        access_key: &EncodingKey,
        refresh_key: &EncodingKey,
    ) -> Result<Token, JsonWebTokenError> {
        let access_token = access_claim.encode_jwt(access_key)?;
        let refresh_token = refresh_claim.encode_jwt(refresh_key)?;

        Ok(Token {
            access_token,
            refresh_token,
            access_expires_at: access_claim.exp,
            refresh_expires_at: refresh_claim.exp,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    access_token: String,
    refresh_token: String,
    access_expires_at: usize,
    refresh_expires_at: usize,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    token: Token,
    user: User,
}

impl LoginResponse {
    pub fn new(user: User, token: Token) -> Self {
        Self { token, user }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginPayload {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    Male,
    Female,
}

impl Gender {
    pub fn as_str(&self) -> &'static str {
        match self {
            Gender::Male => "male",
            Gender::Female => "female",
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAccountPayload {
    pub first_name: String,
    pub last_name: String,
    #[validate(email)]
    pub email: String,
    pub phone_number: String,
    #[validate(length(min = 6, message = "password must be at least 6 characters long"))]
    pub password: String,
    pub date_of_birth: NaiveDate,
    pub gender: Gender,
    pub verification_number: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenPayload {
    pub refresh_token: String,
}
