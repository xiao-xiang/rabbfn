use tower::Service;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use crate::extract::{MqContext, Error};
use crate::handler::Handler;

// 请求体
pub struct MqRequest {
    pub context: MqContext,
}

use std::marker::PhantomData;

// ...

// 包装 Handler 的 Service
pub struct HandlerService<H, Args> {
    handler: H,
    _marker: PhantomData<Args>,
}

impl<H, Args> Clone for HandlerService<H, Args>
where
    H: Clone,
{
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            _marker: PhantomData,
        }
    }
}

impl<H, Args> HandlerService<H, Args> {
    pub fn new(handler: H) -> Self {
        Self { handler, _marker: PhantomData }
    }
}

impl<H, Args> Service<MqRequest> for HandlerService<H, Args>
where
    H: Handler<Args> + Clone + Send + Sync + 'static,
    Args: Send + Sync + 'static,
{
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: MqRequest) -> Self::Future {
        let handler = self.handler.clone();
        let ctx = req.context;

        Box::pin(async move {
            handler.call(ctx).await
        })
    }
}
