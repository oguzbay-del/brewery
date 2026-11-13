//! Engine gRPC service binary.

use std::net::SocketAddr;

use common::{ServiceConfig, telemetry};
use engine_service::{fermentation, mash, recipe, service, tool};
use tonic::transport::Server;

const DEFAULT_NAME: &str = "engine-service";
const DEFAULT_PORT: u16 = 50051;

#[tokio::main]
async fn main() -> common::Result<()> {
    let config = ServiceConfig::from_env(DEFAULT_NAME, DEFAULT_PORT)?;
    telemetry::init_tracing(&config.service_name)?;

    let (reporter, health_service) = tonic_health::server::health_reporter();
    reporter
        .set_serving::<engine_service::proto::engine_service_server::EngineServiceServer<
            service::EngineHandler,
        >>()
        .await;
    reporter
        .set_serving::<engine_service::proto::tool_service_server::ToolServiceServer<
            tool::ToolHandler,
        >>()
        .await;

    reporter
        .set_serving::<engine_service::proto::recipe_service_server::RecipeServiceServer<
            recipe::RecipeHandler,
        >>()
        .await;

    reporter
        .set_serving::<engine_service::proto::mash_service_server::MashServiceServer<
            mash::MashHandler,
        >>()
        .await;

    reporter
        .set_serving::<engine_service::proto::fermentation_service_server::FermentationServiceServer<
            fermentation::FermentationHandler,
        >>()
        .await;

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!(%addr, version = engine::version(), "engine-service listening");

    Server::builder()
        .add_service(health_service)
        .add_service(service::server())
        .add_service(tool::server())
        .add_service(recipe::server())
        .add_service(mash::server())
        .add_service(fermentation::server())
        .serve_with_shutdown(addr, common::shutdown::shutdown_signal())
        .await
        .map_err(|e| common::Error::Io(std::io::Error::other(e)))?;
    tracing::info!("engine-service stopped");
    Ok(())
}
