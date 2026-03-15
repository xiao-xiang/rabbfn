use rabbfn::extract::Error;
use rabbfn::{Json, RabbitMqServer, State, TopologyMode, consumer};

#[derive(Debug, serde::Deserialize)]
struct OrderEvent {
    event_name: String,
}

#[consumer(
    queue(name = "order.topic.q"),
    exchanges = [(name = "order.topic", kind = "topic")],
    bindings = [(exchange = "order.topic", routing_key = "order.*")]
)]
async fn order_topic(State(source): State<String>, Json(msg): Json<OrderEvent>) -> Result<(), Error> {
    println!("topic={} source={}", msg.event_name, source);
    Ok(())
}

#[consumer(
    queue(name = "order.topic.a"),
    exchanges = [(name = "order.topic", kind = "topic")],
    bindings = [(exchange = "order.topic", routing_key = "order.*")]
)]
async fn order_topic_test(State(source): State<String>, Json(msg): Json<OrderEvent>) -> Result<(), Error> {
    println!("topic={} source={}", msg.event_name, source);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    RabbitMqServer::new("amqp://guest:guest@127.0.0.1:5672/%2f")
        .with_topology_mode(TopologyMode::Managed)
        .add_consumer(order_topic)
        .with_state("topic-consumer".to_string())
        .add_consumer(order_topic_test).with_state("topic-test".to_string())
        .run()
        .await?;
    Ok(())
}
