use rabbfn::extract::Error;
use rabbfn::{Json, RabbitMqServer, TopologyMode, consumer};

#[derive(Debug, serde::Deserialize)]
struct CreateOrderReq {
    order_id: String,
}

#[derive(Debug, serde::Serialize)]
struct CreateOrderResp {
    ok: bool,
    order_id: String,
}

#[consumer(
    queue(name = "rpc.order.create"),
    consume_options(consumer_tag = "rpc-order-worker")
)]
async fn rpc_create_order(
    channel: lapin::Channel,
    raw: rabbfn::extract::RawDelivery,
    Json(req): Json<CreateOrderReq>
) -> Result<(), Error> {
    let resp = CreateOrderResp {
        ok: true,
        order_id: req.order_id,
    };
    if let Some(reply_to) = raw.0.properties.reply_to().as_ref() {
        let payload = serde_json::to_vec(&resp).map_err(Error::Json)?;
        channel
            .basic_publish(
                "",
                reply_to.as_str(),
                lapin::options::BasicPublishOptions::default(),
                &payload,
                lapin::BasicProperties::default(),
            )
            .await
            .map_err(Error::Amqp)?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _server = RabbitMqServer::new("amqp://guest:guest@127.0.0.1:5672/%2f")
        .with_topology_mode(TopologyMode::Managed)
        .add_service(RpcCreateOrderConsumer::new().with_state(()))
        .run().await;
    Ok(())
}
