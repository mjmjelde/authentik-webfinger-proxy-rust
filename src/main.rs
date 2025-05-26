use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::env;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

#[derive(Serialize)]
struct WebFingerResponse {
    subject: String,
    links: Vec<Link>,
}

#[derive(Serialize)]
struct Link {
    rel: String,
    href: String,
}

#[derive(Deserialize)]
struct WebFingerQuery {
    resource: Option<String>,
}

async fn webfinger_handler(Query(params): Query<WebFingerQuery>) -> Result<Json<WebFingerResponse>, StatusCode> {
    let resource = match params.resource {
        Some(res) if res.starts_with("acct:") => res,
        Some(_) => {
            warn!("Invalid resource format, must start with 'acct:'");
            return Err(StatusCode::BAD_REQUEST);
        }
        None => {
            warn!("Missing resource parameter");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let domain = env::var("DOMAIN").unwrap_or_else(|_| "idp.example.com".to_string());
    let issuer_url = format!("https://{}/application/o/tailscale/", domain);

    let response = WebFingerResponse {
        subject: resource,
        links: vec![
            Link {
                rel: "http://openid.net/specs/connect/1.0/issuer".to_string(),
                href: issuer_url.clone(),
            },
            Link {
                rel: "authorization_endpoint".to_string(),
                href: format!("{}oauth2/authorize", issuer_url),
            },
            Link {
                rel: "token_endpoint".to_string(),
                href: format!("{}oauth2/token", issuer_url),
            },
            Link {
                rel: "userinfo_endpoint".to_string(),
                href: format!("{}userinfo", issuer_url),
            },
            Link {
                rel: "jwks_uri".to_string(),
                href: format!("{}jwks", issuer_url),
            },
        ],
    };

    info!("WebFinger request processed for resource: {}", response.subject);
    Ok(Json(response))
}

async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "authentik_webfinger_proxy=info,tower_http=debug".into()),
        )
        .init();

    let domain = env::var("DOMAIN").unwrap_or_else(|_| "idp.example.com".to_string());
    info!("Starting Authentik WebFinger Proxy with domain: {}", domain);

    // Build our application with routes
    let app = Router::new()
        .route("/.well-known/webfinger", get(webfinger_handler))
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive());

    let port = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .unwrap_or(8000);

    let addr = format!("0.0.0.0:{}", port);
    info!("Server starting on {}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}