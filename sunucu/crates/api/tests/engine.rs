//! Engine client: cache, `calc_results`, version stamp and error mapping, against an in-process
//! `engine-service` (real recipe service, fake version service) on a random port.
//! Needs PostgreSQL (`#[sqlx::test]`) and the compose Redis.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use api::{
    ErrorCode,
    engine::{CalcScope, CalcTarget, EngineClient, EngineConfig, Source, proto},
    redis_client::Redis,
};
use engine_service::proto::{
    GetVersionRequest, GetVersionResponse,
    engine_service_server::{EngineService, EngineServiceServer},
};
use sqlx::PgPool;
use support::{app_pool, insert_user, insert_workspace, redis_url};
use tokio::{net::TcpListener, sync::oneshot};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{Request, Response, Status, transport::Server};
use uuid::Uuid;

#[derive(Clone)]
struct FakeVersion(Arc<Mutex<String>>);

#[tonic::async_trait]
impl EngineService for FakeVersion {
    async fn get_version(
        &self,
        _: Request<GetVersionRequest>,
    ) -> Result<Response<GetVersionResponse>, Status> {
        Ok(Response::new(GetVersionResponse {
            version: self.0.lock().unwrap().clone(),
        }))
    }
}

struct Engine {
    url: String,
    version: Arc<Mutex<String>>,
    stop: oneshot::Sender<()>,
}

async fn start_engine() -> Engine {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let version = Arc::new(Mutex::new("1.0.0-test".to_owned()));
    let (stop, rx) = oneshot::channel::<()>();
    let fake = FakeVersion(version.clone());
    tokio::spawn(async move {
        Server::builder()
            .add_service(EngineServiceServer::new(fake))
            .add_service(engine_service::recipe::server())
            .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                let _ = rx.await;
            })
            .await
    });
    Engine { url, version, stop }
}

fn client(url: &str, redis: Redis) -> EngineClient {
    let mut cfg = EngineConfig::new(url);
    cfg.version_ttl = std::time::Duration::ZERO;
    cfg.connect_timeout = std::time::Duration::from_millis(300);
    EngineClient::connect_lazy(cfg, redis).unwrap()
}

fn good_request(volume: f64) -> proto::CalculateRecipeRequest {
    proto::CalculateRecipeRequest {
        kind: proto::RecipeKind::AllGrain as i32,
        batch_volume_l: volume,
        boil_volume_l: volume + 3.0,
        boil_time_min: 60.0,
        efficiency_pct: 72.0,
        ibu_equation: proto::IbuEquation::Tinseth as i32,
        equipment: Some(proto::Equipment::default()),
        mash: Some(proto::Mash::default()),
        fermentables: vec![proto::Fermentable {
            name: "Pale".into(),
            amount_kg: 5.0,
            yield_pct: 80.0,
            color_srm: 2.0,
            kind: proto::FermentableKind::Grain as i32,
            usage: proto::FermentableUse::Mash as i32,
            ..Default::default()
        }],
        ..Default::default()
    }
}

async fn setup(pool: &PgPool) -> (PgPool, CalcScope) {
    let user = Uuid::now_v7();
    let ws = Uuid::now_v7();
    insert_user(pool, user).await;
    insert_workspace(pool, ws).await;
    (app_pool(pool).await, CalcScope::new(ws, Some(user)))
}

async fn run(
    eng: &EngineClient,
    app: &PgPool,
    scope: &CalcScope,
    req: proto::CalculateRecipeRequest,
    counter: &AtomicUsize,
) -> api::ApiResult<api::engine::Calculated<proto::RecipeSummary>> {
    let mut tx = db::begin(
        app,
        &db::DbContext::user(scope.user_id.unwrap(), Some(scope.workspace_id)),
    )
    .await
    .unwrap();
    let out = eng
        .calculate(&mut tx, scope, "recipe_summary", req, |ch, r| async move {
            counter.fetch_add(1, Ordering::SeqCst);
            proto::recipe_service_client::RecipeServiceClient::new(ch)
                .calculate(r)
                .await
        })
        .await;
    tx.commit().await.unwrap();
    out
}

