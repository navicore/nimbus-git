//! Authentication for Nimbus
//!
//! Uses Kubernetes secrets for stateless auth management

use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use kube::{Api, Client};
use k8s_openapi::api::core::v1::Secret;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct AuthService {
    jwt_secret: String,
    kube_client: Option<Client>,
    namespace: String,
}

impl std::fmt::Debug for AuthService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthService")
            .field("namespace", &self.namespace)
            .field("has_kube_client", &self.kube_client.is_some())
            .finish()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // Subject (user ID)
    pub exp: usize,   // Expiry time
    pub iat: usize,   // Issued at
    pub role: String, // User role (owner, viewer)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiToken {
    pub id: String,
    pub name: String,
    pub token: String,
    pub created_at: usize,
    pub expires_at: Option<usize>,
}

impl AuthService {
    pub async fn new() -> Self {
        // Try to create Kubernetes client (will fail in local dev)
        let kube_client = Client::try_default().await.ok();
        
        // Get namespace from env or default
        let namespace = std::env::var("NIMBUS_NAMESPACE")
            .unwrap_or_else(|_| "nimbus".to_string());
        
        // Try to load JWT secret from K8s, fallback to env/default
        let jwt_secret = if let Some(client) = &kube_client {
            Self::load_jwt_secret(client, &namespace).await
                .unwrap_or_else(|_| Self::default_jwt_secret())
        } else {
            Self::default_jwt_secret()
        };
        
        Self { 
            jwt_secret,
            kube_client,
            namespace,
        }
    }
    
