use rabbfn::extract::Error;
use rabbfn::{Json, RabbitMqServer, TopologyMode, consumer};

#[derive(Debug, serde::Deserialize)]
struct OrderDelayMsg {
    order_id: String,
}

fn build_delay_queue_args() -> lapin::types::FieldTable {
    let mut args = lapin::types::FieldTable::default();
    args.insert(
        "x-dead-letter-exchange".into(),
        lapin::types::AMQPValue::LongString("order.dead.ex".into()),
    );
    args.insert(
        "x-dead-letter-routing-key".into(),
        lapin::types::AMQPValue::LongString("order.dead".into()),
    );
    args.insert("x-message-ttl".into(), lapin::types::AMQPValue::LongUInt(30_000));
    args
}

#[consumer(
    queue(
        name = "order.delay.q",
        arguments = build_delay_queue_args()
    ),
    exchanges = [(name = "order.delay.ex", kind = "direct")],
    bindings = [(exchange = "order.delay.ex", routing_key = "order.delay")]
)]
async fn delay_consumer(Json(msg): Json<OrderDelayMsg>) -> Result<(), Error> {
    println!("delay={}", msg.order_id);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _server = RabbitMqServer::new("amqp://guest:guest@127.0.0.1:5672/%2f")
        .with_topology_mode(TopologyMode::Managed)
        .add_service(DelayConsumerConsumer::new().with_state(()))
        .run().await?;;
    Ok(())
}
