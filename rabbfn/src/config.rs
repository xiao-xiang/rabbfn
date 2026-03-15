use crate::service::MqRequest;
use crate::extract::Error;
use lapin::types::FieldTable;
use std::str::FromStr;

pub type BoxMqService = tower::util::BoxCloneService<MqRequest, (), Error>;

#[derive(Clone, Debug)]
pub struct BindingConfig {
    pub exchange: String,
    pub routing_key: String,
    pub nowait: bool,
    pub arguments: FieldTable,
}

#[derive(Clone, Debug)]
pub struct ExchangeConfig {
    pub name: String,
    pub kind: String,
    pub passive: bool,
    pub durable: bool,
    pub auto_delete: bool,
    pub internal: bool,
    pub nowait: bool,
    pub declare: bool,
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

#[derive(Clone, Debug, Default)]
pub struct QosConfig {
    pub prefetch_count: u16,
    pub global: bool,
}

#[derive(Clone, Debug)]
pub struct QueueConfig {
    pub name: String,
    pub passive: bool,
    pub durable: bool,
    pub exclusive: bool,
    pub auto_delete: bool,
    pub nowait: bool,
    pub declare: bool,
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

#[derive(Clone, Debug, Default)]
pub enum TopologyMode {
    #[default]
    Managed,
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

#[derive(Clone, Debug)]
pub struct ConsumeConfig {
    pub consumer_tag: String,
    pub no_local: bool,
    pub no_ack: bool,
    pub exclusive: bool,
    pub nowait: bool,
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

pub struct ConsumerDefinition {
    pub queue_config: QueueConfig,
    pub exchanges: Vec<ExchangeConfig>,
    pub concurrency: usize,
    pub qos: QosConfig,
    pub consume_config: ConsumeConfig,
    pub bindings: Vec<BindingConfig>,
    pub service: BoxMqService,
}

pub trait IntoConsumerDefinition {
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
