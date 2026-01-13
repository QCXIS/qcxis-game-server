use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::env;
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub user_id: String,
    pub username: String,
    pub exp: usize,
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    
    // Handle Laravel's base64: prefix
    let secret_bytes = if secret.starts_with("base64:") {
        match general_purpose::STANDARD.decode(&secret[7..]) {
            Ok(bytes) => bytes,
            Err(_) => secret.as_bytes().to_vec(),
        }
    } else {
        secret.as_bytes().to_vec()
    };
    
    let validation = Validation::new(Algorithm::HS256);
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&secret_bytes),
        &validation,
    )?;
    
    Ok(token_data.claims)
}
