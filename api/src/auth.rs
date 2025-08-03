use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use headers::{Authorization, HeaderMapExt};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct AuthError {
    pub error: String,
    pub message: String,
}

pub async fn auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<AuthError>)> {
    // Extract Authorization header
    let auth_header = headers.get("authorization");
    
    if let Some(auth_value) = auth_header {
        let auth_str = auth_value.to_str().map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(AuthError {
                    error: "invalid_header".to_string(),
                    message: "Invalid authorization header format".to_string(),
                }),
            )
        })?;

        // Check if it starts with "Bearer "
        if auth_str.starts_with("Bearer ") {
            let token = &auth_str[7..]; // Remove "Bearer " prefix
            
            // Simple token validation - just check if token exists and is not empty
            // In a real application, you would validate the JWT token here
            if !token.is_empty() && token.len() > 10 {
                // Token is present and has reasonable length
                log::info!("Authentication successful for token: {}...{}", &token[..4], &token[token.len()-4..]);
                let response = next.run(request).await;
                return Ok(response);
            } else {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(AuthError {
                        error: "invalid_token".to_string(),
                        message: "Token is too short or invalid".to_string(),
                    }),
                ));
            }
        } else {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(AuthError {
                    error: "invalid_authorization".to_string(),
                    message: "Authorization header must start with 'Bearer '".to_string(),
                }),
            ));
        }
    } else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthError {
                error: "missing_authorization".to_string(),
                message: "Authorization header is required".to_string(),
            }),
        ));
    }
}

// Alternative implementation using axum-extra typed headers
pub async fn auth_middleware_typed(
    auth: Option<headers::Authorization<headers::authorization::Bearer>>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<AuthError>)> {
    if let Some(auth) = auth {
        let token = auth.token();
        
        // Simple token validation - just check if token exists and is not empty
        if !token.is_empty() && token.len() > 10 {
            log::info!("Authentication successful for token: {}...{}", &token[..4], &token[token.len()-4..]);
            let response = next.run(request).await;
            return Ok(response);
        } else {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(AuthError {
                    error: "invalid_token".to_string(),
                    message: "Token is too short or invalid".to_string(),
                }),
            ));
        }
    } else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthError {
                error: "missing_authorization".to_string(),
                message: "Authorization Bearer token is required".to_string(),
            }),
        ));
    }
}

// Example of what a real JWT validation might look like (commented out since we don't have JWT dependencies)
/*
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn validate_jwt_token(token: &str, secret: &str) -> Result<Claims, String> {
    let key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::new(Algorithm::HS256);
    
    match decode::<Claims>(token, &key, &validation) {
        Ok(token_data) => Ok(token_data.claims),
        Err(err) => Err(format!("JWT validation failed: {}", err)),
    }
}
*/

// Generate a simple mock token for testing
pub fn generate_mock_token(user_id: &str) -> String {
    format!("mock_token_{}_{}", user_id, uuid::Uuid::new_v4())
}

// Mock token validation that just checks format
pub fn validate_mock_token(token: &str) -> bool {
    token.starts_with("mock_token_") && token.len() > 20
}
