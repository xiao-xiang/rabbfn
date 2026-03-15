## RabbitMQ 使用说明与全模式 Demo

### 启动方式

`rabbitmq-server` 支持两种拓扑模式：

- `TopologyMode::Managed`：应用启动时自动声明 exchange/queue/binding。
- `TopologyMode::External`：拓扑由外部系统预建，应用只消费。

```rust
use rabbitmq_server::{RabbitMqServer, TopologyMode};

let server = RabbitMqServer::new("amqp://guest:guest@127.0.0.1:5672/%2f")
    .with_topology_mode(TopologyMode::Managed);
```

### 参数风格

推荐统一使用 `queue(...)`：

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

关于 `exchanges` 参数说明：

- 宏参数统一使用 `exchanges = [ (...), (...) ]`
- 每个列表项为命名参数风格：`(name = "...", kind = "...", ...)`
- 支持单个与多个 exchange，统一同一种写法

多 exchange 场景示例：

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

实践建议：

- 2~3 个 exchange 时，`exchanges=[(...),(...)]` 可读性较好。
- exchange 数量较多（例如 >3）时，建议用外部拓扑管理（`TopologyMode::External`），业务宏里只保留消费与绑定语义，避免注解过长。

`queue` 中未显式填写的字段使用默认值：

- `passive=false`
- `durable=true`
- `exclusive=false`
- `auto_delete=false`
- `nowait=false`
- `declare=true`
- `arguments=FieldTable::default()`

---

### 1) Work Queue（简单队列/竞争消费）

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

### 2) Publish/Subscribe（Fanout 广播）

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

### 3) Routing（Direct 路由）

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

### 4) Topic（通配路由）

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

### 5) Headers（Header 匹配）

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

### 6) RPC（请求-响应）

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

### 6.1) 手动 Ack（Manual Ack）

使用手动 Ack 时，配置 `consume_options(no_ack = false)` 即可（默认也是 false），框架会在 handler 成功后自动 ack、失败时自动 nack。若你在 handler 中手动 ack/nack，请确保业务约定一致，避免重复确认：

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

注意：

- `no_ack = true` 表示 broker 自动确认，框架与业务代码都不应再 ack/nack。
- `no_ack = false` 时，框架默认会根据 handler 结果自动 ack/nack。

### 7) DLX / TTL / Delay（死信与延迟）

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

### bindings 列表语法（推荐）

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
