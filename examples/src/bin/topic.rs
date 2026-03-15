use rabbfn as rabb_fn;
use rabb_fn::extract::Error;
use rabb_fn::{Json, RabbitMqServer, TopologyMode, consumer};

#[derive(Debug, serde::Deserialize)]
struct OrderEvent {
    event_name: String,
}

#[consumer(
    queue(name = "order.topic.q"),
    exchanges = [(name = "order.topic", kind = "topic")],
    bindings = [(exchange = "order.topic", routing_key = "order.*")]
)]
async fn order_topic(Json(msg): Json<OrderEvent>) -> Result<(), Error> {
    println!("topic={}", msg.event_name);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _server = RabbitMqServer::new("amqp://guest:guest@127.0.0.1:5672/%2f")
        .with_topology_mode(TopologyMode::Managed)
        .add_service(OrderTopicConsumer::new().with_state(()));
    Ok(())
}
