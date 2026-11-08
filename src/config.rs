use std::fs;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub host: String,
    pub port: u16,
    pub default_group_id: String,
}

impl Config {
    pub fn from_env() -> Self {
        let db_pass =
            read_secret("db_pass", "POSTGRES_PASS").unwrap_or_else(|| "admin".to_string());
        let jwt_secret = read_secret("jwt_secret", "JWT_SECRET")
            .unwrap_or_else(|| "default-jwt-secret-key-32-chars-long!".to_string());

        let database_url = if let Ok(url) = std::env::var("DATABASE_URL") {
            url
        } else if let Ok(jdbc) = std::env::var("POSTGRES_JDBC_URL") {
            // Convert jdbc:postgresql://host:port/db to postgresql://user:pass@host:port/db
            let user = std::env::var("POSTGRES_USER").unwrap_or_else(|_| "admin".to_string());
            let stripped = jdbc.trim_start_matches("jdbc:postgresql://");
            let clean = stripped.split('?').next().unwrap_or(stripped);
            format!("postgresql://{}:{}@{}", user, db_pass, clean)
        } else {
            let host = std::env::var("POSTGRES_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
            let port = std::env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
            let db = std::env::var("POSTGRES_DB").unwrap_or_else(|_| "inventory_db".to_string());
            let user = std::env::var("POSTGRES_USER").unwrap_or_else(|_| "admin".to_string());
            format!("postgresql://{}:{}@{}:{}/{}", user, db_pass, host, port, db)
        };

        let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        let default_group_id = std::env::var("DEFAULT_GROUP_ID")
            .unwrap_or_else(|_| "default-grocery-group".to_string());

        Self {
            database_url,
            jwt_secret,
            host,
            port,
            default_group_id,
        }
    }
}

fn read_secret(name: &str, env_var: &str) -> Option<String> {
    let secret_path = format!("/run/secrets/{}", name);
    if Path::new(&secret_path).exists() {
        if let Ok(content) = fs::read_to_string(&secret_path) {
            let trimmed = content.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }

    let local_secret_path = format!("secrets/{}.txt", name);
    if Path::new(&local_secret_path).exists() {
        if let Ok(content) = fs::read_to_string(&local_secret_path) {
            let trimmed = content.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }

    std::env::var(env_var)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}
