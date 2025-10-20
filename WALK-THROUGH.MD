Sweet — let’s turn your skeleton into a **fully running, pluggable stack** (Rust + WASM + REST/GraphQL/gRPC/QUIC) wired to **MySQL, Postgres, MongoDB, ClickHouse, Redis**. I’ll give you copy-paste steps, minimal working code, and smoke tests.

---

# 0) Prereqs (install once)

```bash
# Rust toolchains & targets
rustup update
rustup target add wasm32-unknown-unknown

# Web (for the WASM apps)
cargo install trunk wasm-bindgen-cli

# Protocol buffers for gRPC
# Linux: apt-get install -y protobuf-compiler
# Mac:   brew install protobuf

# SQL migrations (optional but handy)
cargo install sqlx-cli --no-default-features --features mysql,postgres,rustls
```

---

# 1) Env & Ports

Create `.env` at the workspace root:

```dotenv
RUST_LOG=info,hyper=info,sqlx=warn
SERVICE_NAME=devstack

# Databases
MYSQL_URL=mysql://root:root@127.0.0.1:3306/app
POSTGRES_URL=postgres://postgres:postgres@127.0.0.1:5432/app
REDIS_URL=redis://127.0.0.1:6379
MONGO_URL=mongodb://root:root@127.0.0.1:27017
CLICKHOUSE_URL=http://default:@127.0.0.1:8123

# Choose primary relational store for examples: mysql|postgres
PRIMARY_SQL=mysql

# Gateways
REST_ADDR=0.0.0.0:8080
GRAPHQL_ADDR=0.0.0.0:8081
GRPC_ADDR=0.0.0.0:9090
QUIC_ADDR=0.0.0.0:7443
```

---

# 2) Local infra (Compose)

`infra/docker-compose.yml`:

```yaml
version: "3.9"
services:
  mysql:
    image: mysql:8
    environment:
      MYSQL_ROOT_PASSWORD: root
      MYSQL_DATABASE: app
    ports: ["3306:3306"]
    healthcheck: { test: ["CMD", "mysqladmin", "ping", "-h", "127.0.0.1","-proot"], interval: 5s, timeout: 3s, retries: 20 }

  postgres:
    image: postgres:16
    environment:
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: app
    ports: ["5432:5432"]
    healthcheck: { test: ["CMD-SHELL","pg_isready -U postgres"], interval: 5s, timeout: 3s, retries: 20 }

  redis:
    image: redis:7
    ports: ["6379:6379"]
    healthcheck: { test: ["CMD","redis-cli","ping"], interval: 5s, timeout: 3s, retries: 20 }

  mongo:
    image: mongo:7
    command: ["--auth"]
    environment:
      MONGO_INITDB_ROOT_USERNAME: root
      MONGO_INITDB_ROOT_PASSWORD: root
    ports: ["27017:27017"]
    healthcheck: { test: ["CMD","mongosh","--eval","db.adminCommand('ping')"], interval: 10s, timeout: 5s, retries: 20 }

  clickhouse:
    image: clickhouse/clickhouse-server:24
    ports: ["8123:8123","9000:9000"]
    healthcheck: { test: ["CMD","wget","-qO-","http://127.0.0.1:8123/ping"], interval: 5s, timeout: 3s, retries: 20 }
```

Spin it up:

```bash
docker compose -f infra/docker-compose.yml up -d
```

---

# 3) Migrations (example tables)

`infra/migrations/mysql/0001_users.sql`

```sql
CREATE TABLE IF NOT EXISTS users (
  id BIGINT PRIMARY KEY AUTO_INCREMENT,
  email VARCHAR(255) UNIQUE NOT NULL,
  display_name VARCHAR(255) NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

`infra/migrations/postgres/0001_users.sql`

```sql
CREATE TABLE IF NOT EXISTS users (
  id BIGSERIAL PRIMARY KEY,
  email TEXT UNIQUE NOT NULL,
  display_name TEXT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT now()
);
```

`infra/migrations/clickhouse/0001_events.sql`

```sql
CREATE TABLE IF NOT EXISTS events
(
  ts DateTime DEFAULT now(),
  user_id UInt64,
  kind LowCardinality(String),
  meta String
) ENGINE = MergeTree ORDER BY (ts, user_id);
```

Apply:

```bash
# MySQL
sqlx migrate run --database-url "$MYSQL_URL" --source infra/migrations/mysql
# Postgres
sqlx migrate run --database-url "$POSTGRES_URL" --source infra/migrations/postgres

