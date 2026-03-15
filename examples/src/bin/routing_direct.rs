use rabbfn::extract::Error;
use rabbfn::{Json, RabbitMqServer, TopologyMode, consumer};

#[derive(Debug, serde::Deserialize)]
struct PayEvent {
    order_id: String,
}

#[consumer(
    queue(name = "pay.success.q"),
    exchanges = [(name = "pay.direct", kind = "direct")],
    bindings = [(exchange = "pay.direct", routing_key = "pay.success")]
)]
async fn pay_success(Json(msg): Json<PayEvent>) -> Result<(), Error> {
    println!("pay={}", msg.order_id);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _server = RabbitMqServer::new("amqp://guest:guest@127.0.0.1:5672/%2f")
        .with_topology_mode(TopologyMode::Managed)
        .add_service(PaySuccessConsumer::new().with_state(()))
        .run().await;
    Ok(())
}
