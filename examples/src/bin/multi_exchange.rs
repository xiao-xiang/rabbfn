use rabbfn::extract::Error;
use rabbfn::{Json, RabbitMqServer, TopologyMode, consumer};

#[derive(Debug, serde::Deserialize)]
struct OrderEvent {
    event_name: String,
}

#[consumer(
    queue(name = "order.unified.q"),
    exchanges = [
        (name = "order.topic.ex", kind = "topic"),
        (name = "order.audit.ex", kind = "fanout")
    ],
    bindings = [
        (exchange = "order.topic.ex", routing_key = "order.created"),
        (exchange = "order.audit.ex")
    ]
)]
async fn multi_exchange_handler(Json(msg): Json<OrderEvent>) -> Result<(), Error> {
    println!("event={}", msg.event_name);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    RabbitMqServer::new("amqp://guest:guest@127.0.0.1:5672/%2f")
        .with_topology_mode(TopologyMode::Managed)
        .add_consumer(multi_exchange_handler)
        .run()
        .await?;
    Ok(())
}
