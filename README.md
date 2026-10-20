# Axum Auth App

![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)
![Axum](https://img.shields.io/badge/Axum-0.8-blue)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-17-336791?logo=postgresql&logoColor=white)
![Redis](https://img.shields.io/badge/Redis-Docker-red?logo=redis&logoColor=white)
![Status](https://img.shields.io/badge/status-en%20développement-yellow)

> Base d'application backend en **Rust** reposant sur **Axum**, conçue pour évoluer vers un service d'authentification avec PostgreSQL, Redis et des tokens JWT. Le dépôt contient actuellement un serveur HTTP minimal, une infrastructure de configuration et des composants de contexte d'authentification.

**Dépôt :** https://github.com/sylvainTIASSOU/axum-auth-app  
**Auteur :** [sylvainTIASSOU](https://github.com/sylvainTIASSOU)

> **État du projet :** le routeur vérifié expose `GET /hello`. Les dépendances et types relatifs à l'authentification ne constituent pas, à eux seuls, une implémentation complète des parcours d'inscription et de connexion. Le README distingue ce qui a été observé dans le code de ce qui reste à intégrer.

## Sommaire

- [Présentation](#présentation)
- [Stack technique](#stack-technique)
- [Fonctionnalités et avancement](#fonctionnalités-et-avancement)
- [Architecture](#architecture)
- [Prérequis](#prérequis)
- [Installation et démarrage](#installation-et-démarrage)
- [API actuelle](#api-actuelle)
- [Configuration et variables d'environnement](#configuration-et-variables-denvironnement)
- [PostgreSQL, Redis et migrations](#postgresql-redis-et-migrations)
- [Authentification JWT : conception](#authentification-jwt--conception)
- [Tests et qualité du code](#tests-et-qualité-du-code)
- [Sécurité et limites](#sécurité-et-limites)
- [Compétences mises en pratique](#compétences-mises-en-pratique)
- [Feuille de route](#feuille-de-route)
- [Contribution](#contribution)

## Présentation

`axum-auth-app` explore la conception d'un backend Rust structuré : démarrage asynchrone du serveur, configuration d'application, instrumentation des requêtes et préparation des ressources d'authentification. L'architecture prévoit notamment des connexions à PostgreSQL et Redis, ainsi que des paires de clés pour l'encodage et le décodage de JWT.

**Objectifs techniques :**

- Organiser une application Axum en modules réutilisables.
- Séparer le démarrage HTTP, la configuration, les erreurs et le contexte d'application.
- Préparer une architecture d'authentification basée sur des access tokens et refresh tokens.
- Exploiter les outils asynchrones de l'écosystème Rust.
- Faire tourner les services de données en environnement local avec Docker Compose.

## Stack technique

Les versions ci-dessous proviennent du `Cargo.toml` observé sur la branche `main` le 20 octobre 2026.

| Technologie | Version déclarée | Rôle |
| --- | --- | --- |
| Rust | Édition 2024 | Langage et système de compilation |
| Axum | 0.8.9 | Routeur et serveur HTTP |
| Tokio | 1.54.0 | Runtime asynchrone |
| Serde / serde_json | 1.x | Sérialisation et désérialisation |
| SQLx | 0.9.0 | Accès asynchrone à PostgreSQL |
| Redis | 1.7.1 | Client Redis asynchrone |
| jsonwebtoken | 11.1.0 | Encodage et validation de JWT |
| Argon2 | 0.6.0 | Bibliothèque de hachage de mots de passe |
| Tower HTTP | 0.7.1 | Couches HTTP, CORS et tracing |
| Tracing | 0.1.44 | Journalisation structurée |
| config | 0.15.27 | Chargement des paramètres |
| thiserror / color-eyre | 2.0.21 / 0.6.5 | Gestion et diagnostic des erreurs |
| Docker Compose | — | Infrastructure PostgreSQL et Redis |

**Remarque :** la présence d'une dépendance ne signifie pas que toutes ses fonctionnalités sont déjà exploitées par les routes publiques.

## Fonctionnalités et avancement

| Fonctionnalité | État observé |
| --- | --- |
| Initialisation du serveur HTTP Axum | Implémentée |
| Route `GET /hello` | Implémentée |
| Chargement de configuration et journalisation | Présents dans le démarrage |
| Tracing des requêtes HTTP | Présent dans le routeur |
| Types de contexte PostgreSQL, Redis et JWT | Définis dans `AppContext` |
| Conteneurs PostgreSQL et Redis | Définis dans `compose.yaml` |
| Endpoints register/login/logout | Non observés dans le routeur vérifié |
| Protection de routes par JWT | Non observée dans le routeur vérifié |
| Refresh token opérationnel | Non confirmé |
| Documentation Swagger / OpenAPI | Non confirmée |
| Tests d'intégration d'authentification | Non vérifiés |

## Architecture

Les fichiers suivants ont été consultés directement ou sont visibles à la racine du dépôt :

```text
axum-auth-app/
├── .idea/                 # Paramètres d'IDE présents dans le dépôt
├── config/                # Dossier de configuration
├── migrations/            # Dossier prévu pour les migrations SQL
├── src/
│   ├── bin/
│   │   └── main.rs        # Point d'entrée du binaire
│   ├── app.rs             # Construction et démarrage d'Axum
│   ├── context.rs         # AppContext, AuthContext, JwtContext
│   ├── lib.rs             # Déclarations et réexports des modules
│   ├── config/            # Module déclaré par lib.rs (détail non audité)
│   ├── error/             # Module déclaré par lib.rs (détail non audité)
│   └── middlewares/       # Module déclaré par lib.rs (détail non audité)
├── .env                   # Présent dans le dépôt : attention aux secrets !
├── .gitignore
├── Cargo.toml
├── Cargo.lock
├── compose.yaml
└── README.md
```

> Certains sous-dossiers sont présentés comme modules car ils sont déclarés dans `src/lib.rs`. Leur contenu détaillé n'a pas pu être confirmé ; l'arborescence ne prétend pas être exhaustive.

### Parcours de démarrage

```text
src/bin/main.rs
    └── App::run()
          ├── Charge la configuration
          ├── Initialise la journalisation
          ├── Construit le Router Axum
          │     ├── GET /hello
          │     └── TraceLayer
          ├── Démarre TcpListener
          └── Lance axum::serve(...)
```

`AppContext` regroupe une configuration, un pool SQLx PostgreSQL, une connexion Redis multiplexée et un contexte JWT (access/refresh). Il représente une base pour injecter les ressources partagées dans les handlers, mais **son utilisation dans le routeur actuel n'a pas été observée**.

## Prérequis

- [Rust et Cargo](https://rustup.rs/) avec une toolchain prenant en charge l'édition 2024.
- [Docker](https://docs.docker.com/get-docker/) et [Docker Compose](https://docs.docker.com/compose/) pour PostgreSQL et Redis.
- Un environnement local autorisant les ports définis dans la configuration de l'application.
- Les paramètres et, si nécessaires, les clés RSA attendus par la configuration du projet.

Vérifier les outils :

```bash
rustc --version
cargo --version
docker --version
docker compose version
```

## Installation et démarrage

### 1. Cloner le dépôt

```bash
git clone https://github.com/sylvainTIASSOU/axum-auth-app.git
cd axum-auth-app
```

### 2. Préparer les paramètres

Examiner les fichiers de `config/`, le `.env` local et les types de configuration avant de lancer le serveur. Les noms exacts de **toutes** les clés de configuration attendues n'ont pas été vérifiés ; ne supposez pas que le modèle ci-dessous couvre toute la configuration du programme.

Pour Docker Compose, les variables suivantes sont explicitement référencées :

```dotenv
# Exemple local uniquement — choisir des valeurs privées
POSTGRES_USER=app
POSTGRES_PASSWORD=change-me-locally
POSTGRES_DATABASE=auth_db
```

Ne jamais publier un `.env` réel ni des clés privées dans Git.

### 3. Démarrer PostgreSQL et Redis

```bash
docker compose up -d
docker compose ps
```

Le fichier `compose.yaml` déclare :

| Service | Image | Port machine | Port conteneur |
| --- | --- | --- | --- |
| PostgreSQL | `postgres:17` | `5434` | `5432` |
| Redis | `redis:latest` | `6377` | `6379` |

La commande crée aussi des volumes nommés `pg_data` et `redis_data` pour conserver les données entre redémarrages.

### 4. Compiler et lancer l'API

```bash
cargo check
cargo run --bin auth
```

Le binaire est nommé `auth` dans `Cargo.toml`. L'IP et le port d'écoute sont déterminés par le module de configuration (`config.server().address()`), **pas par une valeur fixée dans `main.rs`**.

### 5. Vérifier l'endpoint existant

En remplaçant l'adresse par celle affichée dans les logs :

```bash
curl -i http://127.0.0.1:<PORT>/hello
```

**Réponse attendue :** `Hello world` (texte simple).

> Le démarrage de l'application dépend de la configuration. Si celle-ci est absente ou incorrecte, il faut résoudre l'erreur affichée au lancement avant d'appeler l'API.

### Arrêt de l'environnement local

```bash
docker compose down
```

Pour supprimer **également les volumes et toutes les données locales** (opération destructive) :

```bash
docker compose down -v
```

## API actuelle

| Méthode | Chemin | Description | Authentification |
| --- | --- | --- | --- |
| `GET` | `/hello` | Message de test `Hello world` | Non |

### Exemple de requête

```http
GET /hello HTTP/1.1
Host: localhost:<PORT>
```

**À ne pas confondre avec une API d'authentification terminée :** aucune route `/register`, `/login`, `/refresh` ou `/me` n'est enregistrée dans le `Router` examiné.

## Configuration et variables d'environnement

L'application charge la configuration via `Config::load()`, initialise la journalisation via `config.log().setup()` et résout l'adresse serveur via `config.server().address()`.

Le fichier Compose lit :

- `POSTGRES_USER`
- `POSTGRES_PASSWORD`
- `POSTGRES_DATABASE`

Pour les secrets JWT, les paramètres de connexion et la configuration Redis, se référer aux modules du dossier `src/config/` et du dossier `config/` dans le dépôt. Cette documentation n'invente pas de noms de variables non confirmés.

### Recommandations de configuration

- Fournir un `.env.example` sans secret.
- Exclure `.env`, fichiers PEM privés et autres secrets via `.gitignore`.
- Séparer développement, test et production.
- Valider les paramètres au démarrage et échouer avec un message explicite si un paramètre indispensable manque.
- Privilégier les variables d'environnement ou un gestionnaire de secrets en production.

## PostgreSQL, Redis et migrations

### PostgreSQL

`sqlx::PgPool` est le type de pool de connexions prévu dans `AppContext`. PostgreSQL est démarré dans le conteneur Docker sur le port local `5434`.

La présence du dossier `migrations/` montre qu'une organisation des évolutions du schéma est prévue. Les fichiers de migration, leur contenu et leur exécution automatique n'ont pas été confirmés.

Si les migrations sont compatibles avec **SQLx CLI**, un workflow *possible après vérification* serait :

```bash
cargo install sqlx-cli --no-default-features --features postgres
# Vérifier DATABASE_URL et le contenu du dossier migrations/ avant exécution.
# sqlx migrate run
```

### Redis

Une `redis::aio::MultiplexedConnection` est prévue dans le contexte. Redis peut servir ultérieurement à conserver des sessions, des listes de révocation, des codes temporaires ou des mécanismes de limitation de débit — **ces usages sont des pistes d'évolution, pas des fonctions confirmées**.

## Authentification JWT : conception

`src/context.rs` définit :

- `AuthContext` avec les contextes `access` et `refresh`.
- `JwtContext` avec une clé d'encodage, une clé de décodage et une durée d'expiration (`exp`).
- Une construction des clés à partir d'une configuration `RsaJwtConfig`.

Cela établit une **structure de préparation des JWT**. Cela ne démontre pas encore un cycle complet d'inscription, hachage, connexion, émission et renouvellement de tokens.

### Parcours d'authentification envisagé

```text
POST /auth/register (à développer)
    -> Validation -> Hachage Argon2 -> Insertion PostgreSQL
POST /auth/login (à développer)
    -> Vérification mot de passe -> Access + Refresh tokens
GET /auth/me (à développer)
    -> Vérification access token -> Utilisateur authentifié
POST /auth/refresh (à développer)
    -> Validation / rotation du refresh token
POST /auth/logout (à développer)
    -> Révocation ou invalidation côté serveur
```

Ce diagramme est une **feuille de route**, et non la documentation d'endpoints existants.

## Tests et qualité du code

Commandes utiles :

```bash
cargo fmt --all -- --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Ces commandes constituent une procédure recommandée ; elles ne sont **pas présentées comme des tests déjà exécutés avec succès**.

Avant une utilisation réelle de l'authentification, ajouter des tests sur :

- L'inscription et les emails dupliqués.
- La vérification du hachage de mot de passe.
- L'expiration et la signature des JWT.
- Le refus d'un token invalide.
- La rotation et la révocation des refresh tokens.
- L'accès aux routes protégées.
- Les erreurs PostgreSQL et Redis.

## Sécurité et limites

### Points d'attention constatés

1. **`.env` présent dans un dépôt public :** examiner immédiatement son contenu. S'il contient des secrets réels, les révoquer / faire tourner ; supprimer le suivi Git du fichier et envisager une purge de l'historique Git selon l'exposition.
2. **Initialisation du contexte asynchrone :** `AppContext::try_from` utilise `tokio::runtime::Handle::current().block_on(...)`. Appeler cette conversion depuis un contexte asynchrone Tokio peut provoquer un panic. Préférer un constructeur `async fn AppContext::new(...) -> Result<_>`.
3. **Authentification incomplète dans le routeur inspecté :** aucune protection ne doit être supposée avant de l'avoir réellement implémentée et testée.
4. **Conteneurs de développement :** les ports de PostgreSQL et Redis sont exposés sur la machine hôte. Adapter le réseau et les accès avant toute mise en production.

### Bonnes pratiques pour la suite

- Hacher les mots de passe avec Argon2id et un sel aléatoire.
- Vérifier explicitement les algorithmes JWT autorisés, l'émetteur, l'audience et l'expiration.
- Éviter de journaliser les mots de passe, cookies et tokens.
- Ajouter une limitation de débit sur les routes d'authentification.
- Utiliser TLS/HTTPS en production et un cycle de vie explicite des clés.
- Prévoir des tokens de refresh révocables et une rotation contrôlée.
- Stocker la configuration sensible hors du dépôt.

## Compétences mises en pratique

Ce projet est particulièrement utile pour démontrer une progression en développement backend Rust. Les compétences ci-dessous sont reliées aux éléments **observés dans le dépôt**, et non à des fonctionnalités simplement envisagées.

| Compétence | Éléments concrets du dépôt | Niveau de preuve |
| --- | --- | --- |
| **Rust : modularité et bibliothèque** | `src/lib.rs`, réexports, binaire séparé dans `src/bin/main.rs` | Démontrée |
| **Programmation asynchrone** | `#[tokio::main]`, `App::run().await`, `TcpListener`, `axum::serve` | Démontrée |
| **Création d'API HTTP avec Axum** | `Router::new()`, `.route("/hello", get(...))` | Démontrée |
| **Middleware et observabilité** | `TraceLayer`, hooks de requête, réponse et échec | Démontrée |
| **Configuration applicative** | `Config::load()`, paramétrage de logs et serveur | Démontrée |
| **Gestion des erreurs Rust** | `Result`, `?`, `map_err(Into::into)` | Démontrée |
| **Conception d'état partagé** | Structures `AppContext`, `AuthContext`, `JwtContext` avec `Clone` | Architecture préparée |
| **Écosystème SQLx / PostgreSQL** | `PgPool` et service Docker PostgreSQL | Infrastructure préparée |
| **Redis asynchrone** | `MultiplexedConnection` et conteneur Redis | Infrastructure préparée |
| **JWT et cryptographie applicative** | `EncodingKey`, `DecodingKey`, contextes access/refresh | Types préparés, flux non démontré |
| **Docker Compose** | Définition de services, ports et volumes | Démontrée |

### Ce que ce projet m'a permis de pratiquer

- Structurer un backend Rust en composants testables et séparés.
- Démarrer un serveur HTTP asynchrone avec Axum et Tokio.
- Mettre en place la journalisation et le suivi du cycle de vie des requêtes.
- Préparer un contexte applicatif partagé regroupant configuration et services externes.
- Définir une infrastructure locale avec PostgreSQL, Redis et Docker Compose.
- Aborder la conception d'un système de tokens JWT d'accès et de rafraîchissement.

> **Valorisation portfolio :** « Conception d'une base backend Rust/Axum modulaire, avec serveur asynchrone, instrumentation HTTP, configuration centralisée et infrastructure PostgreSQL/Redis. Préparation des composants nécessaires à une authentification JWT access/refresh. »

## Feuille de route

- [x] Créer le projet Rust (édition 2024).
- [x] Séparer la bibliothèque et le binaire.
- [x] Configurer un serveur Axum minimal et le tracing HTTP.
- [x] Définir les services PostgreSQL et Redis dans Docker Compose.
- [x] Définir les contextes de clés JWT.
- [ ] Corriger l'initialisation asynchrone de `AppContext`.
- [ ] Retirer les secrets potentiels du dépôt et ajouter `.env.example`.
- [ ] Connecter explicitement le contexte au routeur Axum.
- [ ] Implémenter inscription et connexion avec mots de passe hachés.
- [ ] Implémenter les extracteurs / middlewares d'authentification.
- [ ] Implémenter access token, refresh token et révocation.
- [ ] Finaliser et vérifier les migrations SQLx.
- [ ] Ajouter les validations des payloads et les réponses d'erreur uniformes.
- [ ] Documenter les endpoints avec OpenAPI / Swagger UI.
- [ ] Ajouter des tests unitaires et d'intégration.
- [ ] Mettre en place CI (fmt, clippy, tests et audit).

## Contribution

Les retours, issues et pull requests sont les bienvenus : [ouvrir une issue](https://github.com/sylvainTIASSOU/axum-auth-app/issues).

Avant une pull request :

```bash
cargo fmt --all
cargo clippy --all-targets --all-features
cargo test
```

## Licence

Aucune licence n'a été vérifiée à la racine du dépôt lors de cette analyse. Vérifier / ajouter un fichier `LICENSE` avant de revendiquer une licence de redistribution.

---

**Auteur :** [sylvainTIASSOU](https://github.com/sylvainTIASSOU)  
**Projet :** [axum-auth-app](https://github.com/sylvainTIASSOU/axum-auth-app)  
**Documentation fondée sur l'état public du dépôt consulté le 20 octobre 2026.**
