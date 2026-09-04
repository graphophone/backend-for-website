pub mod auth {
    tonic::include_proto!("auth");
}
use anyhow::Result;
use auth::auth_service_client::AuthServiceClient;
use tonic::transport::Channel;

pub struct AuthClient {
    client: AuthServiceClient<Channel>,
}

impl AuthClient {
    pub async fn build(auth_api: String) -> Result<Self> {
        let client = AuthServiceClient::connect(auth_api)
            .await?;
        Ok(AuthClient { client })
    }
}