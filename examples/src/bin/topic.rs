use rabbfn::extract::Error;
use rabbfn::{Json, RabbitMqServer, TopologyMode, consumer};

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
    let _server = RabbitMqServer::new("amqp://licong:18258453478lc@www.soyue.top:5672")
        .with_topology_mode(TopologyMode::Managed)
        .add_service(OrderTopicConsumer::new().with_state(()))
        .run().await?;
    Ok(())
}
