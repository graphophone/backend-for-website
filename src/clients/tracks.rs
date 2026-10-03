pub mod tracks_grpc {
    tonic::include_proto!("tracks");
}
use tonic::transport::Channel;
use tracks_grpc::tracks_client;

pub type TracksClient = tracks_client::TracksClient<Channel>;

impl TracksClient {
    pub async fn build(tracks_api: String) -> Result<Self, tonic::transport::Error> {
        tracks_client::TracksClient::connect(tracks_api).await
    }
}
