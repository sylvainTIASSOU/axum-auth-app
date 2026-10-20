mod log;
pub mod db;
pub mod auth;

///Comprendre le système de configuration.
///
/// ServerConfig struct contient les
/// détails de la connexion du serveur.
/// Il fournit deux méthodes d'aide: address() qui
/// formate l'hôte et le port pour la liaison
/// (par ex., 127.0.0.1:7150), et url() qui crée l'URL
/// du serveur complet (par exemple, http://127.0.0.1:7150).
///
/// Config struct est le conteneur de configuration racine.
/// Le load() Une méthode détecte automatiquement l'environnement actuel
/// et charge le fichier de configuration approprié.
/// Le from_env() méthode charge la configuration à partir d'un fichier
/// YAML basé sur le nom de l'environnement et peut être remplacé par
/// des variables d'environnement préfixées avec
/// APP_(en utilisant des doubles soulignements comme séparateurs,
/// par ex., APP_SERVER__PORT=8080). Au fur et à mesure que
/// l'application se développe, nous ajouterons plus de champs à cette structure.
///
/// L'environnement enum représente l'environnement d'exécution de
/// l'application. Le current() La méthode lit l'environnement à partir
/// de APP_ENVIRONMENT ou APP_ENV variables d'environnement, par défaut
/// à Développement si aucune n'est définie. Le From<&str> La mise
/// en œuvre fournit un parsing flexible, acceptant des variations
/// comme "dev"/"développement" ou "prod"/"production", et comprend
/// une autre variante pour les noms d'environnement personnalisés.

use serde::Deserialize;

use crate::Result;

pub use self::{
    auth::{AuthConfig, RsaJwtConfig},
    db::{DatabaseConfig, RedisConfig},
    log::Logger,
};

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    protocol: String,
    host: String,
    port: u16,
}

impl ServerConfig {
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn url(&self) -> String {
        format!("{}://{}", self.protocol, self.address())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    server: ServerConfig,
    log: Logger,
    database: DatabaseConfig,
    redis: RedisConfig,
    auth: AuthConfig,
}

impl Config {
    pub fn load() -> Result<Self> {
        let env =  Environment::current();
        Self::from_env(&env)
    }

    /// Load configuration for a specific environment
    ///
    /// If environment variables are set with prefix APP_, it will also read them
    /// e.g. APP_CONFIG__PORT=5000
    pub fn from_env(env: &Environment) -> Result<Self> {
        let base_dir = std::env::current_dir()?;
        let config_dir = base_dir.join("config");

        let file_name = format!("{}.yaml", env);

        let settings = config::Config::builder()
            .add_source(config::File::from(config_dir.join(file_name)))
            .add_source(
                config::Environment::with_prefix("APP")
                    .separator("__")
                    .prefix_separator("_"),
            )
            .build()?;

        settings.try_deserialize::<Self>().map_err(Into::into)
    }

    pub fn server(&self) -> &ServerConfig {
        &self.server
    }

    pub fn log(&self) -> &Logger {
        &self.log
    }

    pub fn redis(&self) -> &RedisConfig {
        &self.redis
    }
    pub fn database(&self) -> &DatabaseConfig {
        &self.database
    }
    pub fn auth(&self) -> &AuthConfig {
        &self.auth
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub enum Environment {
    #[default]
    Development,
    Production,
    Testing,
    Other(String),
}

impl Environment {
    /// Get the current environment from environment variables
    /// //
    /// Checks `APP_ENVIRONMENT` the `APP_ENV`, defaults to Development
    pub fn current() -> Self {
        std::env::var("APP_ENVIRONMENT")
            .or_else(|_| std::env::var("APP_ENV"))
            .map(|s| Self::from(s.as_str()))
            .unwrap_or_default()
    }
}

impl From<&str> for Environment {
    fn from(s: &str) -> Self {
        match s.to_lowercase().trim() {
            "development" | "dev" => Environment::Development,
            "production" | "prod" => Environment::Production,
            "testing" | "test" => Environment::Testing,
            other => Environment::Other(other.into()),
        }
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Development => "development",
                Self::Production => "production",
                Self::Testing => "testing",
                Self::Other(other) => other.as_str(),
            }
        )
    }
}