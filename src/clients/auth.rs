pub mod auth_grpc {
    tonic::include_proto!("auth");
}
use anyhow::Result;
use auth_grpc::auth_client;
use tonic::transport::Channel;

pub type AuthClient = auth_client::AuthClient<Channel>;

impl AuthClient {
    pub async fn build(auth_api: String) -> Result<Self> {
        let client = auth_client::AuthClient::connect(auth_api).await?;
        Ok(client)
    }
}