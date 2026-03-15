use async_trait::async_trait;
use crate::extract::{MqContext, FromMessage, Error};
use std::future::Future;

#[async_trait]
pub trait Handler<S, Args>: Clone + Send + Sync + 'static {
    async fn call(&self, ctx: MqContext, state: S) -> Result<(), Error>;
}

// 宏实现 Handler Trait for Fn
macro_rules! impl_handler {
    ( $($ty:ident),* ) => {
        #[allow(non_snake_case)]
        #[async_trait]
        impl<F, Fut, S, $($ty,)*> Handler<S, ($($ty,)*)> for F
        where
            F: Fn($($ty,)*) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = Result<(), Error>> + Send,
            S: Clone + Send + Sync + 'static,
            $( $ty: FromMessage<S> + Send, )*
        {
            #[allow(unused_mut, unused_variables)]
            async fn call(&self, mut ctx: MqContext, state: S) -> Result<(), Error> {
                $(
                    let $ty = $ty::from_message(&mut ctx, &state).await?;
                )*
                (self)($($ty,)*).await
            }
        }
    };
}

impl_handler!();
impl_handler!(T1);
impl_handler!(T1, T2);
impl_handler!(T1, T2, T3);
impl_handler!(T1, T2, T3, T4);
impl_handler!(T1, T2, T3, T4, T5);
impl_handler!(T1, T2, T3, T4, T5, T6);
impl_handler!(T1, T2, T3, T4, T5, T6, T7);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8);
