use std::sync::Arc;
use lapin::{Connection, ConnectionProperties, ExchangeKind};
use lapin::options::{BasicConsumeOptions, BasicAckOptions, BasicNackOptions, BasicQosOptions, QueueBindOptions, QueueDeclareOptions, ExchangeDeclareOptions};
use futures::StreamExt;
use tower::{Service, ServiceExt};
use crate::service::MqRequest;
use crate::extract::{Error, MqContext};
use tower::util::BoxCloneService;
use crate::config::{ConsumerConfig, BindingConfig, QosConfig, QueueConfig, ExchangeConfig, ConsumeConfig, TopologyMode};

pub struct RabbitMqServer {
    url: String,
    topology_mode: TopologyMode,
    consumers: Vec<ConsumerDescriptor>,
}

struct ConsumerDescriptor {
    queue_config: QueueConfig,
    exchanges: Vec<ExchangeConfig>,
    concurrency: usize,
    qos: QosConfig,
    consume_config: ConsumeConfig,
    bindings: Vec<BindingConfig>,
    service: BoxCloneService<MqRequest, (), Error>,
}

impl RabbitMqServer {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            topology_mode: TopologyMode::Managed,
            consumers: Vec::new(),
        }
    }

    pub fn with_topology_mode(mut self, mode: TopologyMode) -> Self {
        self.topology_mode = mode;
        self
    }

    pub fn add_consumer<S>(mut self, service: S) -> Self
    where
        S: Service<MqRequest, Response = (), Error = Error> + ConsumerConfig + Clone + Send + Sync + 'static,
        S::Future: Send + 'static,
    {
        let queue_config = service.queue_config();
        let exchanges = service.exchanges();
        let concurrency = service.concurrency();
        let qos = service.qos();
        let consume_config = service.consume_config();
        let bindings = service.bindings();
        
        let boxed_svc = BoxCloneService::new(service);
        self.consumers.push(ConsumerDescriptor {
            queue_config,
            exchanges,
            concurrency,
            qos,
            consume_config,
            bindings,
            service: boxed_svc,
        });
        self
    }

    pub async fn run(self) -> Result<(), Error> {
        let conn = Connection::connect(&self.url, ConnectionProperties::default()).await.map_err(Error::Amqp)?;
        let conn = Arc::new(conn);
        
        if let TopologyMode::Managed = self.topology_mode {
            let channel = conn.create_channel().await.map_err(Error::Amqp)?;
            for consumer in &self.consumers {
                    for exchange in &consumer.exchanges {
                        if exchange.declare {
                            channel.exchange_declare(
                                &exchange.name,
                                parse_exchange_kind(&exchange.kind)?,
                                ExchangeDeclareOptions {
                                    passive: exchange.passive,
                                    durable: exchange.durable,
                                    auto_delete: exchange.auto_delete,
                                    internal: exchange.internal,
                                    nowait: exchange.nowait,
                                },
                                exchange.arguments.clone(),
                            ).await.map_err(Error::Amqp)?;
                        }
                    }

                    if consumer.queue_config.declare {
                        channel.queue_declare(
                            &consumer.queue_config.name,
                            QueueDeclareOptions {
                                passive: consumer.queue_config.passive,
                                durable: consumer.queue_config.durable,
                                exclusive: consumer.queue_config.exclusive,
                                auto_delete: consumer.queue_config.auto_delete,
                                nowait: consumer.queue_config.nowait,
                            },
                            consumer.queue_config.arguments.clone(),
                        ).await.map_err(Error::Amqp)?;
                    }

                    for binding in &consumer.bindings {
                        channel.queue_bind(
                            &consumer.queue_config.name,
                            &binding.exchange,
                            &binding.routing_key,
                            QueueBindOptions {
                                nowait: binding.nowait,
                            },
                            binding.arguments.clone(),
                        ).await.map_err(Error::Amqp)?;
                    }
                }
        }

        let mut handles = Vec::new();

        for consumer in self.consumers {
            for i in 0..consumer.concurrency {
                let conn = conn.clone();
                let queue = consumer.queue_config.name.clone();
                let qos = consumer.qos.clone();
                let consume_config = consumer.consume_config.clone();
                let mut service = consumer.service.clone();
                let tag = if consume_config.consumer_tag.is_empty() {
                    format!("{}_{}", queue, i)
                } else {
                    format!("{}_{}", consume_config.consumer_tag, i)
                };

                let handle = tokio::spawn(async move {
                    if let Ok(channel) = conn.create_channel().await {
                         let _ = channel.basic_qos(qos.prefetch_count, BasicQosOptions { global: qos.global, ..Default::default() }).await;
                         let stream_res = channel.basic_consume(
                             &queue,
                             &tag,
                             BasicConsumeOptions {
                                 no_local: consume_config.no_local,
                                 no_ack: consume_config.no_ack,
                                 exclusive: consume_config.exclusive,
                                 nowait: consume_config.nowait,
                             },
                             consume_config.arguments.clone(),
                         ).await;

                         if let Ok(mut stream) = stream_res {
                             while let Some(delivery) = stream.next().await {
                                 if let Ok(delivery) = delivery {
                                     let acker = delivery.acker.clone();
                                     let req = MqRequest {
                                         context: MqContext {
                                             delivery: Some(delivery),
                                             channel: Some(channel.clone()),
                                         }
                                     };

                                     match service.ready().await {
                                         Ok(svc) => {
                                             if let Err(e) = svc.call(req).await {
                                                 eprintln!("Consumer error: {:?}", e);
                                                 if !consume_config.no_ack {
                                                     let _ = acker.nack(BasicNackOptions::default()).await;
                                                 }
                                             } else {
                                                 if !consume_config.no_ack {
                                                     let _ = acker.ack(BasicAckOptions::default()).await;
                                                 }
                                             }
                                         }
                                         Err(e) => {
                                             eprintln!("Service not ready: {:?}", e);
                                             if !consume_config.no_ack {
                                                 let _ = acker.nack(BasicNackOptions::default()).await;
                                             }
                                         }
                                     }
                                 }
                             }
                         }
                    }
                });
                handles.push(handle);
            }
        }

        // Wait for all handles
        for h in handles {
            let _ = h.await;
        }

        Ok(())
    }
}

fn parse_exchange_kind(kind: &str) -> Result<ExchangeKind, Error> {
    match kind.to_ascii_lowercase().as_str() {
        "direct" => Ok(ExchangeKind::Direct),
        "fanout" => Ok(ExchangeKind::Fanout),
        "topic" => Ok(ExchangeKind::Topic),
        "headers" => Ok(ExchangeKind::Headers),
        _ => Err(Error::Other(format!("invalid exchange kind: {kind}"))),
    }
}
