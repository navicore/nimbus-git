use nimbus_auth::AuthService;
use nimbus_events::InMemoryEventBus as EventBus;
use std::sync::Arc;
use tracing::info;
use warp::Filter;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Nimbus Git Platform");

    // Initialize services
    let _event_bus = Arc::new(EventBus::new(1000)); // 1000 event buffer size
    let auth_service = Arc::new(AuthService::new().await);

    // Health check endpoint
    let health = warp::path("health").map(|| {
        warp::reply::json(&serde_json::json!({
            "status": "healthy",
            "service": "nimbus-web",
            "version": env!("CARGO_PKG_VERSION")
        }))
    });

    // Auth endpoints
    let auth_routes = warp::path("api").and(warp::path("auth")).and(
        register_route(auth_service.clone())
            .or(login_route(auth_service.clone()))
            .or(logout_route(auth_service.clone()))
            .or(create_token_route(auth_service.clone()))
            .or(list_tokens_route(auth_service.clone())),
    );

    // Combine all routes
    let routes = health.or(auth_routes).with(warp::cors().allow_any_origin());

    let port = std::env::var("NIMBUS_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("Invalid port number");

    let host = std::env::var("NIMBUS_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let addr: std::net::SocketAddr = format!("{}:{}", host, port).parse().expect("Invalid address");

    info!("Nimbus server listening on http://{}", addr);

    warp::serve(routes).run(addr).await;
}

// Auth route handlers
fn register_route(
    auth_service: Arc<AuthService>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("register")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_auth_service(auth_service))
        .and_then(handle_register)
}

fn login_route(
    auth_service: Arc<AuthService>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("login")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_auth_service(auth_service))
        .and_then(handle_login)
}

fn logout_route(
    auth_service: Arc<AuthService>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("logout")
        .and(warp::post())
        .and(warp::header::optional::<String>("authorization"))
        .and(with_auth_service(auth_service))
        .and_then(handle_logout)
}

fn with_auth_service(
    auth_service: Arc<AuthService>,
) -> impl Filter<Extract = (Arc<AuthService>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || auth_service.clone())
}

async fn handle_register(
    body: serde_json::Value,
    _auth_service: Arc<AuthService>,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Register request: {:?}", body);

    // TODO: Implement actual registration
    Ok(warp::reply::json(&serde_json::json!({
        "message": "Registration endpoint - not yet implemented",
        "user": body.get("username")
    })))
}

async fn handle_login(
    body: serde_json::Value,
    auth_service: Arc<AuthService>,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Login request: {:?}", body);

    let username = body.get("username")
        .and_then(|v| v.as_str())
        .ok_or_else(|| warp::reject::reject())?;
    
    let password = body.get("password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| warp::reject::reject())?;

    // Validate login
    match auth_service.validate_owner_login(username, password).await {
        Ok(true) => {
            // Generate JWT token
            match auth_service.generate_token(username, "owner") {
                Ok(token) => {
                    Ok(warp::reply::json(&serde_json::json!({
                        "success": true,
                        "token": token,
                        "user": username,
                        "role": "owner"
                    })))
                }
                Err(e) => {
                    info!("Failed to generate token: {}", e);
                    Ok(warp::reply::json(&serde_json::json!({
                        "success": false,
                        "error": "Failed to generate token"
                    })))
                }
            }
        }
        Ok(false) => {
            Ok(warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Invalid credentials"
            })))
        }
        Err(e) => {
            info!("Login error: {}", e);
            Ok(warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Authentication service error"
            })))
        }
    }
}

async fn handle_logout(
    auth_header: Option<String>,
    _auth_service: Arc<AuthService>,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Logout request with auth: {:?}", auth_header);

    // TODO: Implement actual logout
    Ok(warp::reply::json(&serde_json::json!({
        "message": "Logout successful"
    })))
}

fn create_token_route(
    auth_service: Arc<AuthService>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("tokens")
        .and(warp::post())
        .and(warp::header::optional::<String>("authorization"))
        .and(warp::body::json())
        .and(with_auth_service(auth_service.clone()))
        .and_then(handle_create_token)
}

fn list_tokens_route(
    auth_service: Arc<AuthService>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("tokens")
        .and(warp::get())
        .and(warp::header::optional::<String>("authorization"))
        .and(with_auth_service(auth_service.clone()))
        .and_then(handle_list_tokens)
}

async fn handle_create_token(
    _auth_header: Option<String>,
    body: serde_json::Value,
    auth_service: Arc<AuthService>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let name = body.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| warp::reject::reject())?;

    let token = auth_service.generate_api_key();
    
    match auth_service.store_api_token(name, &token).await {
        Ok(_) => {
            Ok(warp::reply::json(&serde_json::json!({
                "success": true,
                "name": name,
                "token": token
            })))
        }
        Err(e) => {
            info!("Failed to store API token: {}", e);
            Ok(warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Failed to create token"
            })))
        }
    }
}

async fn handle_list_tokens(
    _auth_header: Option<String>,
    auth_service: Arc<AuthService>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match auth_service.list_api_tokens().await {
        Ok(tokens) => {
            Ok(warp::reply::json(&serde_json::json!({
                "success": true,
                "tokens": tokens
            })))
        }
        Err(e) => {
            info!("Failed to list API tokens: {}", e);
            Ok(warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Failed to list tokens"
            })))
        }
    }
}