    fn default_jwt_secret() -> String {
        std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "development-secret-change-in-production".to_string())
    }
    
    async fn load_jwt_secret(client: &Client, namespace: &str) -> Result<String, kube::Error> {
        let secrets: Api<Secret> = Api::namespaced(client.clone(), namespace);
        let secret = secrets.get("nimbus-jwt-secret").await?;
        
        if let Some(data) = secret.data {
            if let Some(secret_bytes) = data.get("secret") {
                let decoded = BASE64.decode(&secret_bytes.0)
                    .map_err(|e| kube::Error::Api(kube::error::ErrorResponse {
                        status: "400".to_string(),
                        message: format!("Failed to decode secret: {}", e),
                        reason: "BadRequest".to_string(),
                        code: 400,
                    }))?;
                return Ok(String::from_utf8_lossy(&decoded).to_string());
            }
        }
        
        Err(kube::Error::Api(kube::error::ErrorResponse {
            status: "404".to_string(),
            message: "JWT secret not found in secret data".to_string(),
            reason: "NotFound".to_string(),
            code: 404,
        }))
    }
    
    pub async fn validate_owner_login(&self, username: &str, password: &str) -> Result<bool, String> {
        // In production, check against K8s secret
        if let Some(client) = &self.kube_client {
            let secrets: Api<Secret> = Api::namespaced(client.clone(), &self.namespace);
            
            match secrets.get("nimbus-owner").await {
                Ok(secret) => {
                    if let Some(data) = secret.data {
                        // Check username
                        if let Some(stored_username) = data.get("username") {
                            let decoded_username = BASE64.decode(&stored_username.0)
                                .map_err(|e| format!("Failed to decode username: {}", e))?;
                            let stored_username_str = String::from_utf8_lossy(&decoded_username);
                            
                            if stored_username_str != username {
                                return Ok(false);
                            }
                        }
                        
                        // Check password hash
                        if let Some(stored_hash) = data.get("password_hash") {
                            let decoded_hash = BASE64.decode(&stored_hash.0)
                                .map_err(|e| format!("Failed to decode password hash: {}", e))?;
                            let hash_str = String::from_utf8_lossy(&decoded_hash);
                            
                            if !hash_str.is_empty() {
                                return self.verify_password(password, &hash_str)
                                    .map_err(|e| format!("Password verification failed: {}", e));
                            }
                        }
                        
                        // If no password hash stored, this is first login
                        // Store the hash for future use
                        if username == "admin" {
                            // In a real system, we'd update the secret here
                            // For now, accept any password on first login
                            return Ok(true);
                        }
                    }
                }
                Err(e) => {
                    return Err(format!("Failed to access owner secret: {}", e));
                }
            }
        }
        
        // Fallback for local development
        Ok(username == "admin" && password == "admin")
    }

    pub fn hash_password(&self, password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        Ok(password_hash.to_string())
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
        let parsed_hash = PasswordHash::new(hash)?;
        let argon2 = Argon2::default();
        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub fn generate_token(&self, user_id: &str, role: &str) -> Result<String, jsonwebtoken::errors::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as usize;
        
        let claims = Claims {
            sub: user_id.to_string(),
            exp: now + 86400, // 24 hours
            iat: now,
            role: role.to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        ).map(|data| data.claims)
    }

    pub fn generate_api_key(&self) -> String {
        format!("nmbs_{}", Uuid::new_v4().to_string().replace("-", ""))
    }
    
    pub async fn store_api_token(&self, name: &str, token: &str) -> Result<(), String> {
        if let Some(client) = &self.kube_client {
            let secrets: Api<Secret> = Api::namespaced(client.clone(), &self.namespace);
            
            // Create secret name
            let secret_name = format!("nimbus-token-{}", name.to_lowercase().replace(" ", "-"));
            
            // Create secret data
            let mut data = BTreeMap::new();
            data.insert("token".to_string(), k8s_openapi::ByteString(token.as_bytes().to_vec()));
            data.insert("name".to_string(), k8s_openapi::ByteString(name.as_bytes().to_vec()));
            data.insert("created_at".to_string(), k8s_openapi::ByteString(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string()
                    .as_bytes()
                    .to_vec()
            ));
            
            let secret = Secret {
                metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                    name: Some(secret_name),
                    namespace: Some(self.namespace.clone()),
                    labels: Some({
                        let mut labels = BTreeMap::new();
                        labels.insert("app".to_string(), "nimbus".to_string());
                        labels.insert("type".to_string(), "api-token".to_string());
                        labels
                    }),
                    ..Default::default()
                },
                data: Some(data),
                ..Default::default()
            };
            
            secrets.create(&Default::default(), &secret).await
                .map_err(|e| format!("Failed to store API token: {}", e))?;
            
            Ok(())
        } else {
            Err("Kubernetes client not available".to_string())
        }
    }
    
    pub async fn list_api_tokens(&self) -> Result<Vec<ApiToken>, String> {
        if let Some(client) = &self.kube_client {
            let secrets: Api<Secret> = Api::namespaced(client.clone(), &self.namespace);
            
            // List secrets with label selector
            let params = kube::api::ListParams::default()
                .labels("type=api-token");
            
            let secret_list = secrets.list(&params).await
                .map_err(|e| format!("Failed to list API tokens: {}", e))?;
            
            let mut tokens = Vec::new();
            for secret in secret_list.items {
                if let Some(data) = secret.data {
                    if let (Some(token_bytes), Some(name_bytes), Some(created_bytes)) = 
                        (data.get("token"), data.get("name"), data.get("created_at")) {
                        
                        let token = String::from_utf8_lossy(&token_bytes.0).to_string();
                        let name = String::from_utf8_lossy(&name_bytes.0).to_string();
                        let created_at = String::from_utf8_lossy(&created_bytes.0)
                            .parse::<usize>()
                            .unwrap_or(0);
                        
                        tokens.push(ApiToken {
                            id: secret.metadata.name.unwrap_or_default(),
                            name,
                            token: format!("{}...", &token[..8.min(token.len())]), // Only show prefix
                            created_at,
                            expires_at: None,
                        });
                    }
                }
            }
            
            Ok(tokens)
        } else {
            Ok(Vec::new()) // Return empty list in dev mode
        }
    }
}

impl Default for AuthService {
    fn default() -> Self {
        // Block on async new() - not ideal but works for now
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Self::new())
    }
}