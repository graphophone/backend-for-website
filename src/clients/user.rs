pub mod user {
    tonic::include_proto!("user");
}
use anyhow::Result;
use tonic::transport::Channel;
use user::user_service_client::UserServiceClient;

#[derive(Clone)]
pub struct UserClient {
    client: UserServiceClient<Channel>,
}

impl UserClient {
    pub async fn build(user_api: String) -> Result<Self> {
        let client = UserServiceClient::connect(user_api).await?;
        Ok(UserClient { client })
    }
}