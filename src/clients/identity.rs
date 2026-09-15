pub mod identity_grpc {
    tonic::include_proto!("identity");
}
use tonic::transport::Channel;
use identity_grpc::identity_client;

pub type IdentityClient = identity_client::IdentityClient<Channel>;

impl IdentityClient {
    pub async fn build(identity_api: String) -> Result<Self, tonic::transport::Error> {
        identity_client::IdentityClient::connect(identity_api).await
    }
}