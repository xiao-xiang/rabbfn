use rabbfn::extract::Error;
use rabbfn::{Json, RabbitMqServer, TopologyMode, consumer};

#[derive(Debug, serde::Deserialize)]
struct NotifyPayload {
    content: String,
}

#[consumer(
    queue(name = "notify.sms"),
    exchanges = [(name = "notify.fanout", kind = "fanout")],
    bindings = [(exchange = "notify.fanout")]
)]
async fn sms_worker(Json(msg): Json<NotifyPayload>) -> Result<(), Error> {
    println!("sms={}", msg.content);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    RabbitMqServer::new("amqp://guest:guest@127.0.0.1:5672/%2f")
        .with_topology_mode(TopologyMode::Managed)
        .add_consumer(sms_worker)
        .run()
        .await?;
    Ok(())
}