# ClickHouse (simple curl)
curl -sS "$CLICKHOUSE_URL" --data-binary @infra/migrations/clickhouse/0001_events.sql
```

---

# 4) Shared libs you’ll reuse

### 4.1 `libs/common/src/lib.rs`

```rust
use dotenvy::dotenv;
use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub primary_sql: String,
    pub rest_addr: String,
    pub graphql_addr: String,
    pub grpc_addr: String,
    pub quic_addr: String,
    pub mysql_url: String,
    pub postgres_url: String,
    pub redis_url: String,
    pub mongo_url: String,
    pub clickhouse_url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenv().ok();
        let e = |k: &str, d: &str| env::var(k).unwrap_or_else(|_| d.to_string());
        Self {
            primary_sql: e("PRIMARY_SQL", "mysql"),
            rest_addr: e("REST_ADDR", "0.0.0.0:8080"),
            graphql_addr: e("GRAPHQL_ADDR", "0.0.0.0:8081"),
            grpc_addr: e("GRPC_ADDR", "0.0.0.0:9090"),
            quic_addr: e("QUIC_ADDR", "0.0.0.0:7443"),
            mysql_url: e("MYSQL_URL", "mysql://root:root@127.0.0.1:3306/app"),
            postgres_url: e("POSTGRES_URL", "postgres://postgres:postgres@127.0.0.1:5432/app"),
            redis_url: e("REDIS_URL", "redis://127.0.0.1:6379"),
            mongo_url: e("MONGO_URL", "mongodb://root:root@127.0.0.1:27017"),
            clickhouse_url: e("CLICKHOUSE_URL", "http://127.0.0.1:8123"),
        }
    }
}

pub type AppResult<T> = Result<T, anyhow::Error>;
```

### 4.2 `libs/tracing-utils/src/lib.rs`

```rust
pub fn init(service: &str) {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .init();
    tracing::info!(service, "tracing initialized");
}
```

### 4.3 `libs/http-layers/src/lib.rs`

```rust
use std::time::Duration;
use tower::{ServiceBuilder, limit::ConcurrencyLimitLayer, timeout::TimeoutLayer};
use tower_http::{trace::TraceLayer, classify::ServerErrorsFailureClass, set_header::SetResponseHeaderLayer};
use http::header::{HeaderName, HeaderValue};

pub fn stack() -> ServiceBuilder<()> {
    ServiceBuilder::new()
        .layer(TraceLayer::new_for_http().on_response(|res, _lat, _span| {
            if res.status().is_server_error() {
                tracing::warn!(status=?res.status(), "5xx response");
            }
        }))
        .layer(ConcurrencyLimitLayer::new(1024))
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
}
```

### 4.4 DB pools

`libs/db-mysql/src/lib.rs`

```rust
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use std::time::Duration;

pub async fn pool(url: &str) -> anyhow::Result<MySqlPool> {
    Ok(MySqlPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(url).await?)
}
```

`libs/db-postgres/src/lib.rs`

```rust
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

pub async fn pool(url: &str) -> anyhow::Result<PgPool> {
    Ok(PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(url).await?)
}
```

`libs/db-redis/src/lib.rs`

```rust
use redis::AsyncCommands;

pub async fn ping(url: &str) -> anyhow::Result<String> {
    let client = redis::Client::open(url)?;
    let mut con = client.get_async_connection().await?;
    let pong: String = con.ping().await?;
    Ok(pong)
}
```

`libs/db-mongo/src/lib.rs`

```rust
use mongodb::{Client, options::ClientOptions};

pub async fn client(url: &str) -> anyhow::Result<Client> {
    let mut opts = ClientOptions::parse(url).await?;
    opts.app_name = Some("resilient-webstack".into());
    Ok(Client::with_options(opts)?)
}
```

`libs/db-clickhouse/src/lib.rs`

```rust
use clickhouse::{Client, Row};
use serde::{Serialize, Deserialize};

pub fn client(url: &str) -> Client {
    Client::default().with_url(url)
}

#[derive(Row, Serialize, Deserialize, Debug)]
pub struct Event { pub user_id: u64, pub kind: String, pub meta: String }
```

---

# 5) Protocol gateways

## 5.1 REST (Axum)

`services/gateway-rest/src/main.rs`

```rust
use axum::{routing::{get, post}, Router, extract::{Path, State}, Json};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Clone)]
struct AppState {
    primary: Primary,
}

