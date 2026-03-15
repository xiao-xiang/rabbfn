extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr, Ident, ItemFn, Token, parenthesized};
use syn::parse::{Parse, ParseStream, Parser};

struct ConsumerArgs {
    queue: Expr,
    queue_options: QueueOptions,
    exchanges: Vec<ExchangeSpec>,
    bindings: Vec<BindingSpec>,
    concurrency: Expr,
    qos: QosConfig,
    consume_options: ConsumeOptions,
}

struct QosConfig {
    prefetch_count: Expr,
    global: Expr,
}

struct ConsumeOptions {
    consumer_tag: Expr,
    no_local: Expr,
    no_ack: Expr,
    exclusive: Expr,
    nowait: Expr,
    arguments: Expr,
}

struct QueueOptions {
    passive: Expr,
    durable: Expr,
    exclusive: Expr,
    auto_delete: Expr,
    nowait: Expr,
    declare: Expr,
    arguments: Expr,
}

struct ExchangeOptions {
    passive: Expr,
    durable: Expr,
    auto_delete: Expr,
    internal: Expr,
    nowait: Expr,
    declare: Expr,
    arguments: Expr,
}

struct ExchangeSpec {
    name: Expr,
    kind: Expr,
    options: ExchangeOptions,
}

struct BindingSpec {
    exchange: Expr,
    routing_key: Expr,
    nowait: Expr,
    arguments: Option<Expr>,
}

