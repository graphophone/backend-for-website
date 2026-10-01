pub mod categories_grpc {
    tonic::include_proto!("categories");
}
use anyhow::Result;
use categories_grpc::categories_client;
use tonic::transport::Channel;

pub type CategoriesClient = categories_client::CategoriesClient<Channel>;

impl CategoriesClient {
    pub async fn build(categories_api: String) -> Result<Self> {
        let client = categories_client::CategoriesClient::connect(categories_api).await?;
        Ok(client)
    }
}