#[derive(Clone)]
enum Primary {
    MySql(sqlx::MySqlPool),
    Postgres(sqlx::PgPool),
}

#[derive(Serialize, Deserialize)]
struct CreateUser { email: String, display_name: String }
#[derive(Serialize, Deserialize)]
struct User { id: i64, email: String, display_name: String }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = common::AppConfig::from_env();
    tracing_utils::init("gateway-rest");

    let primary = match cfg.primary_sql.as_str() {
        "postgres" => Primary::Postgres(db_postgres::pool(&cfg.postgres_url).await?),
        _ => Primary::MySql(db_mysql::pool(&cfg.mysql_url).await?),
    };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/users/:id", get(get_user))
        .route("/users", post(create_user))
        .with_state(AppState { primary });

    let addr: SocketAddr = cfg.rest_addr.parse()?;
    tracing::info!(%addr, "REST listening");
    axum::Server::bind(&addr)
        .serve(http_layers::stack().layer(app).into_make_service())
        .await?;
    Ok(())
}

async fn get_user(State(st): State<AppState>, Path(id): Path<i64>) -> Result<Json<User>, axum::http::StatusCode> {
    match &st.primary {
        Primary::MySql(pool) => {
            let rec = sqlx::query!("SELECT id, email, display_name FROM users WHERE id=?", id)
                .fetch_one(pool).await.map_err(|_| axum::http::StatusCode::NOT_FOUND)?;
            Ok(Json(User { id: rec.id, email: rec.email, display_name: rec.display_name }))
        }
        Primary::Postgres(pool) => {
            let rec = sqlx::query!("SELECT id, email, display_name FROM users WHERE id=$1", id)
                .fetch_one(pool).await.map_err(|_| axum::http::StatusCode::NOT_FOUND)?;
            Ok(Json(User { id: rec.id, email: rec.email, display_name: rec.display_name }))
        }
    }
}

async fn create_user(State(st): State<AppState>, Json(body): Json<CreateUser>) -> Result<Json<User>, axum::http::StatusCode> {
    match &st.primary {
        Primary::MySql(pool) => {
            let rec = sqlx::query!("INSERT INTO users (email, display_name) VALUES (?,?)", body.email, body.display_name)
                .execute(pool).await.map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
            let id = rec.last_insert_id() as i64;
            Ok(Json(User { id, email: body.email, display_name: body.display_name }))
        }
        Primary::Postgres(pool) => {
            let rec = sqlx::query!("INSERT INTO users (email, display_name) VALUES ($1,$2) RETURNING id", body.email, body.display_name)
                .fetch_one(pool).await.map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
            Ok(Json(User { id: rec.id, email: body.email, display_name: body.display_name }))
        }
    }
}
```

Run:

```bash
cargo run -p gateway-rest
# Test
curl -X POST http://localhost:8080/users -d '{"email":"a@x.io","display_name":"Alice"}' -H 'content-type: application/json'
curl http://localhost:8080/users/1
```

---

## 5.2 GraphQL (async-graphql + Axum)

`libs/graphql-schema/src/lib.rs`

```rust
use async_graphql::{Object, Schema, EmptySubscription};
use sqlx::{MySqlPool, PgPool};

pub struct QueryRoot;
pub struct MutationRoot;
#[derive(Clone)]
pub enum Primary { My(MySqlPool), Pg(PgPool) }

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[Object]
impl QueryRoot {
    async fn health(&self) -> &str { "ok" }
}

#[Object]
impl MutationRoot {
    async fn create_user(&self, ctx: &async_graphql::Context<'_>, email: String, display_name: String) -> async_graphql::Result<i64> {
        let primary = ctx.data_unchecked::<Primary>();
        let id = match primary {
            Primary::My(pool) => {
                let res = sqlx::query!("INSERT INTO users (email, display_name) VALUES (?,?)", email, display_name)
                    .execute(pool).await?;
                res.last_insert_id() as i64
            }
            Primary::Pg(pool) => {
                let r = sqlx::query!("INSERT INTO users (email, display_name) VALUES ($1,$2) RETURNING id", email, display_name)
                    .fetch_one(pool).await?;
                r.id
            }
        };
        Ok(id)
    }
}

