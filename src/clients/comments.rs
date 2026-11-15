pub mod comments_grpc {
    tonic::include_proto!("comments");
}
use comments_grpc::comments_client;
use tonic::transport::Channel;

pub type CommentsClient = comments_client::CommentsClient<Channel>;

impl CommentsClient {
    pub async fn build(comments_api: String) -> Result<Self, tonic::transport::Error> {
        comments_client::CommentsClient::connect(comments_api).await
    }
}
