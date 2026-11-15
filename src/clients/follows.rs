pub mod follows_grpc {
    tonic::include_proto!("follows");
}
use follows_grpc::follows_client;
use tonic::transport::Channel;

pub type FollowsClient = follows_client::FollowsClient<Channel>;

impl FollowsClient {
    pub async fn build(follows_api: String) -> Result<Self, tonic::transport::Error> {
        follows_client::FollowsClient::connect(follows_api).await
    }
}