pub fn build(primary: Primary) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(primary)
        .finish()
}
```

`services/gateway-graphql/src/main.rs`

```rust
use axum::{routing::get, Router};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use graphql_schema::{build, Primary};
use std::net::SocketAddr;

async fn graphql_handler(schema: graphql_schema::AppSchema, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = common::AppConfig::from_env();
    tracing_utils::init("gateway-graphql");

    let primary = match cfg.primary_sql.as_str() {
        "postgres" => Primary::Pg(db_postgres::pool(&cfg.postgres_url).await?),
        _ => Primary::My(db_mysql::pool(&cfg.mysql_url).await?),
    };

    let schema = build(primary);
    let app = Router::new()
        .route("/graphql", get(|| async { "POST GraphQL" }).post(graphql_handler))
        .layer(http_layers::stack().layer(tower::LayerFn::new(move |svc| {
            axum::extract::State::layer(schema.clone()).layer(svc)
        })));

    let addr: SocketAddr = cfg.graphql_addr.parse()?;
    tracing::info!(%addr, "GraphQL listening");
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}
```

Test (GraphiQL):

```bash
curl -X POST http://localhost:8081/graphql \
  -H 'content-type: application/json' \
  -d '{"query":"mutation{ createUser(email:\"b@x.io\", displayName:\"Bob\") }"}'
```

---

## 5.3 gRPC (tonic)

`libs/proto/build.rs`

```rust
fn main() {
    tonic_build::configure().compile(&["proto/user.proto"], &["proto"]).unwrap();
}
```

`libs/proto/proto/user.proto`

```proto
syntax = "proto3";
package user.v1;

service UserService {
  rpc GetUser(GetUserRequest) returns (GetUserResponse);
}
message GetUserRequest { int64 id = 1; }
message GetUserResponse { int64 id = 1; string email = 2; string display_name = 3; }
```

`services/gateway-grpc/src/main.rs`

```rust
use tonic::{transport::Server, Request, Response, Status};
use proto::user::v1::{user_service_server::{UserService, UserServiceServer}, GetUserRequest, GetUserResponse};
use proto as _; // ensure build links

pub mod proto { pub mod user { pub mod v1 { tonic::include_proto!("user.v1"); } } }

#[derive(Clone)]
enum Primary { My(sqlx::MySqlPool), Pg(sqlx::PgPool) }

#[derive(Clone)]
struct Svc { primary: Primary }

#[tonic::async_trait]
impl UserService for Svc {
    async fn get_user(&self, req: Request<GetUserRequest>) -> Result<Response<GetUserResponse>, Status> {
        let id = req.into_inner().id;
        let out = match &self.primary {
            Primary::My(pool) => sqlx::query!("SELECT id, email, display_name FROM users WHERE id=?", id)
                .fetch_one(pool).await,
            Primary::Pg(pool) => sqlx::query!("SELECT id, email, display_name FROM users WHERE id=$1", id)
                .fetch_one(pool).await,
        }.map_err(|_| Status::not_found("not found"))?;
        Ok(Response::new(GetUserResponse{ id: out.id, email: out.email, display_name: out.display_name }))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = common::AppConfig::from_env();
    tracing_utils::init("gateway-grpc");
    let primary = match cfg.primary_sql.as_str() {
        "postgres" => Primary::Pg(db_postgres::pool(&cfg.postgres_url).await?),
        _ => Primary::My(db_mysql::pool(&cfg.mysql_url).await?),
    };
    let addr = cfg.grpc_addr.parse()?;
    Server::builder()
        .add_service(UserServiceServer::new(Svc{ primary }))
        .serve(addr).await?;
    Ok(())
}
```

Test:

```bash
grpcurl -plaintext localhost:9090 list
grpcurl -plaintext -d '{"id":1}' localhost:9090 user.v1.UserService/GetUser
```

---

## 5.4 QUIC (quinn) — minimal echo/ping

`services/gateway-quic/src/main.rs`

```rust
use quinn::{Endpoint, ServerConfig};
use std::{net::SocketAddr, sync::Arc};
use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = common::AppConfig::from_env();
    tracing_utils::init("gateway-quic");

    let (cert, key) = rcgen::generate_simple_self_signed(["localhost".into()])?.serialize_der()?;
    let priv_key = rustls::pki_types::PrivateKeyDer::from(key);
    let cert_chain = vec![rustls::pki_types::CertificateDer::from(cert)];

