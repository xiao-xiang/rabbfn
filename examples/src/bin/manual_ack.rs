use rabbfn::extract::Error;
use rabbfn::{Json, RabbitMqServer, TopologyMode, consumer};

#[derive(Debug, serde::Deserialize)]
struct OrderEvent {
    order_id: String,
}

#[consumer(
    queue(name = "order.manual.ack.q"),
    consume_options(
        consumer_tag = "manual-ack-worker",
        no_ack = false
    )
)]
async fn ack_handler(
    raw: rabbfn::extract::RawDelivery,
    Json(msg): Json<OrderEvent>
) -> Result<(), Error> {
    if msg.order_id.is_empty() {
        raw.0
            .nack(lapin::options::BasicNackOptions::default())
            .await
            .map_err(Error::Amqp)?;
        return Err(Error::Other("invalid order_id".to_string()));
    }

    println!("manual-ack event={}", msg.order_id);

    raw.0
        .ack(lapin::options::BasicAckOptions::default())
        .await
        .map_err(Error::Amqp)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    RabbitMqServer::new("amqp://guest:guest@127.0.0.1:5672/%2f")
        .with_topology_mode(TopologyMode::Managed)
        .add_consumer(ack_handler)
        .run()
        .await?;
    Ok(())
}
