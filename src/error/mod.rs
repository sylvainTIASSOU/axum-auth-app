/**
Le thiserrorCrate fournit le Errorcaractère,
qui met automatiquement en œuvre les deux
Displaytrait et From<E>conversions pour nos types d'erreurs.
Cela nous donne une gestion des erreurs (ergonomique)
facile et efficace avec la conversion automatique des
types d'erreurs sous-jacents.

Le color_eyre::Reporttype sert de emballage d'erreur fourre-tout.
Il peut accepter toute erreur qui implémente std::error::Error,
fournissant des rapports d'erreur améliorés avec des contre-traces et
une sortie au format couleur. En l'enveloppant dans le nôtre Reportstruct,
nous pouvons gérer à la fois nos erreurs explicitement définies
(dans le Errorenum) et toute autre erreur de bibliothèques
tierces sans problème de compilation.
*/

use std::fmt::{self, Display};

#[derive(Debug)]
pub struct Report(pub color_eyre::Report);

impl<E> From<E> for Report
where
    E: Into<color_eyre::Report>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub type Result<T, E = Report> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Axum(#[from] axum::Error),
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    
    #[error(transparent)]
    DirectiveParseError(#[from] tracing_subscriber::filter::ParseError),
    #[error(transparent)]
    EnvFilter(#[from] std::env::VarError),
    #[error(transparent)]
    FromEnv(#[from] tracing_subscriber::filter::FromEnvError),
    #[error(transparent)]
    TryInit(#[from] tracing_subscriber::util::TryInitError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
}