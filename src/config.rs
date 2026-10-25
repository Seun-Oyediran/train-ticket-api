use std::env;

use chrono_tz::Tz;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_refresh_secret: String,

    pub access_token_ttl_minutes: i64,
    pub refresh_token_ttl_days: i64,
    pub max_booking_days: u8,
    pub train_timezone: Tz,
}

impl Config {
    pub fn new() -> Result<Self, String> {
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|e| format!("PORT must be a valid port number: {}", e))?;

        let access_token_ttl_minutes = env::var("ACCESS_TOKEN_TTL_MINUTES")
            .unwrap_or_else(|_| "30".to_string())
            .parse::<i64>()
            .map_err(|e| format!("ACCESS_TOKEN_TTL_MINUTES must be a valid number: {}", e))?;

        if access_token_ttl_minutes < 1 {
            return Err("ACCESS_TOKEN_TTL_MINUTES must be greater than 0".to_string());
        }

        let refresh_token_ttl_days = env::var("REFRESH_TOKEN_TTL_DAYS")
            .unwrap_or_else(|_| "14".to_string())
            .parse::<i64>()
            .map_err(|e| format!("REFRESH_TOKEN_TTL_DAYS must be a valid number: {}", e))?;

        if refresh_token_ttl_days < 1 {
            return Err("REFRESH_TOKEN_TTL_DAYS must be greater than 0".to_string());
        }

        let database_url =
            env::var("DATABASE_URL").map_err(|_| "Database url is compulsory".to_string())?;

        let jwt_secret =
            env::var("JWT_SECRET").map_err(|_| "Jwt secret is compulsory".to_string())?;
        let jwt_refresh_secret = env::var("JWT_REFRESH_SECRET")
            .map_err(|_| "Jwt refresh secret is compulsory".to_string())?;

        let max_booking_days = env::var("MAX_BOOKING_DAYS")
            .unwrap_or_else(|_| "3".to_string())
            .parse::<u8>()
            .map_err(|e| format!("MAX_BOOKING_DAYS must be a valid u8 number: {}", e))?;

        let train_timezone = env::var("TRAIN_TIMEZONE")
            .map_err(|_| "Train timezone is compulsory".to_string())?
            .parse::<Tz>()
            .map_err(|e| format!("Train timezone must be a valid timezone: {}", e))?;

        Ok(Self {
            port,
            database_url,
            jwt_secret,
            jwt_refresh_secret,
            access_token_ttl_minutes,
            refresh_token_ttl_days,
            max_booking_days,
            train_timezone,
        })
    }
}
