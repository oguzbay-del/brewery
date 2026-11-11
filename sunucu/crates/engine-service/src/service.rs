//! `EngineService` implementation.

use tonic::{Request, Response, Status};

use crate::proto::{
    GetVersionRequest, GetVersionResponse,
    engine_service_server::{EngineService, EngineServiceServer},
};

/// Handler implementing the engine gRPC service.
#[derive(Debug, Default, Clone, Copy)]
pub struct EngineHandler;

#[tonic::async_trait]
impl EngineService for EngineHandler {
    async fn get_version(
        &self,
        _request: Request<GetVersionRequest>,
    ) -> Result<Response<GetVersionResponse>, Status> {
        Ok(Response::new(GetVersionResponse {
            version: engine::version().to_owned(),
        }))
    }
}

/// Builds the tonic server wrapper for the engine handler.
#[must_use]
pub fn server() -> EngineServiceServer<EngineHandler> {
    EngineServiceServer::new(EngineHandler)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{net::TcpListener, sync::oneshot};
    use tokio_stream::wrappers::TcpListenerStream;
    use tonic::transport::Server;
    use tonic_health::pb::{
        HealthCheckRequest, health_check_response::ServingStatus, health_client::HealthClient,
    };

    #[tokio::test]
    async fn get_version_returns_engine_version() {
        let response = EngineHandler
            .get_version(Request::new(GetVersionRequest {}))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(response.version, engine::version());
    }

    #[tokio::test]
    async fn grpc_health_service_reports_serving() {
        let (reporter, health_service) = tonic_health::server::health_reporter();
        reporter
            .set_serving::<EngineServiceServer<EngineHandler>>()
            .await;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (stop_tx, stop_rx) = oneshot::channel::<()>();
        let server = tokio::spawn(async move {
            Server::builder()
                .add_service(health_service)
                .add_service(server())
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                    let _ = stop_rx.await;
                })
                .await
        });

        let channel = tonic::transport::Endpoint::from_shared(format!("http://{addr}"))
            .unwrap()
            .connect()
            .await
            .unwrap();
        let mut client = HealthClient::new(channel);
        let response = client
            .check(HealthCheckRequest {
                service: String::new(),
            })
            .await
            .unwrap()
            .into_inner();
        assert_eq!(response.status(), ServingStatus::Serving);

        let _ = stop_tx.send(());
        server.await.unwrap().unwrap();
    }
}
