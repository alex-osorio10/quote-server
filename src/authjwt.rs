use crate::AppState;
use axum::{
    extract::{FromRequestParts, FromRef}, http::{request::Parts, StatusCode}, response::{IntoResponse, Json, Response}, RequestPartsExt,
};

use axum_extra::{
    headers::{authorization::Bearer, Authorization}, TypedHeader,
};

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::ToSchema;


pub struct JwtKeys {
    pub encoding: EncodingKey, pub decoding: DecodingKey,
}

impl JwtKeys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }


    }


}

pub async fn read_secret(env_var: &str, default_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let secret_file_path = std::env::var(env_var).unwrap_or_else(|_| default_path.to_owned());


    let secret = tokio::fs::read_to_string(secret_file_path).await?;
    Ok(secret.trim().to_string())

}

pub async fn make_jwt_keys() -> Result<JwtKeys, Box<dyn std::error::Error>> {
    let secret = read_secret("JWT_SECRETFILE", "secrets/jwt_secret.txt").await?;
    Ok(JwtKeys::new(secret.as_bytes()))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Claims {
    #[schema(example = "quote-server.example.com")]
    pub iss: String,
    #[schema(example = "Alex Osorio Trujillo <alex@example.com>")]
    pub sub: String,
    #[schema(example = json!(Utc::now().timestamp() + 3600))]
    pub exp: i64,


}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthBody {
    access_token: String,
    token_type: String,
}


impl AuthBody {
    fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }


    }

}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct Registration {
    #[schema(example = "Alex Osorio Trujillo")]
    pub full_name: String,
    #[schema(example = "alex@example.com")]
    pub email: String,
    #[schema(example = "some-secret-password")]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Internal error: Failed to create token")]
    TokenCreation,
    #[error("Invalid registration key")]
    InvalidRegistrationKey,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid authentication token."),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error creating token."),
            AuthError::InvalidRegistrationKey => (StatusCode::UNAUTHORIZED, "Invalid registration key provided."),
        };
        let body = Json(serde_json::json!({ "error": error_message }));
        (status, body).into_response()
    }


}

impl<S> FromRequestParts<S> for Claims
where
    Arc<RwLock<AppState>>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let app_state = Arc::<RwLock<AppState>>::from_ref(state);

        let app_state_reader = app_state.read().await;
        let decoding_key = &app_state_reader.jwt_keys.decoding;
        let validation = Validation::new(jsonwebtoken::Algorithm::HS512);

        let token_data = decode::<Claims>(bearer.token(), decoding_key, &validation)
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }


}


pub fn register_and_create_token(
    app_state: &AppState,
    registration: &Registration,
) -> Result<impl IntoResponse, AuthError> {
    if registration.password != app_state.reg_key {
        return Err(AuthError::InvalidRegistrationKey);
    }

    let claims = Claims {
        iss: "quote-server.example.com".to_owned(),
        sub: format!("{} <{}>", registration.full_name, registration.email),
        exp: (Utc::now() + TimeDelta::days(1)).timestamp(),


    };

    let header = Header::new(jsonwebtoken::Algorithm::HS512);
    let token = encode(&header, &claims, &app_state.jwt_keys.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    let response_body = AuthBody::new(token);

    
    Ok((StatusCode::OK, Json(response_body)))



}
