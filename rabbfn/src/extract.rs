use lapin::message::Delivery;
use lapin::Channel;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use thiserror::Error;
use crate::state::StateStore;
#[derive(Error, Debug)]
pub enum Error {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("AMQP error: {0}")]
    Amqp(#[from] lapin::Error),
    #[error("Other error: {0}")]
    Other(String),
}

pub struct MqContext {
    pub delivery: Option<Delivery>,
    pub channel: Option<Channel>,
    pub states: StateStore,
}

impl MqContext {
    pub fn take_delivery(&mut self) -> Result<Delivery, Error> {
        self.delivery.take().ok_or_else(|| Error::Other("Delivery already taken".to_string()))
    }
}

#[async_trait]
pub trait FromMessage: Sized {
    async fn from_message(ctx: &mut MqContext) -> Result<Self, Error>;
}

// 1. State 提取器
// Use a wrapper struct for State to avoid conflict with generic S
pub struct State<S>(pub S);

#[async_trait]
impl<S> FromMessage for State<S>
where
    S: Clone + Send + Sync + 'static,
{
    async fn from_message(ctx: &mut MqContext) -> Result<Self, Error> {
        ctx.states
            .get::<S>()
            .map(State)
            .ok_or_else(|| Error::Other(format!("State not found: {}", std::any::type_name::<S>())))
    }
}

// 2. Delivery 提取器
pub struct RawDelivery(pub Delivery);

#[async_trait]
impl FromMessage for RawDelivery {
    async fn from_message(ctx: &mut MqContext) -> Result<Self, Error> {
        Ok(RawDelivery(ctx.take_delivery()?))
    }
}

// 3. Json<T> 提取器
pub struct Json<T>(pub T);

#[async_trait]
impl<T: DeserializeOwned + Send> FromMessage for Json<T> {
    async fn from_message(ctx: &mut MqContext) -> Result<Self, Error> {
        if let Some(delivery) = &ctx.delivery {
             serde_json::from_slice(&delivery.data)
                .map(Json)
                .map_err(Error::Json)
        } else {
            Err(Error::Other("Delivery missing or already taken".to_string()))
        }
    }
}

// 4. Channel 提取器
#[async_trait]
impl FromMessage for Channel {
    async fn from_message(ctx: &mut MqContext) -> Result<Self, Error> {
        ctx.channel.clone().ok_or_else(|| Error::Other("Channel not available".to_string()))
    }
}
