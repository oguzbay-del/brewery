//! End-to-end check: every engine service answers over a real gRPC connection.

use engine_service::proto::{
    BlendRequest, CalibrateTiltRequest, DescribeProfileRequest, IonProfile, RecipeKind,
    TotalWaterRequest, WineStabilizersRequest, brewery_service_client::BreweryServiceClient,
    fermentation_service_client::FermentationServiceClient, mash_service_client::MashServiceClient,
    water_service_client::WaterServiceClient, yeast_service_client::YeastServiceClient,
};
use engine_service::{brewery, fermentation, mash, water, yeast};
use tokio::{net::TcpListener, sync::oneshot};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{Code, Status, transport::Channel, transport::Server};

/// The call reached the handler: success or a validation error, never "unimplemented".
fn routed<T>(r: Result<T, Status>) {
    if let Err(e) = r {
        assert!(
            matches!(e.code(), Code::InvalidArgument | Code::FailedPrecondition),
            "unexpected status: {e:?}"
        );
    }
}

#[tokio::test]
async fn all_services_answer_over_grpc() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (stop_tx, stop_rx) = oneshot::channel::<()>();
    let server = tokio::spawn(async move {
        Server::builder()
            .add_service(mash::server())
            .add_service(water::server())
            .add_service(yeast::server())
            .add_service(fermentation::server())
            .add_service(brewery::server())
            .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                let _ = stop_rx.await;
            })
            .await
    });
    let channel: Channel = tonic::transport::Endpoint::from_shared(format!("http://{addr}"))
        .unwrap()
        .connect()
        .await
        .unwrap();

    // All grain: mash + sparge + top-ups (spec 05 §5.8).
    let total = MashServiceClient::new(channel.clone())
        .total_water(TotalWaterRequest {
            kind: RecipeKind::AllGrain as i32,
            mash_water_l: 16.0,
            sparge_l: 19.0,
            topup_kettle_l: 1.0,
            topup_fermenter_l: 2.0,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert!((total.total_water_l - 38.0).abs() < 1e-9);

    let stats = WaterServiceClient::new(channel.clone())
        .describe_profile(DescribeProfileRequest {
            ions: Some(IonProfile {
                sulfate_ppm: 100.0,
                chloride_ppm: 50.0,
                ..Default::default()
            }),
        })
        .await
        .unwrap()
        .into_inner()
        .stats
        .unwrap();
    assert!((stats.sulfate_chloride_ratio - 2.0).abs() < 1e-9);

    routed(
        YeastServiceClient::new(channel.clone())
            .wine_stabilizers(WineStabilizersRequest::default())
            .await,
    );
    routed(
        FermentationServiceClient::new(channel.clone())
            .calibrate_tilt(CalibrateTiltRequest::default())
            .await,
    );
    routed(
        BreweryServiceClient::new(channel)
            .blend(BlendRequest::default())
            .await,
    );

    let _ = stop_tx.send(());
    server.await.unwrap().unwrap();
}