async fn rows(base: &PgPool, ws: Uuid) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM calc_results WHERE workspace_id = $1")
        .bind(ws)
        .fetch_one(base)
        .await
        .unwrap()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn same_input_hits_the_engine_once(pool: PgPool) {
    let e = start_engine().await;
    let (app, scope) = setup(&pool).await;
    let eng = client(&e.url, Redis::connect_lazy(&redis_url()).unwrap());
    let n = AtomicUsize::new(0);
    // Unique volume so an old Redis entry of an earlier run cannot match.
    let vol = 20.0 + f64::from(Uuid::now_v7().as_bytes()[15]) / 1000.0;

    let a = run(&eng, &app, &scope, good_request(vol), &n)
        .await
        .unwrap();
    assert_eq!(a.source, Source::Engine);
    assert!(a.output.og_sg > 1.0);
    assert_eq!(a.engine_version, "1.0.0-test");
    let b = run(&eng, &app, &scope, good_request(vol), &n)
        .await
        .unwrap();
    assert_eq!(n.load(Ordering::SeqCst), 1);
    assert_eq!(b.source, Source::Redis);
    assert!((b.output.og_sg - a.output.og_sg).abs() < 1e-12);
    assert_eq!(rows(&pool, scope.workspace_id).await, 1);

    // A different input is a new calculation.
    run(&eng, &app, &scope, good_request(vol + 1.0), &n)
        .await
        .unwrap();
    assert_eq!(n.load(Ordering::SeqCst), 2);
    assert_eq!(rows(&pool, scope.workspace_id).await, 2);
    let _ = e.stop.send(());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn without_redis_the_database_answers(pool: PgPool) {
    let e = start_engine().await;
    let (app, mut scope) = setup(&pool).await;
    scope.target = CalcTarget::None;
    let eng = client(&e.url, Redis::disabled());
    let n = AtomicUsize::new(0);
    run(&eng, &app, &scope, good_request(21.0), &n)
        .await
        .unwrap();
    let b = run(&eng, &app, &scope, good_request(21.0), &n)
        .await
        .unwrap();
    assert_eq!(b.source, Source::Database);
    assert_eq!(n.load(Ordering::SeqCst), 1);
    let _ = e.stop.send(());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn new_engine_version_misses_the_cache(pool: PgPool) {
    let e = start_engine().await;
    let (app, scope) = setup(&pool).await;
    let eng = client(&e.url, Redis::connect_lazy(&redis_url()).unwrap());
    let n = AtomicUsize::new(0);
    let vol = 22.0 + f64::from(Uuid::now_v7().as_bytes()[15]) / 1000.0;
    run(&eng, &app, &scope, good_request(vol), &n)
        .await
        .unwrap();
    *e.version.lock().unwrap() = "2.0.0-test".to_owned();
    let b = run(&eng, &app, &scope, good_request(vol), &n)
        .await
        .unwrap();
    assert_eq!(b.source, Source::Engine);
    assert_eq!(b.engine_version, "2.0.0-test");
    assert_eq!(n.load(Ordering::SeqCst), 2);
    assert_eq!(rows(&pool, scope.workspace_id).await, 2);
    let _ = e.stop.send(());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn engine_validation_error_is_a_field_error(pool: PgPool) {
    let e = start_engine().await;
    let (app, mut scope) = setup(&pool).await;
    scope.field_prefix = "/recipe".to_owned();
    let eng = client(&e.url, Redis::disabled());
    let n = AtomicUsize::new(0);
    let mut bad = good_request(20.0);
    bad.ibu_equation = 0; // UNSPECIFIED
    let err = run(&eng, &app, &scope, bad, &n).await.unwrap_err();
    assert_eq!(err.code(), ErrorCode::CalculationFailed);
    assert_eq!(err.errors()[0].field, "/recipe/ibu_equation");
    assert_eq!(err.errors()[0].code, "invalid_input");
    assert_eq!(rows(&pool, scope.workspace_id).await, 0);
    let _ = e.stop.send(());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn engine_down_is_503(pool: PgPool) {
    let (app, scope) = setup(&pool).await;
    // Nothing listens on this port.
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", l.local_addr().unwrap());
    drop(l);
    let eng = client(&url, Redis::disabled());
    let n = AtomicUsize::new(0);
    let err = run(&eng, &app, &scope, good_request(20.0), &n)
        .await
        .unwrap_err();
    assert_eq!(err.code(), ErrorCode::Unavailable);
    assert_eq!(n.load(Ordering::SeqCst), 0);

    let err = run(
        &EngineClient::disabled(),
        &app,
        &scope,
        good_request(20.0),
        &n,
    )
    .await
    .unwrap_err();
    assert_eq!(err.code(), ErrorCode::Unavailable);
}
