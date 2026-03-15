use crate::service::MqRequest;
use crate::extract::Error;
use lapin::types::FieldTable;
use std::str::FromStr;

/// Type alias for a cloneable, type-erased consumer service.
pub type BoxMqService = tower::util::BoxCloneService<MqRequest, (), Error>;

/// Queue binding declaration parameters.
#[derive(Clone, Debug)]
pub struct BindingConfig {
    /// Source exchange name.
    pub exchange: String,
    /// Binding routing key.
    pub routing_key: String,
    /// AMQP `nowait` flag for `queue.bind`.
    pub nowait: bool,
    /// AMQP arguments for `queue.bind`.
    pub arguments: FieldTable,
}

/// Exchange declaration parameters.
#[derive(Clone, Debug)]
pub struct ExchangeConfig {
    /// Exchange name.
    pub name: String,
    /// Exchange kind, for example `direct`, `fanout`, `topic`, or `headers`.
    pub kind: String,
    /// AMQP `passive` flag for `exchange.declare`.
    pub passive: bool,
    /// AMQP `durable` flag for `exchange.declare`.
    pub durable: bool,
    /// AMQP `auto_delete` flag for `exchange.declare`.
    pub auto_delete: bool,
    /// AMQP `internal` flag for `exchange.declare`.
    pub internal: bool,
    /// AMQP `nowait` flag for `exchange.declare`.
    pub nowait: bool,
    /// Whether this exchange is declared during server startup.
    pub declare: bool,
    /// AMQP arguments for `exchange.declare`.
    pub arguments: FieldTable,
}

impl Default for ExchangeConfig {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            kind: "direct".to_string(),
            passive: false,
            durable: true,
            auto_delete: false,
            internal: false,
            nowait: false,
            declare: true,
            arguments: FieldTable::default(),
        }
    }
}

/// QoS settings for one consumer.
#[derive(Clone, Debug, Default)]
pub struct QosConfig {
    /// AMQP `prefetch_count` for `basic.qos`.
    pub prefetch_count: u16,
    /// AMQP `global` flag for `basic.qos`.
    pub global: bool,
}

/// Queue declaration parameters.
#[derive(Clone, Debug)]
pub struct QueueConfig {
    /// Queue name.
    pub name: String,
    /// AMQP `passive` flag for `queue.declare`.
    pub passive: bool,
    /// AMQP `durable` flag for `queue.declare`.
    pub durable: bool,
    /// AMQP `exclusive` flag for `queue.declare`.
    pub exclusive: bool,
    /// AMQP `auto_delete` flag for `queue.declare`.
    pub auto_delete: bool,
    /// AMQP `nowait` flag for `queue.declare`.
    pub nowait: bool,
    /// Whether this queue is declared during server startup.
    pub declare: bool,
    /// AMQP arguments for `queue.declare`.
    pub arguments: FieldTable,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            passive: false,
            durable: true,
            exclusive: false,
            auto_delete: false,
            nowait: false,
            declare: true,
            arguments: FieldTable::default(),
        }
    }
}

/// Topology declaration strategy.
#[derive(Clone, Debug, Default)]
pub enum TopologyMode {
    /// Declares exchanges, queues, and bindings during startup.
    #[default]
    Managed,
    /// Assumes topology already exists and only starts consumers.
    External,
}

impl FromStr for TopologyMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "managed" => Ok(TopologyMode::Managed),
            "external" => Ok(TopologyMode::External),
            _ => Err(format!("invalid topology mode: {s}, expected managed|external")),
        }
    }
}

/// Consumer registration parameters for `basic.consume`.
#[derive(Clone, Debug)]
pub struct ConsumeConfig {
    /// Consumer tag prefix.
    pub consumer_tag: String,
    /// AMQP `no_local` flag for `basic.consume`.
    pub no_local: bool,
    /// AMQP `no_ack` flag for `basic.consume`.
    pub no_ack: bool,
    /// AMQP `exclusive` flag for `basic.consume`.
    pub exclusive: bool,
    /// AMQP `nowait` flag for `basic.consume`.
    pub nowait: bool,
    /// AMQP arguments for `basic.consume`.
    pub arguments: FieldTable,
}

impl Default for ConsumeConfig {
    fn default() -> Self {
        Self {
            consumer_tag: "".to_string(),
            no_local: false,
            no_ack: false,
            exclusive: false,
            nowait: false,
            arguments: FieldTable::default(),
        }
    }
}

/// Fully resolved consumer descriptor consumed by [`crate::server::RabbitMqServer`].
pub struct ConsumerDefinition {
    /// Queue declaration configuration.
    pub queue_config: QueueConfig,
    /// Exchange declaration configurations.
    pub exchanges: Vec<ExchangeConfig>,
    /// Number of worker tasks spawned for this consumer.
    pub concurrency: usize,
    /// QoS settings.
    pub qos: QosConfig,
    /// `basic.consume` options.
    pub consume_config: ConsumeConfig,
    /// Queue binding declarations.
    pub bindings: Vec<BindingConfig>,
    /// Executable consumer service.
    pub service: BoxMqService,
}

/// Conversion trait for values accepted by `add_consumer`.
pub trait IntoConsumerDefinition {
    /// Converts `self` into a [`ConsumerDefinition`].
    fn into_definition(self) -> ConsumerDefinition;
}

impl IntoConsumerDefinition for ConsumerDefinition {
    fn into_definition(self) -> ConsumerDefinition {
        self
    }
}

impl<F> IntoConsumerDefinition for F
where
    F: FnOnce() -> ConsumerDefinition,
{
    fn into_definition(self) -> ConsumerDefinition {
        self()
    }
}
