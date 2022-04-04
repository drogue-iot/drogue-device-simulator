use crate::simulator::simulations::Context;
use futures::channel::mpsc;
use futures::channel::mpsc::{SendError, UnboundedReceiver, UnboundedSender};
use futures::{select, FutureExt};
use futures::{SinkExt, StreamExt};
use gloo_timers::future::TimeoutFuture;
use js_sys::Date;
use num_traits::ToPrimitive;
use std::time::Duration;
use wasm_bindgen_futures::spawn_local;

pub trait SenderConfiguration: Clone + 'static {
    fn delay(&self) -> Duration;
}

pub enum Msg<S, C>
where
    C: SenderConfiguration,
    S: Clone + 'static,
{
    Update(S),
    Configure(C),
}

pub struct Sender<S, C>
where
    C: SenderConfiguration,
    S: Clone + 'static,
{
    rx: UnboundedReceiver<Msg<S, C>>,
    tx: SenderHandle<S, C>,
    ctx: Context,
    config: C,
    delay: Duration,
    initial_state: S,
    f: Box<dyn Fn(&SenderHandle<S, C>, &Context, &C, &S)>,
}

#[derive(Clone)]
pub struct SenderHandle<S, C>
where
    C: SenderConfiguration,
    S: Clone + 'static,
{
    tx: UnboundedSender<Msg<S, C>>,
}

impl<S, C> SenderHandle<S, C>
where
    C: SenderConfiguration,
    S: Clone + 'static,
{
    pub async fn configure(&mut self, config: C) -> Result<(), SendError> {
        self.tx.send(Msg::Configure(config)).await
    }

    pub async fn update(&mut self, state: S) -> Result<(), SendError> {
        self.tx.send(Msg::Update(state)).await
    }

    pub fn to_sync(&self) -> SyncSenderHandle<S, C> {
        SyncSenderHandle {
            inner: self.clone(),
        }
    }
}

#[derive(Clone)]
pub struct SyncSenderHandle<S, C>
where
    C: SenderConfiguration,
    S: Clone + 'static,
{
    inner: SenderHandle<S, C>,
}

impl<S, C> SyncSenderHandle<S, C>
where
    C: SenderConfiguration,
    S: Clone + 'static,
{
    pub fn configure(&self, config: C) {
        let mut handle = self.inner.clone();
        spawn_local(async move {
            handle.configure(config).await.ok();
        });
    }

    pub fn update(&self, state: S) {
        let mut handle = self.inner.clone();
        spawn_local(async move {
            handle.update(state).await.ok();
        });
    }
}

impl<S, C> Sender<S, C>
where
    C: SenderConfiguration,
    S: Clone + 'static,
{
    pub fn new<F>(ctx: Context, config: C, initial_state: S, f: F) -> (SenderHandle<S, C>, Self)
    where
        F: Fn(&SenderHandle<S, C>, &Context, &C, &S) + 'static,
    {
        let (tx, rx) = mpsc::unbounded::<Msg<S, C>>();

        let delay = config.delay();
        let handle = SenderHandle { tx: tx.clone() };

        (
            handle.clone(),
            Self {
                rx,
                tx: handle,
                ctx,
                config,
                delay,
                initial_state,
                f: Box::new(f),
            },
        )
    }

    pub fn start(self) {
        spawn_local(async move { self.run().await });
    }

    async fn run(mut self) {
        let mut config = self.config;
        let mut delay = self.delay.as_millis().to_f64().unwrap_or(f64::MAX);
        let ctx = self.ctx;
        let f = self.f;
        let tx = self.tx;

        // internally this is a i32, so infinity is i32::MAX, but as u32
        const INFINITY: u32 = i32::MAX as u32;

        let mut state = self.initial_state;
        let mut next = Date::now();
        let mut timer = TimeoutFuture::new(INFINITY).fuse();

        // send an initial state

        f(&tx, &ctx, &config, &state);

        // now loop

        loop {
            select! {
                msg = self.rx.next() => match msg {
                    Some(Msg::Update(s)) => {
                        state = s;
                        let now = Date::now();
                        let rem = next - now;
                        if rem < 0f64 {
                            f(&tx, &ctx, &config, &state);
                            next = now + delay;
                        }  else {
                            timer = TimeoutFuture::new(rem.to_u32().unwrap_or(INFINITY)).fuse();
                        }
                    }
                    Some(Msg::Configure(new_config)) => {
                        delay = new_config.delay().as_millis().to_f64().unwrap_or(f64::MAX);
                        config = new_config;
                    }
                    None => {
                        self.rx.close();
                        break;
                    }
                },
                () = timer => {
                    f(&tx, &ctx, &config, &state);
                    timer = TimeoutFuture::new(INFINITY).fuse();
                }
            }
        }
    }
}