fn to_pascal_case(input: &str) -> String {
    let mut out = String::new();
    for part in input.split('_') {
        if part.is_empty() {
            continue;
        }
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

impl Default for QosConfig {
    fn default() -> Self {
        Self {
            prefetch_count: syn::parse_str("10").unwrap(),
            global: syn::parse_str("false").unwrap(),
        }
    }
}

impl Default for ConsumeOptions {
    fn default() -> Self {
        Self {
            consumer_tag: syn::parse_str("\"\"").unwrap(),
            no_local: syn::parse_str("false").unwrap(),
            no_ack: syn::parse_str("false").unwrap(),
            exclusive: syn::parse_str("false").unwrap(),
            nowait: syn::parse_str("false").unwrap(),
            arguments: syn::parse_str("lapin::types::FieldTable::default()").unwrap(),
        }
    }
}

impl Default for QueueOptions {
    fn default() -> Self {
        Self {
            passive: syn::parse_str("false").unwrap(),
            durable: syn::parse_str("true").unwrap(),
            exclusive: syn::parse_str("false").unwrap(),
            auto_delete: syn::parse_str("false").unwrap(),
            nowait: syn::parse_str("false").unwrap(),
            declare: syn::parse_str("true").unwrap(),
            arguments: syn::parse_str("lapin::types::FieldTable::default()").unwrap(),
        }
    }
}

impl Default for ExchangeOptions {
    fn default() -> Self {
        Self {
            passive: syn::parse_str("false").unwrap(),
            durable: syn::parse_str("true").unwrap(),
            auto_delete: syn::parse_str("false").unwrap(),
            internal: syn::parse_str("false").unwrap(),
            nowait: syn::parse_str("false").unwrap(),
            declare: syn::parse_str("true").unwrap(),
            arguments: syn::parse_str("lapin::types::FieldTable::default()").unwrap(),
        }
    }
}

impl Parse for ConsumerArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut queue = None;
        let mut concurrency = None;
        let mut qos = QosConfig::default();
        let mut consume_options = ConsumeOptions::default();
        let mut queue_options = QueueOptions::default();
        let mut exchanges: Vec<ExchangeSpec> = Vec::new();
        let mut bindings: Vec<BindingSpec> = Vec::new();

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            
            if key == "queue" {
                if input.peek(Token![=]) {
                    let _ = input.parse::<Token![=]>()?;
                    queue = Some(input.parse()?);
                } else if input.peek(syn::token::Paren) {
                    let content;
                    parenthesized!(content in input);
                    while !content.is_empty() {
                        let q_key: Ident = content.parse()?;
                        let _ = content.parse::<Token![=]>()?;
                        if q_key == "name" {
                            queue = Some(content.parse()?);
                        } else if q_key == "passive" {
                            queue_options.passive = content.parse()?;
                        } else if q_key == "durable" {
                            queue_options.durable = content.parse()?;
                        } else if q_key == "exclusive" {
                            queue_options.exclusive = content.parse()?;
                        } else if q_key == "auto_delete" {
                            queue_options.auto_delete = content.parse()?;
                        } else if q_key == "nowait" {
                            queue_options.nowait = content.parse()?;
                        } else if q_key == "declare" {
                            queue_options.declare = content.parse()?;
                        } else if q_key == "arguments" {
                            queue_options.arguments = content.parse()?;
                        } else {
                            return Err(syn::Error::new(q_key.span(), "unknown queue option"));
                        }
                        if !content.is_empty() { let _ = content.parse::<Token![,]>()?; }
                    }
                } else {
                    return Err(syn::Error::new(key.span(), "queue expects '=' or '(...)'"));
                }
            } else if key == "concurrency" {
                let _ = input.parse::<Token![=]>()?;
                concurrency = Some(input.parse()?);
            } else if key == "prefetch" {
                // Legacy support: prefetch = 10
                let _ = input.parse::<Token![=]>()?;
                qos.prefetch_count = input.parse()?;
            } else if key == "qos" {
                let content;
                parenthesized!(content in input);
                while !content.is_empty() {
                    let q_key: Ident = content.parse()?;
                    let _ = content.parse::<Token![=]>()?;
                    
                    if q_key == "prefetch_count" {
                        qos.prefetch_count = content.parse()?;
                    } else if q_key == "global" {
                        qos.global = content.parse()?;
                    } else {
                        return Err(syn::Error::new(q_key.span(), "unknown qos option"));
                    }
                    
                    if !content.is_empty() {
                        let _ = content.parse::<Token![,]>()?;
                    }
                }
            } else if key == "queue_options" {
                let content;
                parenthesized!(content in input);
                while !content.is_empty() {
                    let q_key: Ident = content.parse()?;
                    let _ = content.parse::<Token![=]>()?;
                    if q_key == "passive" {
                        queue_options.passive = content.parse()?;
                    } else if q_key == "durable" {
                        queue_options.durable = content.parse()?;
                    } else if q_key == "exclusive" {
                        queue_options.exclusive = content.parse()?;
                    } else if q_key == "auto_delete" {
                        queue_options.auto_delete = content.parse()?;
                    } else if q_key == "nowait" {
                        queue_options.nowait = content.parse()?;
                    } else if q_key == "declare" {
                        queue_options.declare = content.parse()?;
                    } else if q_key == "arguments" {
                        queue_options.arguments = content.parse()?;
                    } else {
                        return Err(syn::Error::new(q_key.span(), "unknown queue_options option"));
                    }
                    if !content.is_empty() { let _ = content.parse::<Token![,]>()?; }
                }
            } else if key == "consume_options" {
                let content;
                parenthesized!(content in input);
                while !content.is_empty() {
                    let c_key: Ident = content.parse()?;
                    let _ = content.parse::<Token![=]>()?;
                    if c_key == "consumer_tag" {
                        consume_options.consumer_tag = content.parse()?;
                    } else if c_key == "no_local" {
                        consume_options.no_local = content.parse()?;
                    } else if c_key == "no_ack" {
                        consume_options.no_ack = content.parse()?;
                    } else if c_key == "exclusive" {
                        consume_options.exclusive = content.parse()?;
                    } else if c_key == "nowait" {
                        consume_options.nowait = content.parse()?;
                    } else if c_key == "arguments" {
                        consume_options.arguments = content.parse()?;
                    } else {
                        return Err(syn::Error::new(c_key.span(), "unknown consume_options option"));
                    }
                    if !content.is_empty() { let _ = content.parse::<Token![,]>()?; }
                }
            } else if key == "exchanges" {
                let _ = input.parse::<Token![=]>()?;
                let content;
                syn::bracketed!(content in input);
                while !content.is_empty() {
                    let inner;
                    syn::parenthesized!(inner in content);
                    let mut name: Option<Expr> = None;
                    let mut kind: Option<Expr> = None;
                    let mut options = ExchangeOptions::default();
                    while !inner.is_empty() {
                        let e_key: Ident = inner.parse()?;
                        let _ = inner.parse::<Token![=]>()?;
                        if e_key == "name" {
                            name = Some(inner.parse()?);
                        } else if e_key == "kind" {
                            kind = Some(inner.parse()?);
                        } else if e_key == "passive" {
                            options.passive = inner.parse()?;
                        } else if e_key == "durable" {
                            options.durable = inner.parse()?;
                        } else if e_key == "auto_delete" {
                            options.auto_delete = inner.parse()?;
                        } else if e_key == "internal" {
                            options.internal = inner.parse()?;
                        } else if e_key == "nowait" {
                            options.nowait = inner.parse()?;
                        } else if e_key == "declare" {
                            options.declare = inner.parse()?;
                        } else if e_key == "arguments" {
                            options.arguments = inner.parse()?;
                        } else {
                            return Err(syn::Error::new(e_key.span(), "unknown exchange option in exchanges list"));
                        }
                        if !inner.is_empty() { let _ = inner.parse::<Token![,]>()?; }
                    }
                    exchanges.push(ExchangeSpec {
                        name: name.ok_or_else(|| syn::Error::new(key.span(), "exchange name is required"))?,
                        kind: kind.unwrap_or_else(|| syn::parse_str("\"direct\"").unwrap()),
                        options,
                    });
                    if !content.is_empty() {
                        let _ = content.parse::<Token![,]>()?;
                    }
                }
            } else if key == "bindings" {
                let _ = input.parse::<Token![=]>()?;
                let content;
                syn::bracketed!(content in input);
                while !content.is_empty() {
                    let inner;
                    syn::parenthesized!(inner in content);
                    let mut exchange: Option<Expr> = None;
                    let mut routing_key: Option<Expr> = None;
                    let mut nowait: Expr = syn::parse_str("false").unwrap();
                    let mut arguments: Option<Expr> = None;

                    while !inner.is_empty() {
                        let b_key: Ident = inner.parse()?;
                        let _ = inner.parse::<Token![=]>()?;
                        if b_key == "exchange" {
                            exchange = Some(inner.parse()?);
                        } else if b_key == "routing_key" {
                            routing_key = Some(inner.parse()?);
                        } else if b_key == "nowait" {
                            nowait = inner.parse()?;
                        } else if b_key == "arguments" {
                            arguments = Some(inner.parse()?);
                        } else {
                            return Err(syn::Error::new(b_key.span(), "unknown binding option in bindings list"));
                        }
                        if !inner.is_empty() { let _ = inner.parse::<Token![,]>()?; }
                    }

                    bindings.push(BindingSpec {
                        exchange: exchange.ok_or_else(|| syn::Error::new(key.span(), "binding exchange is required"))?,
                        routing_key: routing_key.unwrap_or_else(|| syn::parse_str("\"\"").unwrap()),
                        nowait,
                        arguments,
                    });
                    
                    if !content.is_empty() {
                        let _ = content.parse::<Token![,]>()?;
                    }
                }
            } else {
                return Err(syn::Error::new(key.span(), "unknown consumer option (use exchanges=[(...)] and bindings=[(...)])"));
            }

            if !input.is_empty() {
                let _ = input.parse::<Token![,]>()?;
            }
        }

        Ok(ConsumerArgs {
            queue: queue.expect("queue is required"),
            queue_options,
            exchanges,
            bindings,
            concurrency: concurrency.unwrap_or_else(|| syn::parse_str("1").unwrap()),
            qos,
            consume_options,
        })
    }
}

