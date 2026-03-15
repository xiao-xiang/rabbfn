use rabbfn::extract::Error;
use rabbfn::{Json, RabbitMqServer, TopologyMode, consumer};

#[derive(Debug, serde::Deserialize)]
struct HeaderEvent {
    title: String,
}

fn build_headers_args() -> lapin::types::FieldTable {
    let mut args = lapin::types::FieldTable::default();
    args.insert("x-match".into(), lapin::types::AMQPValue::LongString("all".into()));
    args.insert("module".into(), lapin::types::AMQPValue::LongString("order".into()));
    args
}

#[consumer(
    queue(name = "headers.q"),
    exchanges = [(name = "headers.ex", kind = "headers")],
    bindings = [(
        exchange = "headers.ex",
        arguments = build_headers_args()
    )]
)]
async fn headers_handler(Json(msg): Json<HeaderEvent>) -> Result<(), Error> {
    println!("headers={}", msg.title);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    RabbitMqServer::new("amqp://guest:guest@127.0.0.1:5672/%2f")
        .with_topology_mode(TopologyMode::Managed)
        .add_service(headers_handler)
        .run()
        .await?;
    Ok(())
}
