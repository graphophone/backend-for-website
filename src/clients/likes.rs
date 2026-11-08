pub mod likes_grpc {
    tonic::include_proto!("likes");
}
use likes_grpc::likes_client;
use tonic::transport::Channel;

pub type LikesClient = likes_client::LikesClient<Channel>;

impl LikesClient {
    pub async fn build(likes_api: String) -> Result<Self, tonic::transport::Error> {
        likes_client::LikesClient::connect(likes_api).await
    }
}