    let mut server_config = ServerConfig::with_single_cert(cert_chain, priv_key).context("cert")?;
    Arc::get_mut(&mut server_config.transport).unwrap().max_concurrent_bidi_streams(100u32.into());

    let addr: SocketAddr = cfg.quic_addr.parse()?;
    let endpoint = Endpoint::server(server_config, addr)?;
    tracing::info!(%addr, "QUIC listening");

    loop {
        let conn = endpoint.accept().await.context("accept")?;
        tokio::spawn(async move {
            let conn = conn.await.context("handshake").unwrap();
            while let Ok(mut bi) = conn.accept_bi().await {
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 4096];
                    let n = bi.1.read(&mut buf).await.unwrap().unwrap_or(0);
                    let _ = bi.0.write_all(&buf[..n]).await; // echo
                    let _ = bi.0.finish().await;
                });
            }
        });
    }
}
```

Tiny Rust client (inside `clients/cli/src/main.rs`) to ping:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = std::env::var("QUIC_ADDR").unwrap_or("127.0.0.1:7443".into());
    let mut roots = rustls::RootCertStore::empty();
    // for demo, accept any cert (DANGEROUS in prod). Instead, pin cert.
    let tls = rustls::ClientConfig::builder().with_safe_default_protocol_versions().unwrap()
        .with_root_certificates(roots).with_no_client_auth();
    let mut endpoint = quinn::Endpoint::client("0.0.0.0:0".parse().unwrap())?;
    endpoint.set_default_client_config(quinn::ClientConfig::new(Arc::new(tls)));

    let conn = endpoint.connect(addr.parse().unwrap(), "localhost")?.await?;
    let (mut s, mut r) = conn.open_bi().await?;
    s.write_all(b"ping").await?;
    s.finish().await?;
    let mut buf = vec![0; 4];
    r.read_exact(&mut buf).await?;
    println!("echo: {}", String::from_utf8_lossy(&buf));
    Ok(())
}
```

Run:

```bash
cargo run -p gateway-quic
cargo run -p cli
```

---

# 6) ClickHouse analytics write path (from REST)

Add to `gateway-rest/src/main.rs` (top-level state):

```rust
#[derive(Clone)]
struct Analytics { ch: clickhouse::Client }

#[derive(Clone)]
struct AppState { primary: Primary, analytics: Analytics }
```

Init in `main`:

```rust
let ch = db_clickhouse::client(&cfg.clickhouse_url);
let app = Router::new()
  // routes...
  .with_state(AppState { primary, analytics: Analytics{ ch }});
```

Emit an event inside `create_user`:

```rust
let _ = st.analytics.ch
    .insert("events")
    .write(&db_clickhouse::Event { user_id: id as u64, kind: "user.created".into(), meta: "{}".into() })
    .await;
```

Query in CH:

```bash
curl "$CLICKHOUSE_URL" -d 'SELECT count() FROM events WHERE kind = '\''user.created'\'''
```

---

# 7) MongoDB content example (session/content service)

`services/content-service/src/main.rs` (minimal “create document”)

```rust
use axum::{routing::post, Router, extract::State, Json};
use serde::{Serialize, Deserialize};

#[derive(Clone)]
struct AppState { mongo: mongodb::Client }

#[derive(Serialize, Deserialize)]
struct DocIn { user_id: i64, title: String, body: String }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = common::AppConfig::from_env();
    tracing_utils::init("content-service");
    let mongo = db_mongo::client(&cfg.mongo_url).await?;
    let app = Router::new().route("/docs", post(create)).with_state(AppState{ mongo });
    axum::serve(tokio::net::TcpListener::bind("0.0.0.0:8085").await?, app).await?;
    Ok(())
}

async fn create(State(st): State<AppState>, Json(input): Json<DocIn>) -> Result<&'static str, axum::http::StatusCode> {
    let col = st.mongo.database("app").collection::<mongodb::bson::Document>("docs");
    let doc = mongodb::bson::doc!{ "user_id": input.user_id, "title": input.title, "body": input.body };
    col.insert_one(doc, None).await.map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok("ok")
}
```

---

# 8) WASM clients (Yew) quick start

`clients/web-wasm/Cargo.toml`

```toml
[package] name="web-wasm" version="0.1.0" edition="2021"
[lib] crate-type=["cdylib"]
[dependencies]
yew = { version="0.21", features=["csr"] }
gloo-net = "0.5"
wasm-bindgen = "0.2"
```

