use rabbfn::extract::Error;
use rabbfn::{Json, RabbitMqServer, TopologyMode, consumer};

#[derive(Debug, serde::Deserialize)]
struct TaskPayload {
    id: String,
}

#[consumer(
    queue(name = "task.queue"),
    qos(prefetch_count = 10)
)]
async fn task_worker(Json(task): Json<TaskPayload>) -> Result<(), Error> {
    println!("task={}", task.id);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    RabbitMqServer::new("amqp://guest:guest@127.0.0.1:5672/%2f")
        .with_topology_mode(TopologyMode::Managed)
        .add_service(task_worker)
        .run()
        .await?;
    Ok(())
}
