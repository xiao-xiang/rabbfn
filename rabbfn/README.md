## RabbitMQ Usage Guide and Full-Pattern Demo

### Startup Modes

`rabbitmq-server` supports two topology modes:

- `TopologyMode::Managed`: declare exchange/queue/binding automatically when the app starts.
- `TopologyMode::External`: topology is pre-created by external systems, and the app only consumes.

```rust
use rabbitmq_server::{RabbitMqServer, TopologyMode};

let server = RabbitMqServer::new("amqp://guest:guest@127.0.0.1:5672/%2f")
    .with_topology_mode(TopologyMode::Managed);
```

### Parameter Style

It is recommended to use `queue(...)` consistently:

```rust
#[consumer(
    queue(name = "order.created", durable = true, auto_delete = false),
    qos(prefetch_count = 20)
)]
async fn handle(Json(msg): Json<OrderCreated>) -> Result<(), Error> {
    println!("{:?}", msg);
    Ok(())
}
```

Notes for the `exchanges` parameter:

- Use `exchanges = [ (...), (...) ]` in macro arguments.
- Each list item uses named arguments: `(name = "...", kind = "...", ...)`.
- The same syntax works for both single and multiple exchanges.

Example for multiple exchanges:

```rust
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
    println!("event={:?}", msg);
    Ok(())
}
```

Practical recommendations:

- With 2–3 exchanges, `exchanges=[(...),(...)]` is usually readable.
- For many exchanges (for example, >3), prefer external topology management (`TopologyMode::External`) and keep only consume/binding semantics in macros to avoid overly long annotations.

Default values for fields not explicitly set in `queue`:

- `passive=false`
- `durable=true`
- `exclusive=false`
- `auto_delete=false`
- `nowait=false`
- `declare=true`
- `arguments=FieldTable::default()`

---

### 1) Work Queue (Simple Queue / Competing Consumers)

```rust
#[consumer(
    queue(name = "task.queue"),
    qos(prefetch_count = 10)
)]
async fn task_worker(Json(task): Json<TaskPayload>) -> Result<(), Error> {
    println!("task={:?}", task);
    Ok(())
}
```

### 2) Publish/Subscribe (Fanout Broadcast)

```rust
#[consumer(
    queue(name = "notify.sms"),
    exchanges = [(name = "notify.fanout", kind = "fanout")],
    bindings = [(exchange = "notify.fanout")]
)]
async fn sms_worker(Json(msg): Json<NotifyPayload>) -> Result<(), Error> {
    println!("sms={:?}", msg);
    Ok(())
}
```

### 3) Routing (Direct Routing)

```rust
#[consumer(
    queue(name = "pay.success.q"),
    exchanges = [(name = "pay.direct", kind = "direct")],
    bindings = [(exchange = "pay.direct", routing_key = "pay.success")]
)]
async fn pay_success(Json(msg): Json<PayEvent>) -> Result<(), Error> {
    println!("pay={:?}", msg);
    Ok(())
}
```

### 4) Topic (Wildcard Routing)

```rust
#[consumer(
    queue(name = "order.topic.q"),
    exchanges = [(name = "order.topic", kind = "topic")],
    bindings = [(exchange = "order.topic", routing_key = "order.*")]
)]
async fn order_topic(Json(msg): Json<OrderEvent>) -> Result<(), Error> {
    println!("topic={:?}", msg);
    Ok(())
}
```

### 5) Headers (Header Matching)

```rust
#[consumer(
    queue(name = "headers.q"),
    exchanges = [(name = "headers.ex", kind = "headers")],
    bindings = [(
        exchange = "headers.ex",
        arguments = build_headers_args()
    )]
)]
async fn headers_handler(Json(msg): Json<HeaderEvent>) -> Result<(), Error> {
    println!("headers={:?}", msg);
    Ok(())
}
```

### 6) RPC (Request-Response)

```rust
#[consumer(
    queue(name = "rpc.order.create"),
    consume_options(consumer_tag = "rpc-order-worker")
)]
async fn rpc_create_order(
    channel: lapin::Channel,
    raw: rabbitmq_server::extract::RawDelivery,
    Json(req): Json<CreateOrderReq>
) -> Result<(), Error> {
    let resp = CreateOrderResp { ok: true };
    if let Some(reply_to) = raw.0.properties.reply_to().as_ref() {
        let payload = serde_json::to_vec(&resp).map_err(Error::Json)?;
        channel.basic_publish(
            "",
            reply_to.as_str(),
            lapin::options::BasicPublishOptions::default(),
            &payload,
            lapin::BasicProperties::default()
        ).await.map_err(Error::Amqp)?;
    }
    Ok(())
}
```

### 6.1) Manual Ack

To use manual ack, set `consume_options(no_ack = false)` (the default is also `false`).  
The framework automatically acks on handler success and nacks on failure.  
If you manually ack/nack in your handler, make sure your team has a clear convention to avoid duplicate acknowledgements.

```rust
#[consumer(
    queue(name = "order.manual.ack.q"),
    consume_options(
        consumer_tag = "manual-ack-worker",
        no_ack = false
    )
)]
async fn ack_handler(
    raw: rabbitmq_server::extract::RawDelivery,
    Json(msg): Json<OrderEvent>
) -> Result<(), Error> {
    if msg.order_id.is_empty() {
        raw.0
            .nack(lapin::options::BasicNackOptions::default())
            .await
            .map_err(Error::Amqp)?;
        return Err(Error::Other("invalid order_id".to_string()));
    }

    println!("manual-ack event={:?}", msg);

    raw.0
        .ack(lapin::options::BasicAckOptions::default())
        .await
        .map_err(Error::Amqp)?;

    Ok(())
}
```

Notes:

- `no_ack = true` means broker auto-ack; neither framework nor business code should ack/nack.
- When `no_ack = false`, the framework automatically ack/nack based on handler result by default.

### 7) DLX / TTL / Delay (Dead Letter and Delay)

```rust
#[consumer(
    queue(
        name = "order.delay.q",
        arguments = build_delay_queue_args()
    ),
    exchanges = [(name = "order.delay.ex", kind = "direct")],
    bindings = [(exchange = "order.delay.ex", routing_key = "order.delay")]
)]
async fn delay_consumer(Json(msg): Json<OrderDelayMsg>) -> Result<(), Error> {
    println!("delay={:?}", msg);
    Ok(())
}
```

---

### bindings List Syntax (Recommended)

```rust
#[consumer(
    queue(name = "multi.bind.q"),
    bindings = [
        (exchange = "order.topic", routing_key = "order.created"),
        (exchange = "order.topic", routing_key = "order.updated", nowait = true, arguments = lapin::types::FieldTable::default())
    ]
)]
async fn multi_bindings(Json(msg): Json<OrderEvent>) -> Result<(), Error> {
    println!("{:?}", msg);
    Ok(())
}
```