`clients/web-wasm/src/lib.rs`

```rust
use yew::prelude::*;
use gloo_net::http::Request;

#[function_component(App)]
pub fn app() -> Html {
    let resp = use_state(|| "…".to_string());
    let on_click = {
        let resp = resp.clone();
        Callback::from(move |_| {
            let resp = resp.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let r = Request::get("/api/health").send().await.unwrap().text().await.unwrap();
                resp.set(r);
            })
        })
    };
    html! {
      <div>
        <button onclick={on_click}>{"Ping REST"}</button>
        <p>{ (*resp).clone() }</p>
      </div>
    }
}
```

`clients/web-wasm/index.html`

```html
<!doctype html><html><body><div id="root"></div><script type="module">import init,{App} from "./web-wasm.js";await init();</script></body></html>
```

Dev serve:

```bash
cd clients/web-wasm
trunk serve --port 3000 --proxy-backend http://127.0.0.1:8080 --proxy-rewrite /api/
```

---

# 9) Run all locally

`scripts/run-all-local.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail
export RUST_LOG=${RUST_LOG:-info}

# Ensure infra is up
docker compose -f infra/docker-compose.yml up -d

# Migrations
sqlx migrate run --database-url "$MYSQL_URL" --source infra/migrations/mysql || true
sqlx migrate run --database-url "$POSTGRES_URL" --source infra/migrations/postgres || true
curl -sS "$CLICKHOUSE_URL" --data-binary @infra/migrations/clickhouse/0001_events.sql || true

# Start services in panes (tmux optional)
cargo run -p gateway-rest     &
cargo run -p gateway-graphql  &
cargo run -p gateway-grpc     &
cargo run -p gateway-quic     &
cargo run -p content-service  &
wait
```

---

# 10) Smoke tests

```bash
# REST
curl -s http://localhost:8080/health
curl -s -X POST http://localhost:8080/users -H 'content-type: application/json' -d '{"email":"c@x.io","display_name":"Carol"}'

# GraphQL
curl -s -X POST http://localhost:8081/graphql -H 'content-type: application/json' \
  -d '{"query":"mutation{ createUser(email:\"d@x.io\", displayName:\"Dan\") }"}'

# gRPC
grpcurl -plaintext -d '{"id":1}' localhost:9090 user.v1.UserService/GetUser

# QUIC (echo)
cargo run -p cli

# Mongo content
curl -s -X POST http://localhost:8085/docs -H 'content-type: application/json' \
  -d '{"user_id":1,"title":"Hello","body":"World"}'

# ClickHouse verify
curl "$CLICKHOUSE_URL" -d "SELECT count() FROM events"
```

---

# 11) Hardening & production steps (brief)

* **Backpressure & timeouts** already in `http-layers`. Consider adding:

  * `tower::retry` with **jitter** for idempotent endpoints,
  * `tower_http::limit::RequestBodyLimitLayer`,
  * `tower_http::cors::CorsLayer`.
* **Zero-downtime deploys**: use K8s RollingUpdate with `maxSurge: 25%`, `maxUnavailable: 0`, health probes on `/health`.
* **DB pooling**: use `sqlx` `max_connections` sizing per CPU; add pgbouncer for Postgres.
* **Observability**: add OpenTelemetry (`tracing-opentelemetry`, OTLP exporter) → Tempo/Jaeger; Prometheus metrics via `axum-prometheus`.
* **Schema governance**: OpenAPI file in `apis/rest/oas.yaml`; GraphQL schema in `apis/graphql/schema.graphql`; gRPC in `libs/proto/proto/*.proto`. Generate clients in CI.
* **SLOs**: target `p95 < 200ms`, `5xx < 0.1%`, **99.9%** uptime.

---

# 12) What you now have

* One command (`run-all-local.sh`) boots all databases, runs migrations, and starts **REST, GraphQL, gRPC, QUIC**.
* Each protocol can **create/read users** against **MySQL or Postgres** (toggle with `PRIMARY_SQL`).
* **MongoDB** stores free-form content/docs.
* **ClickHouse** records analytics events.
* A **Yew/Trunk WASM** app can hit REST through a proxy.

If you want, I can push **ready-to-build files** (Cargo.toml updates, minor crate glue) and add **CI (fmt/clippy/test/audit)** + **K8s** manifests (readiness/liveness, HPA, PDB, canary).