#[proc_macro_attribute]
pub fn consumer(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    
    let args_parser = ConsumerArgs::parse;
    let args = args_parser.parse(args).expect("Failed to parse consumer args");

    let fn_name = &input_fn.sig.ident;
    let struct_name = syn::Ident::new(&format!("{}Consumer", to_pascal_case(&fn_name.to_string())), fn_name.span());
    let handler_fn_name = syn::Ident::new(&format!("__rabbfn_handler_{}", fn_name), fn_name.span());
    let with_state_fn_name = syn::Ident::new(&format!("{}_with_state", fn_name), fn_name.span());
    let fn_vis = &input_fn.vis;
    let mut handler_fn = input_fn.clone();
    handler_fn.sig.ident = handler_fn_name.clone();

    let queue = args.queue;
    let concurrency = args.concurrency;
    let prefetch_count = args.qos.prefetch_count;
    let qos_global = args.qos.global;
    let consume_consumer_tag = args.consume_options.consumer_tag;
    let consume_no_local = args.consume_options.no_local;
    let consume_no_ack = args.consume_options.no_ack;
    let consume_exclusive = args.consume_options.exclusive;
    let consume_nowait = args.consume_options.nowait;
    let consume_arguments = args.consume_options.arguments;
    
    // Queue Options
    let q_durable = args.queue_options.durable;
    let q_exclusive = args.queue_options.exclusive;
    let q_auto_delete = args.queue_options.auto_delete;
    let q_passive = args.queue_options.passive;
    let q_nowait = args.queue_options.nowait;
    let q_declare = args.queue_options.declare;
    let q_arguments = args.queue_options.arguments;

    let exchanges_code = args.exchanges.iter().map(|exchange| {
        let name = &exchange.name;
        let kind = &exchange.kind;
        let passive = &exchange.options.passive;
        let durable = &exchange.options.durable;
        let auto_delete = &exchange.options.auto_delete;
        let internal = &exchange.options.internal;
        let nowait = &exchange.options.nowait;
        let declare = &exchange.options.declare;
        let arguments = &exchange.options.arguments;
        quote! {
            rabbfn::config::ExchangeConfig {
                name: #name.to_string(),
                kind: #kind.to_string(),
                passive: #passive,
                durable: #durable,
                auto_delete: #auto_delete,
                internal: #internal,
                nowait: #nowait,
                declare: #declare,
                arguments: #arguments,
            }
        }
    });

    let bindings_code = args.bindings.iter().map(|binding| {
        let ex = &binding.exchange;
        let rk = &binding.routing_key;
        let nowait = &binding.nowait;
        let args_code = match &binding.arguments {
            Some(a) => quote! { #a },
            None => quote! { lapin::types::FieldTable::default() },
        };
        quote! {
            rabbfn::config::BindingConfig {
                exchange: #ex.to_string(),
                routing_key: #rk.to_string(),
                nowait: #nowait,
                arguments: #args_code,
            }
        }
    });

    let expanded = quote! {
        #handler_fn

        #[derive(Clone)]
        #fn_vis struct #struct_name<S> {
            state: Option<S>,
        }

        impl<S> #struct_name<S> {
            pub fn new() -> Self {
                Self { state: None }
            }
            
            pub fn with_state(mut self, state: S) -> Self {
                self.state = Some(state);
                self
            }
        }

        #[allow(non_upper_case_globals)]
        #fn_vis const #fn_name: #struct_name<()> = #struct_name { state: Some(()) };

        #fn_vis fn #with_state_fn_name<S>(state: S) -> #struct_name<S> {
            #struct_name::new().with_state(state)
        }
        
        impl<S> rabbfn::config::ConsumerConfig for #struct_name<S> {
            fn queue_config(&self) -> rabbfn::config::QueueConfig {
                rabbfn::config::QueueConfig {
                    name: #queue.to_string(),
                    passive: #q_passive,
                    durable: #q_durable,
                    exclusive: #q_exclusive,
                    auto_delete: #q_auto_delete,
                    nowait: #q_nowait,
                    declare: #q_declare,
                    arguments: #q_arguments,
                }
            }
            fn exchanges(&self) -> Vec<rabbfn::config::ExchangeConfig> {
                vec![
                    #(#exchanges_code),*
                ]
            }
            fn concurrency(&self) -> usize {
                #concurrency
            }
            fn qos(&self) -> rabbfn::config::QosConfig {
                rabbfn::config::QosConfig {
                    prefetch_count: #prefetch_count,
                    global: #qos_global,
                }
            }
            fn consume_config(&self) -> rabbfn::config::ConsumeConfig {
                rabbfn::config::ConsumeConfig {
                    consumer_tag: #consume_consumer_tag.to_string(),
                    no_local: #consume_no_local,
                    no_ack: #consume_no_ack,
                    exclusive: #consume_exclusive,
                    nowait: #consume_nowait,
                    arguments: #consume_arguments,
                }
            }
            fn bindings(&self) -> Vec<rabbfn::config::BindingConfig> {
                vec![
                    #(#bindings_code),*
                ]
            }
        }

        impl<S> tower::Service<rabbfn::service::MqRequest> for #struct_name<S>
        where
            S: Clone + Send + Sync + 'static,
        {
            type Response = ();
            type Error = rabbfn::extract::Error;
            type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Self::Error>> + Send>>;

            fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
                std::task::Poll::Ready(Ok(()))
            }

            fn call(&mut self, req: rabbfn::service::MqRequest) -> Self::Future {
                let handler = #handler_fn_name;
                let state = self.state.clone().expect("State must be injected using with_state()");
                
                Box::pin(async move {
                    rabbfn::handler::Handler::call(&handler, req.context, state).await
                })
            }
        }
    };

    TokenStream::from(expanded)
}
