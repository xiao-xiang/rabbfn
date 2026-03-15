use lapin::message::Delivery;
use lapin::Channel;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use thiserror::Error;
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
}

impl MqContext {
    pub fn take_delivery(&mut self) -> Result<Delivery, Error> {
        self.delivery.take().ok_or_else(|| Error::Other("Delivery already taken".to_string()))
    }
}

#[async_trait]
pub trait FromMessage<S>: Sized {
    async fn from_message(ctx: &mut MqContext, state: &S) -> Result<Self, Error>;
}

// 1. State 提取器
// Use a wrapper struct for State to avoid conflict with generic S
pub struct State<S>(pub S);

#[async_trait]
impl<S: Clone + Send + Sync> FromMessage<S> for State<S> {
    async fn from_message(_ctx: &mut MqContext, state: &S) -> Result<Self, Error> {
        Ok(State(state.clone()))
    }
}

// 2. Delivery 提取器
pub struct RawDelivery(pub Delivery);

#[async_trait]
impl<S: Send + Sync> FromMessage<S> for RawDelivery {
    async fn from_message(ctx: &mut MqContext, _state: &S) -> Result<Self, Error> {
        Ok(RawDelivery(ctx.take_delivery()?))
    }
}

// 3. Json<T> 提取器
pub struct Json<T>(pub T);

#[async_trait]
impl<T: DeserializeOwned + Send, S: Send + Sync> FromMessage<S> for Json<T> {
    async fn from_message(ctx: &mut MqContext, _state: &S) -> Result<Self, Error> {
        // We need to access delivery data.
        // If delivery is already taken, we fail?
        // Ideally we should peek.
        // But if RawDelivery took it, we can't peek.
        // Axum solves this by enforcing order or cloning body (Bytes is cheap clone).
        // Delivery is not cheap clone.
        // But Delivery.data is Vec<u8>.
        // We should probably just clone the data?
        // But we can't clone Delivery.
        
        // For now, let's assume we can reference it if it's there.
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
impl<S: Send + Sync> FromMessage<S> for Channel {
    async fn from_message(ctx: &mut MqContext, _state: &S) -> Result<Self, Error> {
        ctx.channel.clone().ok_or_else(|| Error::Other("Channel not available".to_string()))
    }
}
