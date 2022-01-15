#[async_trait::async_trait]
pub trait QueueType<T> {
    const EXCHANGE: &'static str;
    const QUEUE: &'static str;

    async fn init_producer(chan: &lapin::Channel) -> crate::Result<()>;
    async fn init_consumer(chan: &lapin::Channel) -> crate::Result<()>;

    fn publish_opts(msg: &T) -> lapin::options::BasicPublishOptions;
    fn properties(msg: &T) -> lapin::BasicProperties;
}
