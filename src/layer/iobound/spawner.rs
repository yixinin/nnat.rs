use super::io::BiStream;
pub trait Spawner<T>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Send + 'static,
{
    fn spawn(self) -> impl std::future::Future<Output = std::io::Result<BiStream<T>>> + Send;
}
