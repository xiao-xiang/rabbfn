use rabbfn::extract::Error;
use rabbfn::{Json, RabbitMqServer, TopologyMode, consumer};

#[derive(Debug, serde::Deserialize)]
struct OrderEvent {
    event_name: String,
}

#[consumer(
    queue(name = "multi.bind.q"),
    bindings = [
        (exchange = "order.topic", routing_key = "order.created"),
        (exchange = "order.topic", routing_key = "order.updated", nowait = true, arguments = lapin::types::FieldTable::default())
    ]
)]
async fn multi_bindings(Json(msg): Json<OrderEvent>) -> Result<(), Error> {
    println!("binding={}", msg.event_name);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    RabbitMqServer::new("amqp://guest:guest@127.0.0.1:5672/%2f")
        .with_topology_mode(TopologyMode::External)
        .add_service(multi_bindings)
        .run()
        .await?;
    Ok(())
}
