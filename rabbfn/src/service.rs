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
#[derive(Clone)]
pub struct HandlerService<H, S, Args> {
    handler: H,
    state: S,
    _marker: PhantomData<Args>,
}

impl<H, S, Args> HandlerService<H, S, Args> {
    pub fn new(handler: H, state: S) -> Self {
        Self { handler, state, _marker: PhantomData }
    }
}

impl<H, S, Args> Service<MqRequest> for HandlerService<H, S, Args>
where
    H: Handler<S, Args> + Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
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
        let state = self.state.clone();
        let ctx = req.context;

        Box::pin(async move {
            handler.call(ctx, state).await
        })
    }
}
