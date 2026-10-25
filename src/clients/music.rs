pub mod music_grpc {
    tonic::include_proto!("music");
}
use music_grpc::music_client;
use tonic::transport::Channel;

pub type MusicClient = music_client::MusicClient<Channel>;

impl MusicClient {
    pub async fn build(music_api: String) -> Result<Self, tonic::transport::Error> {
        music_client::MusicClient::connect(music_api).await
    }
}
