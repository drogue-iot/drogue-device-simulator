pub mod sawtooth;
pub mod sine;
pub mod tick;

use crate::simulator::publish::Publisher;
use std::time::Duration;

const fn default_period() -> Duration {
    Duration::from_secs(1)
}

pub struct Context {
    publisher: Box<dyn Publisher>,
}

impl Context {
    pub fn new<P: Publisher + 'static>(publisher: P) -> Self {
        Self {
            publisher: Box::new(publisher),
        }
    }

    pub fn publisher(&mut self) -> &mut dyn Publisher {
        self.publisher.as_mut()
    }
}

pub trait Generator {
    type Properties;

    fn new(properties: Self::Properties) -> Self;
    fn update(&mut self, properties: Self::Properties);

    fn start(&mut self, ctx: Context);
    fn stop(&mut self);
}
